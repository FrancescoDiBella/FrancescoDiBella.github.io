//! Static sample data: the app has no backend, every number below is made up.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

pub struct Repo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub language: &'static str,
    pub hue: f32,
    pub stars: u32,
    pub forks: u32,
    pub issues: u32,
    pub prs: u32,
    pub updated: &'static str,
    pub visibility: Visibility,
    pub fork: bool,
    pub branch: &'static str,
    pub topics: &'static [&'static str],
}

pub const REPOS: &[Repo] = &[
    Repo { id: "site", name: "francescodibella.github.io", description: "Personal site and playground: shell UI, widgets and a procedural Three.js bunny.", language: "JavaScript", hue: 95.0, stars: 12, forks: 2, issues: 3, prs: 1, updated: "2 ore fa", visibility: Visibility::Public, fork: false, branch: "main", topics: &["github-pages", "threejs", "portfolio"] },
    Repo { id: "ngsi", name: "ngsi-ld-xr-contexts", description: "JSON-LD contexts for AR, VR and MR entities in NGSI-LD smart data models.", language: "JSON-LD", hue: 250.0, stars: 41, forks: 9, issues: 5, prs: 2, updated: "ieri", visibility: Visibility::Public, fork: false, branch: "main", topics: &["fiware", "ngsi-ld", "xr"] },
    Repo { id: "thesis", name: "xr-digital-twin-thesis", description: "Master thesis: a digital twin of an industrial cell streamed to a WebXR headset.", language: "TypeScript", hue: 230.0, stars: 7, forks: 0, issues: 11, prs: 0, updated: "3 giorni fa", visibility: Visibility::Private, fork: false, branch: "dev", topics: &["webxr", "digital-twin"] },
    Repo { id: "dotfiles", name: "dotfiles", description: "Shell, editor and terminal configuration, bootstrapped with one script.", language: "Shell", hue: 140.0, stars: 3, forks: 1, issues: 0, prs: 0, updated: "settimana scorsa", visibility: Visibility::Public, fork: false, branch: "main", topics: &["zsh", "neovim"] },
    Repo { id: "rune", name: "rune", description: "A native GPU-rendered Rust UI framework for macOS, Windows, Linux and the web.", language: "Rust", hue: 40.0, stars: 0, forks: 0, issues: 0, prs: 1, updated: "2 settimane fa", visibility: Visibility::Public, fork: true, branch: "dev", topics: &["rust", "gpu", "ui"] },
    Repo { id: "iot", name: "iot-sensor-gateway", description: "MQTT to NGSI-LD bridge for low-power environmental sensors.", language: "Python", hue: 265.0, stars: 18, forks: 4, issues: 2, prs: 0, updated: "il mese scorso", visibility: Visibility::Private, fork: false, branch: "main", topics: &["mqtt", "iot", "fiware"] },
    Repo { id: "notes", name: "university-notes", description: "Lecture notes in Markdown, compiled to a searchable static site.", language: "Markdown", hue: 300.0, stars: 25, forks: 6, issues: 1, prs: 0, updated: "2 mesi fa", visibility: Visibility::Public, fork: false, branch: "main", topics: &["notes", "mkdocs"] },
];

pub struct Pull {
    pub number: u32,
    pub title: &'static str,
    pub repo: &'static str,
    pub author: &'static str,
    pub state: PullState,
    pub checks: Checks,
    pub age: &'static str,
    pub comments: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PullState {
    Open,
    Draft,
    Merged,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Checks {
    Passing,
    Failing,
    Running,
}

pub const PULLS: &[Pull] = &[
    Pull { number: 14, title: "Add procedural Three.js Blue Bunny model", repo: "francescodibella.github.io", author: "claude", state: PullState::Open, checks: Checks::Passing, age: "2 ore fa", comments: 3 },
    Pull { number: 9, title: "Mixed reality context v0.2: anchors and spatial meshes", repo: "ngsi-ld-xr-contexts", author: "francescodibella", state: PullState::Draft, checks: Checks::Running, age: "ieri", comments: 1 },
    Pull { number: 8, title: "Fix @vocab for VR entity relationships", repo: "ngsi-ld-xr-contexts", author: "contributor42", state: PullState::Open, checks: Checks::Failing, age: "4 giorni fa", comments: 6 },
    Pull { number: 3, title: "Repo manager example on the web target", repo: "rune", author: "francescodibella", state: PullState::Open, checks: Checks::Passing, age: "2 settimane fa", comments: 0 },
    Pull { number: 21, title: "Chapter 4: latency measurements", repo: "xr-digital-twin-thesis", author: "francescodibella", state: PullState::Merged, checks: Checks::Passing, age: "3 settimane fa", comments: 12 },
];

pub struct Issue {
    pub number: u32,
    pub title: &'static str,
    pub repo: &'static str,
    pub label: &'static str,
    pub hue: f32,
    pub age: &'static str,
}

pub const ISSUES: &[Issue] = &[
    Issue { number: 31, title: "Bunny poncho clips through lowered arms", repo: "francescodibella.github.io", label: "bug", hue: 25.0, age: "1 ora fa" },
    Issue { number: 12, title: "Document the MR context with examples", repo: "ngsi-ld-xr-contexts", label: "docs", hue: 240.0, age: "2 giorni fa" },
    Issue { number: 44, title: "Measure headset frame pacing over Wi-Fi 6", repo: "xr-digital-twin-thesis", label: "research", hue: 300.0, age: "5 giorni fa" },
    Issue { number: 7, title: "Battery drain on the CO2 sensor node", repo: "iot-sensor-gateway", label: "bug", hue: 25.0, age: "1 settimana fa" },
];

pub struct Run {
    pub workflow: &'static str,
    pub repo: &'static str,
    pub branch: &'static str,
    pub checks: Checks,
    pub duration: &'static str,
    pub age: &'static str,
}

pub const RUNS: &[Run] = &[
    Run { workflow: "pages-build-deployment", repo: "francescodibella.github.io", branch: "main", checks: Checks::Passing, duration: "41s", age: "2 ore fa" },
    Run { workflow: "validate-jsonld", repo: "ngsi-ld-xr-contexts", branch: "mr-v0.2", checks: Checks::Running, duration: "1m 12s", age: "in corso" },
    Run { workflow: "validate-jsonld", repo: "ngsi-ld-xr-contexts", branch: "fix-vocab", checks: Checks::Failing, duration: "38s", age: "4 giorni fa" },
    Run { workflow: "latex-build", repo: "xr-digital-twin-thesis", branch: "dev", checks: Checks::Passing, duration: "3m 05s", age: "3 giorni fa" },
    Run { workflow: "pytest", repo: "iot-sensor-gateway", branch: "main", checks: Checks::Passing, duration: "57s", age: "il mese scorso" },
];

/// File rows shown in a repository's Code tab: (name, is_dir, last commit message, age).
pub const FILES: &[(&str, bool, &str, &str)] = &[
    (".github", true, "Add Pages workflow", "3 settimane fa"),
    ("bunny", true, "Add procedural Three.js Blue Bunny model", "2 ore fa"),
    ("css", true, "Timer widget styles", "l'anno scorso"),
    ("js", true, "Shell command history", "l'anno scorso"),
    ("README.md", false, "Describe the widgets", "il mese scorso"),
    ("index.html", false, "Shell layout", "l'anno scorso"),
    ("context_test.jsonld", false, "Update context_test.jsonld", "l'anno scorso"),
];

/// Commits per week for the activity sparkline (oldest first).
pub const ACTIVITY: [u8; 26] = [2, 4, 1, 0, 3, 6, 8, 5, 2, 0, 1, 4, 7, 9, 6, 3, 2, 5, 8, 12, 10, 6, 4, 7, 11, 9];
