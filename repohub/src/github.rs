//! GitHub REST API client (desktop builds only). Runs on worker threads; the UI applies results
//! through `AppHandle::update`.
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::config::Config;
use crate::model::{Checks, Commit, Data, Detail, Issue, Profile, Pull, PullState, Repo, Run, ago, duration, hex_hue, lang_hue, parse_iso};

pub struct Gh {
    agent: ureq::Agent,
    api: String,
    token: Option<String>,
    now: i64,
}

fn s(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_owned()
}

fn n(v: &Value, k: &str) -> u32 {
    v.get(k).and_then(Value::as_u64).unwrap_or(0) as u32
}

fn b(v: &Value, k: &str) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(false)
}

/// `owner/name` from an API URL ending in `/repos/owner/name`.
fn repo_from_url(url: &str) -> String {
    let parts: Vec<&str> = url.rsplitn(3, '/').collect();
    if parts.len() == 3 { format!("{}/{}", parts[1], parts[0]) } else { url.to_owned() }
}

impl Gh {
    pub fn new(cfg: &Config) -> Gh {
        let agent = ureq::AgentBuilder::new()
            .try_proxy_from_env(!bypass_proxy(&cfg.api))
            .timeout(std::time::Duration::from_secs(20))
            .user_agent("RepoHub (Rune)")
            .build();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        Gh { agent, api: cfg.api.clone(), token: cfg.token.clone(), now }
    }

    fn req(&self, method: &str, path: &str) -> ureq::Request {
        let url = if path.starts_with("http") { path.to_owned() } else { format!("{}{}", self.api, path) };
        let mut r = self.agent.request(method, &url)
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28");
        if let Some(t) = &self.token {
            r = r.set("Authorization", &format!("Bearer {t}"));
        }
        r
    }

    fn explain(code: u16, resp: ureq::Response) -> String {
        let remaining = resp.header("x-ratelimit-remaining").map(str::to_owned);
        let reset = resp.header("x-ratelimit-reset").and_then(|v| v.parse::<i64>().ok());
        let msg = resp.into_json::<Value>().ok().map(|v| s(&v, "message")).unwrap_or_default();
        match code {
            401 => "GitHub ha rifiutato il token (401): controlla GITHUB_TOKEN nel file .env.".into(),
            403 | 429 if remaining.as_deref() == Some("0") => {
                let when = reset.map(|r| format!(" Riprova tra {} minuti.", ((r - now_secs()) / 60).max(1))).unwrap_or_default();
                format!("Limite di richieste API raggiunto.{when} Con GITHUB_TOKEN il limite sale a 5000 richieste l'ora.")
            }
            404 => format!("Non trovato (404): {msg}. Controlla GITHUB_USER o i permessi del token."),
            _ if msg.is_empty() => format!("Errore GitHub {code} dall'API."),
            _ => format!("Errore GitHub {code}: {msg}"),
        }
    }

    fn get(&self, path: &str) -> Result<Value, String> {
        match self.req("GET", path).call() {
            Ok(r) if r.status() == 202 || r.status() == 204 => Ok(Value::Null),
            Ok(r) => r.into_json::<Value>().map_err(|e| format!("Risposta non valida da GitHub: {e}")),
            Err(ureq::Error::Status(code, r)) => Err(Self::explain(code, r)),
            Err(e) => Err(format!("Impossibile raggiungere {}: {e}", self.api)),
        }
    }

    fn get_raw(&self, path: &str) -> Result<Option<String>, String> {
        match self.req("GET", path).set("Accept", "application/vnd.github.raw+json").call() {
            Ok(r) => r.into_string().map(Some).map_err(|e| e.to_string()),
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(ureq::Error::Status(code, r)) => Err(Self::explain(code, r)),
            Err(e) => Err(e.to_string()),
        }
    }

    fn ago(&self, iso: &str) -> String {
        parse_iso(iso).map(|t| ago(t, self.now)).unwrap_or_default()
    }

    pub fn profile(&self, cfg: &Config) -> Result<Profile, String> {
        let v = match (&cfg.user, &self.token) {
            (Some(u), _) => self.get(&format!("/users/{u}"))?,
            (None, Some(_)) => self.get("/user")?,
            (None, None) => return Err("Imposta GITHUB_USER o GITHUB_TOKEN nel file .env.".into()),
        };
        let login = s(&v, "login");
        let name = Some(s(&v, "name")).filter(|x| !x.is_empty()).unwrap_or_else(|| login.clone());
        Ok(Profile { login, name, bio: s(&v, "bio") })
    }

    fn viewer_login(&self) -> Option<String> {
        self.token.as_ref()?;
        self.get("/user").ok().map(|v| s(&v, "login"))
    }

    fn repo(&self, v: &Value, starred: &HashSet<String>) -> Repo {
        let full = s(v, "full_name");
        let language = Some(s(v, "language")).filter(|l| !l.is_empty()).unwrap_or_else(|| "—".into());
        Repo {
            owner: v.get("owner").map(|o| s(o, "login")).unwrap_or_default(),
            name: s(v, "name"),
            description: s(v, "description"),
            hue: lang_hue(&language),
            language,
            stars: n(v, "stargazers_count"),
            forks: n(v, "forks_count"),
            issues: n(v, "open_issues_count"),
            updated: self.ago(&s(v, "pushed_at")),
            private: b(v, "private"),
            fork: b(v, "fork"),
            archived: b(v, "archived"),
            branch: s(v, "default_branch"),
            topics: v.get("topics").and_then(Value::as_array).map(|a| a.iter().filter_map(|t| t.as_str().map(str::to_owned)).collect()).unwrap_or_default(),
            url: s(v, "html_url"),
            starred: starred.contains(&full),
            full_name: full,
        }
    }

    pub fn repos(&self, cfg: &Config, login: &str, own: bool) -> Result<Vec<Repo>, String> {
        let starred: HashSet<String> = if self.token.is_some() {
            self.get("/user/starred?per_page=100").ok().and_then(|v| v.as_array().cloned()).unwrap_or_default()
                .iter().map(|r| s(r, "full_name")).collect()
        } else {
            HashSet::new()
        };
        let mut out = Vec::new();
        let mut page = 1;
        while out.len() < cfg.max_repos && page <= 10 {
            // The token's owner also sees private repositories through /user/repos.
            let path = if own {
                format!("/user/repos?affiliation=owner&sort=pushed&per_page=100&page={page}")
            } else {
                format!("/users/{login}/repos?type=owner&sort=pushed&per_page=100&page={page}")
            };
            let v = self.get(&path)?;
            let items = v.as_array().cloned().unwrap_or_default();
            let done = items.len() < 100;
            for r in &items {
                let repo = self.repo(r, &starred);
                if (!repo.fork || cfg.include_forks) && (!repo.archived || cfg.include_archived) {
                    out.push(repo);
                }
            }
            if done {
                break;
            }
            page += 1;
        }
        out.truncate(cfg.max_repos);
        Ok(out)
    }

    fn search(&self, q: &str) -> Vec<Value> {
        let q = q.replace(' ', "+").replace(':', "%3A");
        self.get(&format!("/search/issues?q={q}&sort=updated&order=desc&per_page=40"))
            .ok().and_then(|v| v.get("items").and_then(Value::as_array).cloned()).unwrap_or_default()
    }

    pub fn pulls(&self, login: &str) -> Vec<Pull> {
        self.search(&format!("is:pr involves:{login}")).iter().map(|v| {
            let merged = v.get("pull_request").and_then(|p| p.get("merged_at")).is_some_and(|m| !m.is_null());
            let state = if merged { PullState::Merged } else if s(v, "state") == "closed" { PullState::Closed } else if b(v, "draft") { PullState::Draft } else { PullState::Open };
            Pull {
                number: n(v, "number"),
                title: s(v, "title"),
                repo: repo_from_url(&s(v, "repository_url")),
                author: v.get("user").map(|u| s(u, "login")).unwrap_or_default(),
                state,
                checks: None,
                age: self.ago(&s(v, "updated_at")),
                comments: n(v, "comments"),
                url: s(v, "html_url"),
            }
        }).collect()
    }

    pub fn issues(&self, login: &str) -> Vec<Issue> {
        self.search(&format!("is:issue is:open archived:false user:{login}")).iter().map(|v| Issue {
            number: n(v, "number"),
            title: s(v, "title"),
            repo: repo_from_url(&s(v, "repository_url")),
            label: v.get("labels").and_then(Value::as_array).and_then(|l| l.first()).map(|l| (s(l, "name"), hex_hue(&s(l, "color")))),
            age: self.ago(&s(v, "created_at")),
            comments: n(v, "comments"),
            url: s(v, "html_url"),
        }).collect()
    }

    pub fn runs(&self, full: &str) -> Vec<Run> {
        let v = self.get(&format!("/repos/{full}/actions/runs?per_page=4")).unwrap_or(Value::Null);
        v.get("workflow_runs").and_then(Value::as_array).cloned().unwrap_or_default().iter().map(|r| {
            let checks = match (s(r, "status").as_str(), s(r, "conclusion").as_str()) {
                ("completed", "success" | "skipped" | "neutral") => Checks::Passing,
                ("completed", _) => Checks::Failing,
                _ => Checks::Running,
            };
            let start = parse_iso(&s(r, "run_started_at"));
            let end = parse_iso(&s(r, "updated_at"));
            Run {
                workflow: s(r, "name"),
                repo: full.to_owned(),
                branch: s(r, "head_branch"),
                duration: match (start, end) {
                    (Some(a), Some(b)) if checks != Checks::Running => duration(b - a),
                    (Some(a), _) => duration(self.now - a),
                    _ => String::new(),
                },
                age: if checks == Checks::Running { "in corso".into() } else { self.ago(&s(r, "updated_at")) },
                checks,
                url: s(r, "html_url"),
            }
        }).collect()
    }

    /// The owner's commits per week over the last 26 weeks (zeros while GitHub computes them).
    pub fn participation(&self, full: &str) -> Vec<u32> {
        let v = self.get(&format!("/repos/{full}/stats/participation")).unwrap_or(Value::Null);
        let weeks: Vec<u32> = v.get("owner").and_then(Value::as_array).map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0) as u32).collect()).unwrap_or_default();
        if weeks.len() >= 26 { weeks[weeks.len() - 26..].to_vec() } else { vec![0; 26] }
    }

    pub fn detail(&self, full: &str) -> Detail {
        let mut d = Detail::default();
        match self.get(&format!("/repos/{full}/contents")) {
            Ok(v) => {
                let mut files: Vec<(String, bool)> = v.as_array().cloned().unwrap_or_default().iter().map(|f| (s(f, "name"), s(f, "type") == "dir")).collect();
                files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())));
                d.files = files;
            }
            Err(e) => d.error = Some(e),
        }
        d.readme = self.get_raw(&format!("/repos/{full}/readme")).ok().flatten().map(|t| t.chars().take(4000).collect());
        if let Ok(v) = self.get(&format!("/repos/{full}/languages")) {
            let mut langs: Vec<(String, u64)> = v.as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0))).collect()).unwrap_or_default();
            langs.sort_by(|a, b| b.1.cmp(&a.1));
            d.languages = langs;
        }
        if let Ok(v) = self.get(&format!("/repos/{full}/commits?per_page=1"))
            && let Some(c) = v.as_array().and_then(|a| a.first())
        {
            let commit = c.get("commit").cloned().unwrap_or(Value::Null);
            let author = commit.get("author").cloned().unwrap_or(Value::Null);
            d.commit = Some(Commit {
                author: c.get("author").filter(|a| !a.is_null()).map(|a| s(a, "login")).unwrap_or_else(|| s(&author, "name")),
                message: s(&commit, "message").lines().next().unwrap_or("").to_owned(),
                sha: s(c, "sha").chars().take(7).collect(),
                age: self.ago(&s(&author, "date")),
            });
        }
        d
    }

    pub fn set_star(&self, full: &str, on: bool) -> Result<(), String> {
        let r = self.req(if on { "PUT" } else { "DELETE" }, &format!("/user/starred/{full}")).set("Content-Length", "0");
        let r = if on { r.send_string("") } else { r.call() };
        match r {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(code, resp)) => Err(Self::explain(code, resp)),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// True when NO_PROXY (or a loopback host) says to reach `api` directly. The HTTP client reads
/// HTTPS_PROXY by itself but not NO_PROXY.
fn bypass_proxy(api: &str) -> bool {
    let host = api.split("://").nth(1).unwrap_or(api).split(['/', ':']).next().unwrap_or("").to_ascii_lowercase();
    if host == "localhost" || host.starts_with("127.") || host == "[::1]" {
        return true;
    }
    let list = std::env::var("NO_PROXY").or_else(|_| std::env::var("no_proxy")).unwrap_or_default();
    list.split(',').map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty())
        .any(|e| e == "*" || host == e || host.ends_with(&format!(".{e}")))
}

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Everything the overview and lists need, in one go (about 4 + 2 × `activity_repos` requests).
pub fn load_all(cfg: &Config) -> Result<Data, String> {
    let gh = Gh::new(cfg);
    let profile = gh.profile(cfg)?;
    let own = cfg.token.is_some() && cfg.user.as_ref().is_none_or(|u| gh.viewer_login().is_some_and(|v| v.eq_ignore_ascii_case(u)));
    let repos = gh.repos(cfg, &profile.login, own)?;
    let pulls = gh.pulls(&profile.login);
    let issues = gh.issues(&profile.login);
    let active: Vec<&Repo> = repos.iter().filter(|r| !r.archived).take(cfg.activity_repos).collect();
    let mut runs: Vec<Run> = active.iter().flat_map(|r| gh.runs(&r.full_name)).collect();
    runs.truncate(20);
    let mut activity = vec![0u32; 26];
    for r in &active {
        for (i, w) in gh.participation(&r.full_name).iter().enumerate() {
            activity[i] += w;
        }
    }
    Ok(Data { profile, repos, pulls, issues, runs, activity, sample: false })
}

pub fn load_detail(cfg: &Config, full: &str) -> Detail {
    Gh::new(cfg).detail(full)
}

pub fn set_star(cfg: &Config, full: &str, on: bool) -> Result<(), String> {
    Gh::new(cfg).set_star(full, on)
}

#[cfg(test)]
mod tests {
    #[test]
    fn repo_name_from_url() {
        assert_eq!(super::repo_from_url("https://api.github.com/repos/octo/hello"), "octo/hello");
    }
}
