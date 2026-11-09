//! RepoHub: a personal GitHub repository manager built with Rune and rune-kit.
//!
//! It reads your repositories, pull requests, issues and workflow runs from the GitHub REST API.
//! The desktop app is configured by a `.env` file (see `.env.example`); the web build gets its
//! configuration and a token-adding proxy from `repohub-server` (the Docker image). Without either
//! it shows built-in sample data.
use std::collections::HashMap;
use std::sync::Arc;

use rune::{AppHandle, AppSpec, Cx, El, KeyEvent, ScrollSpec, TextEdit, Tx, col, row, text};
use rune_kit::shell::{self, PaneState, Sidebar, SidebarRow, SidebarSection, SidebarToggle, SplitView, Toolbar};
use rune_kit::{ControlSize, IconButton, IconButtonVariant, Kit, PaletteSeed};

mod config;
mod github;
mod icons;
mod lists;
mod model;
mod net;
mod pages;
mod ui;

use config::Config;
use model::{Data, Detail, PullState};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Repos,
    Repo(usize),
    Pulls,
    Issues,
    Actions,
    Settings,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Status {
    /// Showing the sample data (no configuration, demo mode or web build).
    Sample,
    Loading,
    Ready,
    Error(String),
}

pub struct App {
    sidebar: PaneState,
    page: Page,
    filter: usize,
    tab: usize,
    pr_filter: usize,
    default_vis: usize,
    search: TextEdit,
    prefs: [bool; 5],
    pinned_open: bool,
    toast: Option<String>,
    cfg: Arc<Config>,
    data: Data,
    details: HashMap<String, Detail>,
    status: Status,
    handle: Option<AppHandle<App>>,
}

impl App {
    fn go(&mut self, p: Page) {
        self.page = p;
        self.toast = None;
    }

    fn open(&mut self, i: usize) {
        self.tab = 0;
        self.go(Page::Repo(i));
        self.load_detail(i);
    }

    /// Reloads everything from GitHub in the background.
    fn refresh(&mut self) {
        if !self.cfg.live() || self.status == Status::Loading {
            return;
        }
        self.status = Status::Loading;
        if let Some(h) = self.handle.clone() {
            let cfg = self.cfg.clone();
            net::spawn(h, move || async move { github::load_all(&cfg).await }, |s: &mut App, r| s.apply(r));
        }
    }

    fn apply(&mut self, result: Result<Data, String>) {
        match result {
            Ok(d) => {
                self.data = d;
                self.details.clear();
                self.status = Status::Ready;
                if let Page::Repo(i) = self.page {
                    if i < self.data.repos.len() { self.load_detail(i) } else { self.page = Page::Repos }
                }
            }
            Err(e) => self.status = Status::Error(e),
        }
    }

    fn load_detail(&mut self, i: usize) {
        let Some(r) = self.data.repos.get(i) else { return };
        let full = r.full_name.clone();
        if self.data.sample {
            let d = model::sample_detail(r);
            self.details.insert(full, d);
            return;
        }
        if self.details.contains_key(&full) {
            return;
        }
        self.details.insert(full.clone(), Detail { loading: true, ..Default::default() });
        if let Some(h) = self.handle.clone() {
            let cfg = self.cfg.clone();
            let key = full.clone();
            net::spawn(h, move || async move { github::load_detail(&cfg, &full).await }, move |s: &mut App, d| {
                s.details.insert(key, d);
            });
        }
    }

    /// Stars or unstars repository `i`: on GitHub when a token is configured, locally otherwise.
    fn toggle_star(&mut self, i: usize) {
        let Some(r) = self.data.repos.get_mut(i) else { return };
        let on = !r.starred;
        r.starred = on;
        r.stars = if on { r.stars + 1 } else { r.stars.saturating_sub(1) };
        if !self.cfg.live() || self.cfg.token.is_none() {
            if self.cfg.live() {
                self.toast = Some("Stella cambiata solo qui: serve GITHUB_TOKEN per salvarla su GitHub.".into());
            }
            return;
        }
        if let Some(h) = self.handle.clone() {
            let (cfg, full) = (self.cfg.clone(), r.full_name.clone());
            let key = full.clone();
            net::spawn(h, move || async move { github::set_star(&cfg, &full, on).await }, move |s: &mut App, res| {
                if let Err(e) = res {
                    if let Some(r) = s.data.repos.iter_mut().find(|r| r.full_name == key) {
                        r.starred = !on;
                        r.stars = if on { r.stars.saturating_sub(1) } else { r.stars + 1 };
                    }
                    s.toast = Some(format!("Non sono riuscito a cambiare la stella: {e}"));
                }
            });
        }
    }

    fn starred_count(&self) -> u32 {
        self.data.repos.iter().map(|r| r.stars).sum()
    }
}

fn init() -> App {
    let args = rune::extra_args();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let cfg = Arc::new(Config::load());
    let live = cfg.live();
    let mut s = App {
        sidebar: PaneState::new(248.0).range(180.0, 360.0),
        page: Page::Overview,
        filter: 0,
        tab: 0,
        pr_filter: 0,
        default_vis: 0,
        search: TextEdit::new(""),
        prefs: [cfg.theme == Some(true), true, false, true, true],
        pinned_open: true,
        toast: None,
        data: if live { Data::default() } else { Data::sample() },
        details: HashMap::new(),
        status: if live { Status::Loading } else { Status::Sample },
        handle: None,
        cfg,
    };
    // `REPOHUB_BLOCKING=1` loads synchronously before the first frame (headless snapshots).
    #[cfg(not(target_arch = "wasm32"))]
    if live && std::env::var("REPOHUB_BLOCKING").is_ok_and(|v| v == "1") {
        let r = pollster::block_on(github::load_all(&s.cfg));
        s.apply(r);
        if let Some(i) = arg("page").filter(|p| p == "repo").map(|_| 0)
            && let Some(r) = s.data.repos.get(i)
        {
            let full = r.full_name.clone();
            s.details.insert(full.clone(), pollster::block_on(github::load_detail(&s.cfg, &full)));
        }
    }
    // `-- page repos|repo|pulls|issues|actions|settings` opens a page (for headless stills).
    s.page = match arg("page").as_deref() {
        Some("repos") => Page::Repos,
        Some("repo") if !s.data.repos.is_empty() => Page::Repo(0),
        Some("pulls") => Page::Pulls,
        Some("issues") => Page::Issues,
        Some("actions") => Page::Actions,
        Some("settings") => Page::Settings,
        _ => Page::Overview,
    };
    if s.data.sample && matches!(s.page, Page::Repo(_)) {
        s.load_detail(0);
    }
    if let Some(t) = arg("tab").and_then(|t| t.parse().ok()) {
        s.tab = t;
    }
    if let Some(q) = arg("search") {
        s.search = TextEdit::new(q);
    }
    s
}

/// GitHub-like palette seeds: canvas, ink and the blue accent.
fn kit(dark: bool) -> Kit {
    let c = |h: &str| rune::color::hex(h).expect("valid colour");
    let mut k = if dark {
        Kit::from_palette(PaletteSeed { bg: c("#0d1117"), fg: c("#e6edf3"), accent: c("#2f81f7") })
    } else {
        Kit::from_palette(PaletteSeed { bg: c("#f6f8fa"), fg: c("#1f2328"), accent: c("#0969da") })
    };
    k.radius.lg = 10.0;
    k
}

fn nav(s: &App, k: &Kit, label: &str, icon: &'static rune::Icon, page: Page, badge: Option<usize>) -> El {
    let selected = s.page == page || (page == Page::Repos && matches!(s.page, Page::Repo(_)));
    let mut r = SidebarRow::new(label).icon(icon).height(32.0).selected(selected).on_select(move |s: &mut App, _| s.go(page));
    if let Some(n) = badge.filter(|n| *n > 0) {
        r = r.badge(n.to_string());
    }
    r.build(k).key(format!("nav-{label}"))
}

/// The pinned repositories: the names in REPOHUB_PINNED, else the three most recently pushed.
fn pinned(s: &App) -> Vec<usize> {
    let repos = &s.data.repos;
    if s.cfg.pinned.is_empty() {
        return (0..repos.len().min(3)).collect();
    }
    s.cfg.pinned.iter().filter_map(|p| repos.iter().position(|r| r.name.eq_ignore_ascii_case(p) || r.full_name.eq_ignore_ascii_case(p))).collect()
}

fn sidebar(s: &App, k: &Kit) -> El {
    let open_prs = s.data.pulls.iter().filter(|p| matches!(p.state, PullState::Open | PullState::Draft)).count();
    let brand = row().w_full().h(40.0).px(10.0).gap(10.0).center_y()
        .child(row().size(26.0, 26.0).radius(7.0).center().fill(k.c.text_strong)
            .child(ui::ico(&icons::REPO, 15.0, k.c.page)))
        .child(text(s.cfg.title.clone()).font(k.ty.body_medium).color(k.c.text_strong));
    let pinned: Vec<El> = pinned(s).into_iter().map(|i| {
        let r = &s.data.repos[i];
        SidebarRow::new(r.name.clone()).avatar(ui::avatar(k, &r.name, r.hue, 16.0)).height(30.0)
            .selected(s.page == Page::Repo(i)).on_select(move |s: &mut App, _| s.open(i)).build(k).key(format!("pin-{}", r.full_name))
    }).collect();
    Sidebar::new()
        .label("Navigazione")
        .header(brand)
        .header(nav(s, k, "Panoramica", &icons::HOME, Page::Overview, None))
        .header(nav(s, k, "Repository", &icons::REPO, Page::Repos, Some(s.data.repos.len())))
        .header(nav(s, k, "Pull request", &icons::PULL, Page::Pulls, Some(open_prs)))
        .header(nav(s, k, "Issue", &icons::ISSUE, Page::Issues, Some(s.data.issues.len())))
        .header(nav(s, k, "Actions", &icons::PLAY, Page::Actions, None))
        .section(SidebarSection::new("Fissati").count(pinned.len()).collapsed(!s.pinned_open)
            .on_toggle(|s: &mut App, _| s.pinned_open = !s.pinned_open).rows(pinned))
        .footer(nav(s, k, "Impostazioni", &icons::GEAR, Page::Settings, None))
        .build(k)
}

fn title(s: &App) -> String {
    match s.page {
        Page::Overview => "Panoramica".into(),
        Page::Repos => "Repository".into(),
        Page::Repo(i) => s.data.repos.get(i).map(|r| r.full_name.replace('/', " / ")).unwrap_or_default(),
        Page::Pulls => "Pull request".into(),
        Page::Issues => "Issue".into(),
        Page::Actions => "Actions".into(),
        Page::Settings => "Impostazioni".into(),
    }
}

/// A banner above the page for loading, errors and sample data.
fn banner(s: &App, k: &Kit) -> Option<El> {
    let (msg, tint, action) = match &s.status {
        Status::Loading => ("Caricamento dei dati da GitHub…".to_owned(), k.c.info.icon, false),
        Status::Error(e) => (e.clone(), k.c.danger.icon, true),
        Status::Sample if !s.cfg.demo => ("Dati di esempio. Per vedere i tuoi repository crea un file .env con GITHUB_USER o GITHUB_TOKEN (vedi .env.example) e avvia RepoHub con Docker o come app desktop.".to_owned(), k.c.warning.icon, false),
        _ => return None,
    };
    let mut r = row().w_full().px(14.0).py(10.0).gap(10.0).center_y().radius(k.radius.lg)
        .fill(tint.alpha(0.12)).border(1.0, tint.alpha(0.45))
        .child(ui::ico(if action { &icons::CROSS } else { &icons::BELL }, 15.0, tint))
        .child(text(msg).font(k.ty.small).color(k.c.text_base).wrap().grow(1.0).shrink());
    if action {
        r = r.child(rune_kit::Button::new("Riprova").control_size(ControlSize::Small).on_press(|s: &mut App| {
            s.status = Status::Ready;
            s.refresh();
        }).build(k).key("retry"));
    }
    Some(r.key("banner"))
}

fn view(s: &App, cx: &mut Cx<App>) -> El {
    let dark = s.prefs[0] || (s.cfg.theme.is_none() && cx.system_dark());
    let k = kit(dark);
    let content = match s.page {
        Page::Overview => pages::overview(s, &k),
        Page::Repos => pages::repos(s, &k),
        Page::Repo(i) if i < s.data.repos.len() => pages::repo_detail(s, &k, i),
        Page::Repo(_) => pages::repos(s, &k),
        Page::Pulls => lists::pulls(s, &k),
        Page::Issues => lists::issues(s, &k),
        Page::Actions => lists::actions(s, &k),
        Page::Settings => lists::settings(s, &k),
    };
    let mut inner = col().w_full().max_w(1080.0).gap(16.0);
    if let Some(b) = banner(s, &k) {
        inner = inner.child(b);
    }
    let mut page = col().w_full().padding(24.0).gap(16.0).items_center().child(inner.child(content));
    if let Some(msg) = &s.toast {
        page = page.child(row().gap(10.0).px(14.0).py(10.0).radius(k.radius.lg).fill(k.c.bg_inverse)
            .child(text(msg.clone()).font(k.ty.small).color(k.c.text_inverse)));
    }
    let main = col().key("page").w_full().h_full().shrink().fill(k.c.page)
        .scroll(ScrollSpec { stick_bottom: false }).child(page);
    let theme = IconButton::new(&icons::MOON).variant(IconButtonVariant::GhostMuted).label("Tema")
        .help("Tema scuro").expanded(s.prefs[0]).on_press(|s: &mut App| s.prefs[0] = !s.prefs[0]).build(&k).key("theme");
    let mut trailing = row().gap(4.0).center_y();
    if s.cfg.live() {
        trailing = trailing.child(IconButton::new(&icons::REFRESH).variant(IconButtonVariant::GhostMuted).label("Aggiorna")
            .help("Aggiorna da GitHub").disabled(s.status == Status::Loading).on_press(|s: &mut App| s.refresh()).build(&k).key("refresh"));
    }
    let who = if s.data.profile.login.is_empty() { "?".to_owned() } else { s.data.profile.login.clone() };
    let toolbar = Toolbar::new().title(title(s)).trailing(trailing
        .child(theme)
        .child(ui::user(&k, &who, 26.0).ml(6.0)));
    SplitView::new(main)
        .sidebar(&s.sidebar, |s: &mut App| &mut s.sidebar, sidebar(s, &k))
        .toolbar(toolbar)
        .titlebar_inset(cx.titlebar_inset())
        .sidebar_toggle(SidebarToggle::new().size(ControlSize::Large))
        .reduced_motion(cx.is_reduced_motion())
        .build(&k)
}

fn on_key(s: &mut App, ev: &KeyEvent, _: &mut Tx) -> bool {
    if shell::is_sidebar_toggle(ev) {
        s.sidebar.toggle();
        return true;
    }
    if ev.is_command('r') {
        s.refresh();
        return true;
    }
    false
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn start() {
    let app = rune::App::new(AppSpec { on_key: Some(on_key), ..AppSpec::new("RepoHub", (1180.0, 760.0), init, view) });
    let handle = app.handle();
    let h2 = handle.clone();
    handle.update(move |s: &mut App, _| {
        s.handle = Some(h2.clone());
        if s.status == Status::Loading && s.data.repos.is_empty() {
            s.status = Status::Ready; // let refresh() start the first load
            s.refresh();
        }
        // The web build asks repohub-server for its configuration; a static host has none.
        #[cfg(target_arch = "wasm32")]
        net::spawn(h2, || net::fetch_config(), |s: &mut App, cfg| {
            if let Some(cfg) = cfg.filter(|c| c.live()) {
                if cfg.theme == Some(true) {
                    s.prefs[0] = true;
                }
                s.cfg = Arc::new(cfg);
                s.data = Data::default();
                s.status = Status::Ready;
                s.refresh();
            }
        });
    });
    app.run();
}
