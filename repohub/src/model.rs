//! The data the pages show, loaded from GitHub (see `github.rs`) or, without a configuration,
//! the built-in sample set.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PullState {
    Open,
    Draft,
    Merged,
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Checks {
    Passing,
    Failing,
    Running,
}

#[derive(Clone, Debug, Default)]
pub struct Repo {
    pub full_name: String,
    pub owner: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub hue: f32,
    pub stars: u32,
    pub forks: u32,
    pub issues: u32,
    /// "aggiornato 2 ore fa" text, computed when loaded.
    pub updated: String,
    pub private: bool,
    pub fork: bool,
    pub archived: bool,
    pub branch: String,
    pub topics: Vec<String>,
    pub url: String,
    pub starred: bool,
}

#[derive(Clone, Debug)]
pub struct Pull {
    pub number: u32,
    pub title: String,
    pub repo: String,
    pub author: String,
    pub state: PullState,
    pub checks: Option<Checks>,
    pub age: String,
    pub comments: u32,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct Issue {
    pub number: u32,
    pub title: String,
    pub repo: String,
    /// First label and its hue.
    pub label: Option<(String, f32)>,
    pub age: String,
    pub comments: u32,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct Run {
    pub workflow: String,
    pub repo: String,
    pub branch: String,
    pub checks: Checks,
    pub duration: String,
    pub age: String,
    pub url: String,
}

#[derive(Clone, Debug, Default)]
pub struct Profile {
    pub login: String,
    pub name: String,
    pub bio: String,
}

#[derive(Clone, Debug)]
pub struct Commit {
    pub author: String,
    pub message: String,
    pub sha: String,
    pub age: String,
}

/// What a repository's Code tab shows; loaded when the repository is opened.
#[derive(Clone, Debug, Default)]
pub struct Detail {
    pub loading: bool,
    pub error: Option<String>,
    /// (name, is directory)
    pub files: Vec<(String, bool)>,
    pub readme: Option<String>,
    /// (language, bytes), largest first.
    pub languages: Vec<(String, u64)>,
    pub commit: Option<Commit>,
}

#[derive(Clone, Debug, Default)]
pub struct Data {
    pub profile: Profile,
    pub repos: Vec<Repo>,
    pub pulls: Vec<Pull>,
    pub issues: Vec<Issue>,
    pub runs: Vec<Run>,
    /// Commits per week, oldest first.
    pub activity: Vec<u32>,
    /// True for the built-in sample data.
    pub sample: bool,
}

/// A stable hue for a language name, close to GitHub's colours for the common ones.
pub fn lang_hue(name: &str) -> f32 {
    match name {
        "Rust" => 40.0,
        "JavaScript" => 95.0,
        "TypeScript" => 235.0,
        "Python" => 255.0,
        "HTML" => 30.0,
        "CSS" => 295.0,
        "Shell" => 140.0,
        "Go" => 205.0,
        "Java" => 60.0,
        "Kotlin" => 300.0,
        "Swift" => 35.0,
        "C" => 260.0,
        "C++" => 340.0,
        "C#" => 150.0,
        "PHP" => 275.0,
        "Ruby" => 20.0,
        "Dart" => 200.0,
        "Jupyter Notebook" => 50.0,
        _ => (name.bytes().fold(11u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32)) % 360) as f32,
    }
}

/// The hue of a `rrggbb` label colour.
pub fn hex_hue(hex: &str) -> f32 {
    let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0x888888);
    let (r, g, b) = (((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    if d < 1e-4 {
        return 250.0;
    }
    let h = if max == r { ((g - b) / d).rem_euclid(6.0) } else if max == g { (b - r) / d + 2.0 } else { (r - g) / d + 4.0 };
    h * 60.0
}

/// Seconds since the Unix epoch for an ISO 8601 UTC timestamp (`2026-11-09T09:35:11Z`).
pub fn parse_iso(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<i64>().ok();
    let (y, mo, d, h, mi, se) = (n(0..4)?, n(5..7)?, n(8..10)?, n(11..13)?, n(14..16)?, n(17..19)?);
    // Days from civil (Howard Hinnant).
    let y2 = if mo <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = (mo + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + h * 3600 + mi * 60 + se)
}

/// "3 ore fa", "ieri", "2 settimane fa"… for a timestamp, relative to `now`.
pub fn ago(ts: i64, now: i64) -> String {
    let s = (now - ts).max(0);
    let (m, h, d) = (s / 60, s / 3600, s / 86400);
    match () {
        _ if s < 60 => "adesso".into(),
        _ if m < 60 => if m == 1 { "1 minuto fa".into() } else { format!("{m} minuti fa") },
        _ if h < 24 => if h == 1 { "1 ora fa".into() } else { format!("{h} ore fa") },
        _ if d == 1 => "ieri".into(),
        _ if d < 7 => format!("{d} giorni fa"),
        _ if d < 30 => if d / 7 == 1 { "1 settimana fa".into() } else { format!("{} settimane fa", d / 7) },
        _ if d < 365 => if d / 30 == 1 { "1 mese fa".into() } else { format!("{} mesi fa", d / 30) },
        _ => if d / 365 == 1 { "1 anno fa".into() } else { format!("{} anni fa", d / 365) },
    }
}

/// "41s", "3m 05s", "1h 02m".
pub fn duration(secs: i64) -> String {
    let s = secs.max(0);
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{}h {:02}m", s / 3600, (s % 3600) / 60)
    }
}

/// Sample files for the sample repository's Code tab.
pub fn sample_detail(r: &Repo) -> Detail {
    let files = [(".github", true), ("bunny", true), ("css", true), ("js", true), ("repohub", true), ("README.md", false), ("index.html", false)];
    Detail {
        loading: false,
        error: None,
        files: files.iter().map(|(n, d)| (n.to_string(), *d)).collect(),
        readme: Some(format!("# {}\n\n{}\n\n## Avvio rapido\n\ngit clone {}.git", r.name, r.description, r.url)),
        languages: vec![(r.language.clone(), 70), ("HTML".into(), 20), ("CSS".into(), 10)],
        commit: Some(Commit { author: r.owner.clone(), message: "Add RepoHub, a Rune-based GitHub repository manager UI".into(), sha: "a6f6543".into(), age: "2 ore fa".into() }),
    }
}

impl Data {
    /// Built-in sample data, shown when no GitHub configuration is present. Everything is made up.
    pub fn sample() -> Data {
        let repo = |name: &str, desc: &str, lang: &str, stars, forks, issues, updated: &str, private, fork, branch: &str, topics: &[&str], starred| Repo {
            full_name: format!("francescodibella/{name}"),
            owner: "francescodibella".into(),
            name: name.into(),
            description: desc.into(),
            language: lang.into(),
            hue: lang_hue(lang),
            stars,
            forks,
            issues,
            updated: updated.into(),
            private,
            fork,
            archived: false,
            branch: branch.into(),
            topics: topics.iter().map(|t| t.to_string()).collect(),
            url: format!("https://github.com/francescodibella/{name}"),
            starred,
        };
        let repos = vec![
            repo("francescodibella.github.io", "Personal site and playground: shell UI, widgets and a procedural Three.js bunny.", "JavaScript", 12, 2, 3, "2 ore fa", false, false, "main", &["github-pages", "threejs", "portfolio"], false),
            repo("ngsi-ld-xr-contexts", "JSON-LD contexts for AR, VR and MR entities in NGSI-LD smart data models.", "JSON-LD", 41, 9, 5, "ieri", false, false, "main", &["fiware", "ngsi-ld", "xr"], true),
            repo("xr-digital-twin-thesis", "Master thesis: a digital twin of an industrial cell streamed to a WebXR headset.", "TypeScript", 7, 0, 11, "3 giorni fa", true, false, "dev", &["webxr", "digital-twin"], false),
            repo("dotfiles", "Shell, editor and terminal configuration, bootstrapped with one script.", "Shell", 3, 1, 0, "settimana scorsa", false, false, "main", &["zsh", "neovim"], false),
            repo("rune", "A native GPU-rendered Rust UI framework for macOS, Windows, Linux and the web.", "Rust", 0, 0, 0, "2 settimane fa", false, true, "dev", &["rust", "gpu", "ui"], false),
            repo("iot-sensor-gateway", "MQTT to NGSI-LD bridge for low-power environmental sensors.", "Python", 18, 4, 2, "il mese scorso", true, false, "main", &["mqtt", "iot", "fiware"], false),
            repo("university-notes", "Lecture notes in Markdown, compiled to a searchable static site.", "Markdown", 25, 6, 1, "2 mesi fa", false, false, "main", &["notes", "mkdocs"], false),
        ];
        let pull = |number, title: &str, repo: &str, author: &str, state, checks, age: &str, comments| Pull {
            number, title: title.into(), repo: repo.into(), author: author.into(), state, checks: Some(checks), age: age.into(), comments,
            url: format!("https://github.com/francescodibella/{repo}/pull/{number}"),
        };
        let pulls = vec![
            pull(14, "Add procedural Three.js Blue Bunny model", "francescodibella.github.io", "claude", PullState::Open, Checks::Passing, "2 ore fa", 3),
            pull(9, "Mixed reality context v0.2: anchors and spatial meshes", "ngsi-ld-xr-contexts", "francescodibella", PullState::Draft, Checks::Running, "ieri", 1),
            pull(8, "Fix @vocab for VR entity relationships", "ngsi-ld-xr-contexts", "contributor42", PullState::Open, Checks::Failing, "4 giorni fa", 6),
            pull(3, "Repo manager example on the web target", "rune", "francescodibella", PullState::Open, Checks::Passing, "2 settimane fa", 0),
            pull(21, "Chapter 4: latency measurements", "xr-digital-twin-thesis", "francescodibella", PullState::Merged, Checks::Passing, "3 settimane fa", 12),
        ];
        let issue = |number, title: &str, repo: &str, label: &str, hue, age: &str| Issue {
            number, title: title.into(), repo: repo.into(), label: Some((label.into(), hue)), age: age.into(), comments: 0,
            url: format!("https://github.com/francescodibella/{repo}/issues/{number}"),
        };
        let issues = vec![
            issue(31, "Bunny poncho clips through lowered arms", "francescodibella.github.io", "bug", 25.0, "1 ora fa"),
            issue(12, "Document the MR context with examples", "ngsi-ld-xr-contexts", "docs", 240.0, "2 giorni fa"),
            issue(44, "Measure headset frame pacing over Wi-Fi 6", "xr-digital-twin-thesis", "research", 300.0, "5 giorni fa"),
            issue(7, "Battery drain on the CO2 sensor node", "iot-sensor-gateway", "bug", 25.0, "1 settimana fa"),
        ];
        let run = |workflow: &str, repo: &str, branch: &str, checks, duration: &str, age: &str| Run {
            workflow: workflow.into(), repo: repo.into(), branch: branch.into(), checks, duration: duration.into(), age: age.into(),
            url: format!("https://github.com/francescodibella/{repo}/actions"),
        };
        let runs = vec![
            run("pages-build-deployment", "francescodibella.github.io", "main", Checks::Passing, "41s", "2 ore fa"),
            run("validate-jsonld", "ngsi-ld-xr-contexts", "mr-v0.2", Checks::Running, "1m 12s", "in corso"),
            run("validate-jsonld", "ngsi-ld-xr-contexts", "fix-vocab", Checks::Failing, "38s", "4 giorni fa"),
            run("latex-build", "xr-digital-twin-thesis", "dev", Checks::Passing, "3m 05s", "3 giorni fa"),
            run("pytest", "iot-sensor-gateway", "main", Checks::Passing, "57s", "il mese scorso"),
        ];
        Data {
            profile: Profile { login: "francescodibella".into(), name: "Francesco Di Bella".into(), bio: "XR & IoT developer".into() },
            repos,
            pulls,
            issues,
            runs,
            activity: vec![2, 4, 1, 0, 3, 6, 8, 5, 2, 0, 1, 4, 7, 9, 6, 3, 2, 5, 8, 12, 10, 6, 4, 7, 11, 9],
            sample: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_and_ago() {
        assert_eq!(parse_iso("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_iso("2026-11-09T09:35:11Z"), Some(1_794_216_911));
        let t = parse_iso("2026-11-09T09:35:11Z").unwrap();
        assert_eq!(ago(t - 30, t), "adesso");
        assert_eq!(ago(t - 7200, t), "2 ore fa");
        assert_eq!(ago(t - 86400 - 10, t), "ieri");
        assert_eq!(ago(t - 15 * 86400, t), "2 settimane fa");
        assert_eq!(duration(185), "3m 05s");
    }

    #[test]
    fn label_hue() {
        assert!((hex_hue("d73a4a") - 354.0).abs() < 2.0);
        assert!((hex_hue("0075ca") - 205.0).abs() < 2.0);
    }
}
