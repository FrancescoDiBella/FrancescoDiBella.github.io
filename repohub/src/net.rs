//! How requests reach GitHub, and how background work reports back to the UI.
//!
//! - Desktop: blocking HTTPS with `ureq` straight to the API (token from `.env`), run on a worker
//!   thread and driven by `pollster`.
//! - Web: `fetch` to the same-origin `/gh` proxy of `repohub-server`, which adds the token, so the
//!   token never reaches the page. Driven by `wasm_bindgen_futures`.
use std::future::Future;

use rune::AppHandle;

use crate::App;
use crate::config::Config;

/// One HTTP response, with the rate-limit headers the error messages use.
pub struct Resp {
    pub status: u16,
    pub body: String,
    pub remaining: Option<String>,
    pub reset: Option<i64>,
}

/// Runs `work` in the background and hands its result to `done` on the UI thread.
pub fn spawn<R, Fut>(h: AppHandle<App>, work: impl FnOnce() -> Fut + Send + 'static, done: impl FnOnce(&mut App, R) + Send + 'static)
where
    R: Send + 'static,
    Fut: Future<Output = R> + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        let r = pollster::block_on(work());
        h.update(move |s: &mut App, _| done(s, r));
    });
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let r = work().await;
        h.update(move |s: &mut App, _| done(s, r));
    });
}

// ------------------------------------------------------------------ desktop

#[cfg(not(target_arch = "wasm32"))]
pub struct Transport {
    agent: ureq::Agent,
    api: String,
    token: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Transport {
    pub fn new(cfg: &Config) -> Transport {
        let agent = ureq::AgentBuilder::new()
            .try_proxy_from_env(!bypass_proxy(&cfg.api))
            .timeout(std::time::Duration::from_secs(20))
            .user_agent("RepoHub (Rune)")
            .build();
        Transport { agent, api: cfg.api.clone(), token: cfg.token.clone() }
    }

    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    pub async fn send(&self, method: &str, path: &str, accept: &str) -> Result<Resp, String> {
        let mut r = self.agent.request(method, &format!("{}{}", self.api, path))
            .set("Accept", accept)
            .set("X-GitHub-Api-Version", "2022-11-28");
        if let Some(t) = &self.token {
            r = r.set("Authorization", &format!("Bearer {t}"));
        }
        let res = if method == "PUT" { r.set("Content-Length", "0").send_string("") } else { r.call() };
        let resp = match res {
            Ok(resp) | Err(ureq::Error::Status(_, resp)) => resp,
            Err(e) => return Err(format!("Impossibile raggiungere {}: {e}", self.api)),
        };
        let status = resp.status();
        let remaining = resp.header("x-ratelimit-remaining").map(str::to_owned);
        let reset = resp.header("x-ratelimit-reset").and_then(|v| v.parse().ok());
        let body = resp.into_string().map_err(|e| e.to_string())?;
        Ok(Resp { status, body, remaining, reset })
    }
}

/// True when NO_PROXY (or a loopback host) says to reach `api` directly. The HTTP client reads
/// HTTPS_PROXY by itself but not NO_PROXY.
#[cfg(not(target_arch = "wasm32"))]
fn bypass_proxy(api: &str) -> bool {
    let host = api.split("://").nth(1).unwrap_or(api).split(['/', ':']).next().unwrap_or("").to_ascii_lowercase();
    if host == "localhost" || host.starts_with("127.") || host == "[::1]" {
        return true;
    }
    let list = std::env::var("NO_PROXY").or_else(|_| std::env::var("no_proxy")).unwrap_or_default();
    list.split(',').map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty())
        .any(|e| e == "*" || host == e || host.ends_with(&format!(".{e}")))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

// ------------------------------------------------------------------ web

#[cfg(target_arch = "wasm32")]
mod js {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(inline_js = "
export async function gh_fetch(method, url, accept) {
    try {
        const r = await fetch(url, { method, headers: { Accept: accept }, credentials: 'same-origin' });
        const body = await r.text();
        return [r.status, body, r.headers.get('x-ratelimit-remaining') || '', r.headers.get('x-ratelimit-reset') || ''];
    } catch (e) {
        return [0, String(e), '', ''];
    }
}
export function now_secs() { return Math.floor(Date.now() / 1000); }
")]
    extern "C" {
        pub async fn gh_fetch(method: &str, url: &str, accept: &str) -> JsValue;
        pub fn now_secs() -> f64;
    }
}

#[cfg(target_arch = "wasm32")]
pub struct Transport {
    has_token: bool,
}

#[cfg(target_arch = "wasm32")]
impl Transport {
    pub fn new(cfg: &Config) -> Transport {
        Transport { has_token: cfg.token.is_some() }
    }

    pub fn has_token(&self) -> bool {
        self.has_token
    }

    pub async fn send(&self, method: &str, path: &str, accept: &str) -> Result<Resp, String> {
        let v = js::gh_fetch(method, &format!("gh{path}"), accept).await;
        let a = js_sys::Array::from(&v);
        let status = a.get(0).as_f64().unwrap_or(0.0) as u16;
        let body = a.get(1).as_string().unwrap_or_default();
        if status == 0 {
            return Err(format!("Impossibile raggiungere il server di RepoHub: {body}"));
        }
        let remaining = a.get(2).as_string().filter(|s| !s.is_empty());
        let reset = a.get(3).as_string().and_then(|s| s.parse().ok());
        Ok(Resp { status, body, remaining, reset })
    }
}

#[cfg(target_arch = "wasm32")]
pub fn now() -> i64 {
    js::now_secs() as i64
}

/// Web only: reads `config.json` from `repohub-server`. Absent (a static host such as GitHub
/// Pages) means the sample data.
#[cfg(target_arch = "wasm32")]
pub async fn fetch_config() -> Option<Config> {
    let v = js::gh_fetch("GET", "config.json", "application/json").await;
    let a = js_sys::Array::from(&v);
    if a.get(0).as_f64() != Some(200.0) {
        return None;
    }
    let body = a.get(1).as_string()?;
    Config::from_json(&body)
}
