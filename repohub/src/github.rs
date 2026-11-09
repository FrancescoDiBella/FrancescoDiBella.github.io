//! GitHub REST API client, shared by the desktop and web builds. Requests go through
//! [`net::Transport`]; the functions here are async and run in the background (see `net::spawn`).
use std::collections::HashSet;

use serde_json::Value;

use crate::config::Config;
use crate::net::{self, Resp, Transport};
use crate::model::{Checks, Commit, Data, Detail, Issue, Profile, Pull, PullState, Repo, Run, ago, duration, hex_hue, lang_hue, parse_iso};

pub struct Gh {
    t: Transport,
    now: i64,
}

const JSON: &str = "application/vnd.github+json";

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
        Gh { t: Transport::new(cfg), now: net::now() }
    }

    fn explain(&self, r: &Resp) -> String {
        let msg = serde_json::from_str::<Value>(&r.body).ok().map(|v| s(&v, "message")).unwrap_or_default();
        match r.status {
            401 => "GitHub ha rifiutato il token (401): controlla GITHUB_TOKEN nel file .env.".into(),
            403 | 429 if r.remaining.as_deref() == Some("0") => {
                let when = r.reset.map(|t| format!(" Riprova tra {} minuti.", ((t - self.now) / 60).max(1))).unwrap_or_default();
                format!("Limite di richieste API raggiunto.{when} Con GITHUB_TOKEN il limite sale a 5000 richieste l'ora.")
            }
            404 => format!("Non trovato (404): {msg}. Controlla GITHUB_USER o i permessi del token."),
            code if msg.is_empty() => format!("Errore GitHub {code} dall'API."),
            code => format!("Errore GitHub {code}: {msg}"),
        }
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        let r = self.t.send("GET", path, JSON).await?;
        match r.status {
            202 | 204 => Ok(Value::Null),
            200..=299 => serde_json::from_str(&r.body).map_err(|e| format!("Risposta non valida da GitHub: {e}")),
            _ => Err(self.explain(&r)),
        }
    }

    async fn get_raw(&self, path: &str) -> Result<Option<String>, String> {
        let r = self.t.send("GET", path, "application/vnd.github.raw+json").await?;
        match r.status {
            200..=299 => Ok(Some(r.body)),
            404 => Ok(None),
            _ => Err(self.explain(&r)),
        }
    }

    fn ago(&self, iso: &str) -> String {
        parse_iso(iso).map(|t| ago(t, self.now)).unwrap_or_default()
    }

    pub async fn profile(&self, cfg: &Config) -> Result<Profile, String> {
        let v = match (&cfg.user, self.t.has_token()) {
            (Some(u), _) => self.get(&format!("/users/{u}")).await?,
            (None, true) => self.get("/user").await?,
            (None, false) => return Err("Imposta GITHUB_USER o GITHUB_TOKEN nel file .env.".into()),
        };
        let login = s(&v, "login");
        let name = Some(s(&v, "name")).filter(|x| !x.is_empty()).unwrap_or_else(|| login.clone());
        Ok(Profile { login, name, bio: s(&v, "bio") })
    }

    async fn viewer_login(&self) -> Option<String> {
        if !self.t.has_token() {
            return None;
        }
        self.get("/user").await.ok().map(|v| s(&v, "login"))
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

    pub async fn repos(&self, cfg: &Config, login: &str, own: bool) -> Result<Vec<Repo>, String> {
        let starred: HashSet<String> = if self.t.has_token() {
            self.get("/user/starred?per_page=100").await.ok().and_then(|v| v.as_array().cloned()).unwrap_or_default()
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
            let v = self.get(&path).await?;
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

    async fn search(&self, q: &str) -> Vec<Value> {
        let q = q.replace(' ', "+").replace(':', "%3A");
        self.get(&format!("/search/issues?q={q}&sort=updated&order=desc&per_page=40")).await
            .ok().and_then(|v| v.get("items").and_then(Value::as_array).cloned()).unwrap_or_default()
    }

    pub async fn pulls(&self, login: &str) -> Vec<Pull> {
        self.search(&format!("is:pr involves:{login}")).await.iter().map(|v| {
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

    pub async fn issues(&self, login: &str) -> Vec<Issue> {
        self.search(&format!("is:issue is:open archived:false user:{login}")).await.iter().map(|v| Issue {
            number: n(v, "number"),
            title: s(v, "title"),
            repo: repo_from_url(&s(v, "repository_url")),
            label: v.get("labels").and_then(Value::as_array).and_then(|l| l.first()).map(|l| (s(l, "name"), hex_hue(&s(l, "color")))),
            age: self.ago(&s(v, "created_at")),
            comments: n(v, "comments"),
            url: s(v, "html_url"),
        }).collect()
    }

    pub async fn runs(&self, full: &str) -> Vec<Run> {
        let v = self.get(&format!("/repos/{full}/actions/runs?per_page=4")).await.unwrap_or(Value::Null);
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
    pub async fn participation(&self, full: &str) -> Vec<u32> {
        let v = self.get(&format!("/repos/{full}/stats/participation")).await.unwrap_or(Value::Null);
        let weeks: Vec<u32> = v.get("owner").and_then(Value::as_array).map(|a| a.iter().map(|x| x.as_u64().unwrap_or(0) as u32).collect()).unwrap_or_default();
        if weeks.len() >= 26 { weeks[weeks.len() - 26..].to_vec() } else { vec![0; 26] }
    }

    pub async fn detail(&self, full: &str) -> Detail {
        let mut d = Detail::default();
        match self.get(&format!("/repos/{full}/contents")).await {
            Ok(v) => {
                let mut files: Vec<(String, bool)> = v.as_array().cloned().unwrap_or_default().iter().map(|f| (s(f, "name"), s(f, "type") == "dir")).collect();
                files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase())));
                d.files = files;
            }
            Err(e) => d.error = Some(e),
        }
        d.readme = self.get_raw(&format!("/repos/{full}/readme")).await.ok().flatten().map(|t| t.chars().take(4000).collect());
        if let Ok(v) = self.get(&format!("/repos/{full}/languages")).await {
            let mut langs: Vec<(String, u64)> = v.as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0))).collect()).unwrap_or_default();
            langs.sort_by(|a, b| b.1.cmp(&a.1));
            d.languages = langs;
        }
        if let Ok(v) = self.get(&format!("/repos/{full}/commits?per_page=1")).await
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

    pub async fn set_star(&self, full: &str, on: bool) -> Result<(), String> {
        let r = self.t.send(if on { "PUT" } else { "DELETE" }, &format!("/user/starred/{full}"), JSON).await?;
        if (200..300).contains(&r.status) { Ok(()) } else { Err(self.explain(&r)) }
    }
}

/// Everything the overview and lists need, in one go (about 4 + 2 × `activity_repos` requests).
pub async fn load_all(cfg: &Config) -> Result<Data, String> {
    let gh = Gh::new(cfg);
    let profile = gh.profile(cfg).await?;
    let own = gh.t.has_token() && match &cfg.user {
        None => true,
        Some(u) => gh.viewer_login().await.is_some_and(|v| v.eq_ignore_ascii_case(u)),
    };
    let repos = gh.repos(cfg, &profile.login, own).await?;
    let pulls = gh.pulls(&profile.login).await;
    let issues = gh.issues(&profile.login).await;
    let active: Vec<String> = repos.iter().filter(|r| !r.archived).take(cfg.activity_repos).map(|r| r.full_name.clone()).collect();
    let mut runs: Vec<Run> = Vec::new();
    let mut activity = vec![0u32; 26];
    for full in &active {
        runs.extend(gh.runs(full).await);
        for (i, w) in gh.participation(full).await.iter().enumerate() {
            activity[i] += w;
        }
    }
    runs.truncate(20);
    Ok(Data { profile, repos, pulls, issues, runs, activity, sample: false })
}

pub async fn load_detail(cfg: &Config, full: &str) -> Detail {
    Gh::new(cfg).detail(full).await
}

pub async fn set_star(cfg: &Config, full: &str, on: bool) -> Result<(), String> {
    Gh::new(cfg).set_star(full, on).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn repo_name_from_url() {
        assert_eq!(super::repo_from_url("https://api.github.com/repos/octo/hello"), "octo/hello");
    }
}
