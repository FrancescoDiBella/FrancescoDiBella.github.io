//! repohub-server: serves the RepoHub web build and proxies the GitHub API for it.
//!
//! - `GET /` and static files from `WEB_DIR`.
//! - `GET /config.json`: the non-secret settings (user, title, theme, pinned…) and `hasToken`.
//! - `/gh/<path>`: forwarded to `GITHUB_API_URL/<path>` with the token added, so the token stays
//!   in this process and never reaches the browser. Only the endpoints RepoHub uses are allowed,
//!   only `PUT`/`DELETE` on `/user/starred/…` may write, and requests from other origins or with
//!   a foreign `Host` (DNS rebinding) are refused.
//!
//! Configuration comes from the environment (`docker run --env-file .env`, or `env_file` in
//! compose.yaml), with the same variables as the desktop app's `.env`.
use std::io::Read;
use std::path::{Path, PathBuf};

use tiny_http::{Header, Method, Request, Response, Server};

struct Cfg {
    token: Option<String>,
    api: String,
    web_dir: PathBuf,
    allowed_hosts: Vec<String>,
    public_json: String,
    agent: ureq::Agent,
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok().map(|v| v.trim().trim_matches('"').trim_matches('\'').to_owned()).filter(|v| !v.is_empty())
}

fn flag(k: &str, d: bool) -> bool {
    match env(k).map(|v| v.to_ascii_lowercase()).as_deref() {
        Some("1" | "true" | "yes" | "si" | "on") => true,
        Some("0" | "false" | "no" | "off") => false,
        _ => d,
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl Cfg {
    fn load() -> Cfg {
        let token = env("GITHUB_TOKEN");
        let api = env("GITHUB_API_URL").unwrap_or_else(|| "https://api.github.com".into()).trim_end_matches('/').to_owned();
        let opt = |k: &str| env(k).map(|v| json_str(&v)).unwrap_or_else(|| "null".into());
        let pinned: Vec<String> = env("REPOHUB_PINNED").map(|v| v.split(',').map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect()).unwrap_or_default();
        let num = |k: &str, d: u64| env(k).and_then(|v| v.parse::<u64>().ok()).unwrap_or(d);
        let public_json = format!(
            "{{\"user\":{},\"hasToken\":{},\"api\":{},\"title\":{},\"theme\":{},\"pinned\":[{}],\"includeForks\":{},\"includeArchived\":{},\"maxRepos\":{},\"activityRepos\":{},\"demo\":{}}}",
            opt("GITHUB_USER"),
            token.is_some(),
            json_str(&api),
            opt("REPOHUB_TITLE"),
            opt("REPOHUB_THEME"),
            pinned.iter().map(|p| json_str(p)).collect::<Vec<_>>().join(","),
            flag("REPOHUB_INCLUDE_FORKS", true),
            flag("REPOHUB_INCLUDE_ARCHIVED", false),
            num("REPOHUB_MAX_REPOS", 100),
            num("REPOHUB_ACTIVITY_REPOS", 5),
            flag("REPOHUB_DEMO", false),
        );
        let mut allowed_hosts: Vec<String> = ["localhost", "127.0.0.1", "[::1]"].iter().map(|s| s.to_string()).collect();
        if let Some(extra) = env("REPOHUB_ALLOWED_HOSTS") {
            allowed_hosts.extend(extra.split(',').map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty()));
        }
        let agent = ureq::AgentBuilder::new()
            .try_proxy_from_env(!bypass_proxy(&api))
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("RepoHub server")
            .build();
        Cfg {
            token,
            api,
            web_dir: env("WEB_DIR").map(PathBuf::from).unwrap_or_else(|| "web".into()),
            allowed_hosts,
            public_json,
            agent,
        }
    }
}

/// True when NO_PROXY (or a loopback host) says to reach `api` directly.
fn bypass_proxy(api: &str) -> bool {
    let host = api.split("://").nth(1).unwrap_or(api).split(['/', ':']).next().unwrap_or("").to_ascii_lowercase();
    if host == "localhost" || host.starts_with("127.") || host == "[::1]" {
        return true;
    }
    let list = env("NO_PROXY").or_else(|| env("no_proxy")).unwrap_or_default();
    list.split(',').map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty())
        .any(|e| e == "*" || host == e || host.ends_with(&format!(".{e}")))
}

fn header(r: &Request, name: &str) -> Option<String> {
    r.headers().iter().find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name)).map(|h| h.value.as_str().to_owned())
}

fn host_only(h: &str) -> String {
    let h = h.trim().to_ascii_lowercase();
    if h.starts_with('[') {
        return h.split(']').next().map(|x| format!("{x}]")).unwrap_or(h);
    }
    h.split(':').next().unwrap_or("").to_owned()
}

/// Which proxied API calls are allowed: the read endpoints RepoHub uses, and starring.
fn allowed(method: &Method, path: &str) -> bool {
    let p = path.split('?').next().unwrap_or("");
    match method {
        Method::Get => {
            p == "/user"
                || p == "/user/repos"
                || p == "/user/starred"
                || p == "/search/issues"
                || p.starts_with("/users/")
                || (p.starts_with("/repos/") && !p.contains(".."))
        }
        Method::Put | Method::Delete => p.starts_with("/user/starred/") && p.matches('/').count() == 4,
        _ => false,
    }
}

fn mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "json" => "application/json",
        "css" => "text/css",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn h(k: &str, v: &str) -> Header {
    Header::from_bytes(k.as_bytes(), v.as_bytes()).expect("valid header")
}

fn text(code: u16, body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_status_code(code).with_header(h("Content-Type", "text/plain; charset=utf-8"))
}

fn serve_static(cfg: &Cfg, req: Request) {
    let raw = req.url().split('?').next().unwrap_or("/").to_owned();
    let rel = raw.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    if rel.split('/').any(|seg| seg == ".." || seg.starts_with('.')) {
        let _ = req.respond(text(404, "Not found"));
        return;
    }
    let path = cfg.web_dir.join(rel);
    match std::fs::read(&path) {
        Ok(bytes) => {
            let resp = Response::from_data(bytes).with_header(h("Content-Type", mime(&path))).with_header(h("Cache-Control", "no-cache"));
            let _ = req.respond(resp);
        }
        Err(_) => {
            let _ = req.respond(text(404, "Not found"));
        }
    }
}

fn proxy(cfg: &Cfg, mut req: Request) {
    let path = req.url()["/gh".len()..].to_owned();
    if !allowed(req.method(), &path) {
        let _ = req.respond(text(403, "Endpoint non consentito dal proxy di RepoHub"));
        return;
    }
    let accept = header(&req, "Accept").unwrap_or_else(|| "application/vnd.github+json".into());
    let method = req.method().as_str().to_owned();
    let mut body = Vec::new();
    let _ = req.as_reader().take(1 << 16).read_to_end(&mut body);
    let mut r = cfg.agent.request(&method, &format!("{}{}", cfg.api, path))
        .set("Accept", &accept)
        .set("X-GitHub-Api-Version", "2022-11-28");
    if let Some(t) = &cfg.token {
        r = r.set("Authorization", &format!("Bearer {t}"));
    }
    let result = if method == "PUT" { r.set("Content-Length", "0").send_string("") } else { r.call() };
    let resp = match result {
        Ok(resp) | Err(ureq::Error::Status(_, resp)) => resp,
        Err(e) => {
            let _ = req.respond(text(502, &format!("GitHub non raggiungibile: {e}")));
            return;
        }
    };
    let status = resp.status();
    let mut headers = vec![h("Cache-Control", "no-store")];
    for name in ["Content-Type", "x-ratelimit-remaining", "x-ratelimit-reset", "x-ratelimit-limit"] {
        if let Some(v) = resp.header(name) {
            headers.push(h(name, v));
        }
    }
    let mut data = Vec::new();
    let _ = resp.into_reader().take(32 << 20).read_to_end(&mut data);
    let mut out = Response::from_data(data).with_status_code(status);
    for x in headers {
        out.add_header(x);
    }
    let _ = req.respond(out);
}

fn handle(cfg: &Cfg, req: Request) {
    // Refuse a foreign Host (DNS rebinding) and cross-site callers, so no other page can use the token.
    let host = header(&req, "Host").map(|h| host_only(&h)).unwrap_or_default();
    if !cfg.allowed_hosts.iter().any(|a| *a == host) {
        let _ = req.respond(text(403, "Host non consentito: aggiungilo a REPOHUB_ALLOWED_HOSTS"));
        return;
    }
    if let Some(origin) = header(&req, "Origin") {
        let o = host_only(origin.split("://").nth(1).unwrap_or(""));
        if o != host {
            let _ = req.respond(text(403, "Origine non consentita"));
            return;
        }
    }
    let url = req.url().to_owned();
    if url.starts_with("/gh/") {
        proxy(cfg, req);
    } else if url.split('?').next() == Some("/config.json") {
        let resp = Response::from_string(cfg.public_json.clone()).with_header(h("Content-Type", "application/json")).with_header(h("Cache-Control", "no-store"));
        let _ = req.respond(resp);
    } else if *req.method() == Method::Get || *req.method() == Method::Head {
        serve_static(cfg, req);
    } else {
        let _ = req.respond(text(405, "Metodo non consentito"));
    }
}

fn main() {
    let cfg = std::sync::Arc::new(Cfg::load());
    let addr = env("REPOHUB_ADDR").unwrap_or_else(|| "127.0.0.1:8080".into());
    let server = Server::http(&addr).unwrap_or_else(|e| {
        eprintln!("repohub-server: non riesco ad ascoltare su {addr}: {e}");
        std::process::exit(1);
    });
    let port = addr.rsplit(':').next().unwrap_or("8080");
    eprintln!("repohub-server: in ascolto su {addr}. Apri http://localhost:{port}");
    eprintln!(
        "repohub-server: GitHub {} · token {} · file statici da {}",
        cfg.api,
        if cfg.token.is_some() { "configurato" } else { "assente (solo dati pubblici)" },
        cfg.web_dir.display()
    );
    let server = std::sync::Arc::new(server);
    let workers: Vec<_> = (0..8).map(|_| {
        let (server, cfg) = (server.clone(), cfg.clone());
        std::thread::spawn(move || {
            for req in server.incoming_requests() {
                handle(&cfg, req);
            }
        })
    }).collect();
    for w in workers {
        let _ = w.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_allowlist() {
        assert!(allowed(&Method::Get, "/user/repos?per_page=100"));
        assert!(allowed(&Method::Get, "/repos/o/r/contents"));
        assert!(allowed(&Method::Get, "/search/issues?q=is%3Apr"));
        assert!(allowed(&Method::Put, "/user/starred/o/r"));
        assert!(!allowed(&Method::Delete, "/repos/o/r"));
        assert!(!allowed(&Method::Post, "/user/repos"));
        assert!(!allowed(&Method::Get, "/orgs/x/members"));
        assert!(!allowed(&Method::Put, "/user/starred/o/r/extra"));
    }

    #[test]
    fn hosts() {
        assert_eq!(host_only("localhost:8080"), "localhost");
        assert_eq!(host_only("[::1]:8080"), "[::1]");
        assert_eq!(json_str("a\"b"), "\"a\\\"b\"");
    }
}
