//! Settings read from the environment and from a `.env` file (see `.env.example`).
//!
//! The token is only ever sent to the configured API host; it is never shown in the UI or logged.

#[derive(Clone)]
pub struct Config {
    /// Personal access token (read-only is enough; "Starring" write to star from the app).
    pub token: Option<String>,
    /// Whose repositories to show; defaults to the token's owner.
    pub user: Option<String>,
    /// REST API base, `https://api.github.com` or a GitHub Enterprise `https://host/api/v3`.
    pub api: String,
    pub title: String,
    /// `Some(true)` dark, `Some(false)` light, `None` follows the system.
    pub theme: Option<bool>,
    /// Repository names (or `owner/name`) shown under "Fissati".
    pub pinned: Vec<String>,
    pub include_forks: bool,
    pub include_archived: bool,
    pub max_repos: usize,
    /// How many recently pushed repositories to read workflow runs and commit activity from.
    pub activity_repos: usize,
    /// Force the sample data even when credentials are present.
    pub demo: bool,
    /// The `.env` file that was loaded, if any.
    pub env_file: Option<String>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("token", &self.token.as_ref().map(|_| "<set>"))
            .field("user", &self.user)
            .field("api", &self.api)
            .field("env_file", &self.env_file)
            .finish_non_exhaustive()
    }
}

fn flag(v: Option<String>, default: bool) -> bool {
    match v.as_deref().map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("1" | "true" | "yes" | "si" | "on") => true,
        Some("0" | "false" | "no" | "off") => false,
        _ => default,
    }
}

/// Finds the `.env` file: `REPOHUB_ENV`, else `.env` in the current folder or one of its parents.
#[cfg(not(target_arch = "wasm32"))]
fn find_env_file() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("REPOHUB_ENV") {
        return Some(p.into());
    }
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..4 {
        let candidate = dir.join(".env");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Parses `KEY=value` lines (optional `export`, quotes and `#` comments) into the process
/// environment. Variables already set in the environment are kept.
#[cfg(not(target_arch = "wasm32"))]
fn load_env_file() -> Option<String> {
    let path = find_env_file()?;
    let src = std::fs::read_to_string(&path).ok()?;
    for (k, v) in parse_env(&src) {
        if std::env::var_os(&k).is_none() {
            // SAFETY: called once at startup, before any other thread exists.
            unsafe { std::env::set_var(k, v) };
        }
    }
    Some(path.display().to_string())
}

/// `KEY=value` pairs from a `.env` file's text.
pub fn parse_env(src: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((k, v)) = line.split_once('=') else { continue };
        let k = k.trim();
        if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let v = v.trim();
        let v = if let Some(q) = v.chars().next().filter(|c| *c == '"' || *c == '\'') {
            v[1..].split(q).next().unwrap_or("").to_owned()
        } else {
            // An unquoted value ends at " #" (a trailing comment).
            v.split(" #").next().unwrap_or("").trim().to_owned()
        };
        out.push((k.to_owned(), v));
    }
    out
}

impl Config {
    /// Reads `.env` (from `REPOHUB_ENV`, else the current folder or one of its parents), then the
    /// process environment, which wins over the file.
    pub fn load() -> Config {
        #[cfg(not(target_arch = "wasm32"))]
        let env_file = load_env_file();
        #[cfg(target_arch = "wasm32")]
        let env_file: Option<String> = None;
        let get = |k: &str| -> Option<String> {
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::env::var(k).ok().map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = k;
                None
            }
        };
        let num = |k: &str, d: usize| get(k).and_then(|v| v.parse().ok()).unwrap_or(d);
        Config {
            token: get("GITHUB_TOKEN"),
            user: get("GITHUB_USER"),
            api: get("GITHUB_API_URL").unwrap_or_else(|| "https://api.github.com".into()).trim_end_matches('/').to_owned(),
            title: get("REPOHUB_TITLE").unwrap_or_else(|| "RepoHub".into()),
            theme: match get("REPOHUB_THEME").as_deref() {
                Some("dark") => Some(true),
                Some("light") => Some(false),
                _ => None,
            },
            pinned: get("REPOHUB_PINNED").map(|v| v.split(',').map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect()).unwrap_or_default(),
            include_forks: flag(get("REPOHUB_INCLUDE_FORKS"), true),
            include_archived: flag(get("REPOHUB_INCLUDE_ARCHIVED"), false),
            max_repos: num("REPOHUB_MAX_REPOS", 100).clamp(1, 1000),
            activity_repos: num("REPOHUB_ACTIVITY_REPOS", 5).clamp(0, 20),
            demo: flag(get("REPOHUB_DEMO"), false),
            env_file,
        }
    }

    /// True when the app should read from GitHub instead of showing the sample data.
    pub fn live(&self) -> bool {
        !self.demo && (self.token.is_some() || self.user.is_some())
    }

    /// The web build's configuration, as `repohub-server` publishes it in `config.json`. It never
    /// contains the token, only whether the server has one (`hasToken`).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub fn from_json(src: &str) -> Option<Config> {
        let v: serde_json::Value = serde_json::from_str(src).ok()?;
        let st = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_owned).filter(|x| !x.is_empty());
        let bo = |k: &str, d: bool| v.get(k).and_then(|x| x.as_bool()).unwrap_or(d);
        let nu = |k: &str, d: usize| v.get(k).and_then(|x| x.as_u64()).map(|x| x as usize).unwrap_or(d);
        Some(Config {
            // A marker, not a secret: the server adds the real token to proxied requests.
            token: bo("hasToken", false).then(|| "server".to_owned()),
            user: st("user"),
            api: st("api").unwrap_or_else(|| "https://api.github.com".into()),
            title: st("title").unwrap_or_else(|| "RepoHub".into()),
            theme: match st("theme").as_deref() {
                Some("dark") => Some(true),
                Some("light") => Some(false),
                _ => None,
            },
            pinned: v.get("pinned").and_then(|p| p.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default(),
            include_forks: bo("includeForks", true),
            include_archived: bo("includeArchived", false),
            max_repos: nu("maxRepos", 100).clamp(1, 1000),
            activity_repos: nu("activityRepos", 5).clamp(0, 20),
            demo: bo("demo", false),
            env_file: Some("gestito da repohub-server (Docker)".into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, parse_env};

    #[test]
    fn web_config() {
        let c = Config::from_json(r#"{"user":"octocat","hasToken":true,"pinned":["a","b"],"theme":"dark","maxRepos":20}"#).unwrap();
        assert_eq!(c.user.as_deref(), Some("octocat"));
        assert!(c.live() && c.token.is_some());
        assert_eq!(c.pinned, vec!["a", "b"]);
        assert_eq!((c.theme, c.max_repos), (Some(true), 20));
        assert!(!Config::from_json(r#"{"hasToken":false}"#).unwrap().live());
    }

    #[test]
    fn env_lines() {
        let src = "# comment\nGITHUB_USER=octocat\nREPOHUB_TITLE=RepoHub di Francesco  # titolo\nexport GITHUB_TOKEN=\"ghp abc\"\nEMPTY=\nREPOHUB_PINNED='a, b'\nbad line\n";
        let v = parse_env(src);
        assert_eq!(v, vec![
            ("GITHUB_USER".into(), "octocat".into()),
            ("REPOHUB_TITLE".into(), "RepoHub di Francesco".into()),
            ("GITHUB_TOKEN".into(), "ghp abc".into()),
            ("EMPTY".into(), "".into()),
            ("REPOHUB_PINNED".into(), "a, b".into()),
        ]);
    }
}
