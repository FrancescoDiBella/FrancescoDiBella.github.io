//! RepoHub: a personal GitHub repository manager, UI only (no network, sample data), built
//! with Rune and rune-kit. Runs as a desktop window, headless (`--snap`) and on the web.
use rune::{AppSpec, Cx, El, KeyEvent, ScrollSpec, TextEdit, Tx, col, row, text};
use rune_kit::shell::{self, PaneState, Sidebar, SidebarRow, SidebarSection, SidebarToggle, SplitView, Toolbar};
use rune_kit::{ControlSize, IconButton, IconButtonVariant, Kit, PaletteSeed};

mod data;
mod icons;
mod lists;
mod pages;
mod ui;

use data::{PULLS, PullState, REPOS};

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

pub struct App {
    sidebar: PaneState,
    page: Page,
    filter: usize,
    tab: usize,
    pr_filter: usize,
    default_vis: usize,
    search: TextEdit,
    starred: Vec<bool>,
    prefs: [bool; 5],
    pinned_open: bool,
    toast: Option<String>,
}

impl App {
    fn go(&mut self, p: Page) {
        self.page = p;
        self.toast = None;
    }
    fn open(&mut self, i: usize) {
        self.tab = 0;
        self.go(Page::Repo(i));
    }
}

fn init() -> App {
    let args = rune::extra_args();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let mut s = App {
        sidebar: PaneState::new(248.0).range(180.0, 360.0),
        page: Page::Overview,
        filter: 0,
        tab: 0,
        pr_filter: 0,
        default_vis: 0,
        search: TextEdit::new(""),
        starred: REPOS.iter().map(|r| r.id == "ngsi").collect(),
        prefs: [false, true, false, true, true],
        pinned_open: true,
        toast: None,
    };
    // `-- page repos|repo|pulls|issues|actions|settings` opens a page (for headless stills).
    s.page = match arg("page").as_deref() {
        Some("repos") => Page::Repos,
        Some("repo") => Page::Repo(0),
        Some("pulls") => Page::Pulls,
        Some("issues") => Page::Issues,
        Some("actions") => Page::Actions,
        Some("settings") => Page::Settings,
        _ => Page::Overview,
    };
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
    if let Some(n) = badge {
        r = r.badge(n.to_string());
    }
    r.build(k).key(format!("nav-{label}"))
}

fn sidebar(s: &App, k: &Kit) -> El {
    let open_prs = PULLS.iter().filter(|p| p.state != PullState::Merged).count();
    let brand = row().w_full().h(40.0).px(10.0).gap(10.0).center_y()
        .child(row().size(26.0, 26.0).radius(7.0).center().fill(k.c.text_strong)
            .child(ui::ico(&icons::REPO, 15.0, k.c.page)))
        .child(text("RepoHub").font(k.ty.body_medium).color(k.c.text_strong));
    let pinned: Vec<El> = [0usize, 1, 2].iter().map(|&i| {
        let r = &REPOS[i];
        SidebarRow::new(r.name).avatar(ui::avatar(k, r.name, r.hue, 16.0)).height(30.0)
            .selected(s.page == Page::Repo(i)).on_select(move |s: &mut App, _| s.open(i)).build(k).key(format!("pin-{}", r.id))
    }).collect();
    Sidebar::new()
        .label("Navigazione")
        .header(brand)
        .header(nav(s, k, "Panoramica", &icons::HOME, Page::Overview, None))
        .header(nav(s, k, "Repository", &icons::REPO, Page::Repos, Some(REPOS.len())))
        .header(nav(s, k, "Pull request", &icons::PULL, Page::Pulls, Some(open_prs)))
        .header(nav(s, k, "Issue", &icons::ISSUE, Page::Issues, Some(data::ISSUES.len())))
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
        Page::Repo(i) => format!("francescodibella / {}", REPOS[i].name),
        Page::Pulls => "Pull request".into(),
        Page::Issues => "Issue".into(),
        Page::Actions => "Actions".into(),
        Page::Settings => "Impostazioni".into(),
    }
}

fn view(s: &App, cx: &mut Cx<App>) -> El {
    let dark = s.prefs[0] || cx.system_dark();
    let k = kit(dark);
    let content = match s.page {
        Page::Overview => pages::overview(s, &k),
        Page::Repos => pages::repos(s, &k),
        Page::Repo(i) => pages::repo_detail(s, &k, i),
        Page::Pulls => lists::pulls(s, &k),
        Page::Issues => lists::issues(s, &k),
        Page::Actions => lists::actions(s, &k),
        Page::Settings => lists::settings(s, &k),
    };
    let mut page = col().w_full().padding(24.0).gap(16.0).items_center()
        .child(col().w_full().max_w(1080.0).child(content));
    if let Some(msg) = &s.toast {
        page = page.child(row().gap(10.0).px(14.0).py(10.0).radius(k.radius.lg).fill(k.c.bg_inverse)
            .child(text(msg.clone()).font(k.ty.small).color(k.c.text_inverse)));
    }
    let main = col().key("page").w_full().h_full().shrink().fill(k.c.page)
        .scroll(ScrollSpec { stick_bottom: false }).child(page);
    let icon_btn = |i: &'static rune::Icon, label: &str, key: &str| {
        IconButton::new(i).variant(IconButtonVariant::GhostMuted).label(label.to_owned()).help(label.to_owned()).build(&k).key(key.to_owned())
    };
    let theme = IconButton::new(&icons::MOON).variant(IconButtonVariant::GhostMuted).label("Tema")
        .help("Tema scuro").expanded(s.prefs[0]).on_press(|s: &mut App| s.prefs[0] = !s.prefs[0]).build(&k).key("theme");
    let toolbar = Toolbar::new().title(title(s)).trailing(row().gap(4.0).center_y()
        .child(icon_btn(&icons::BELL, "Notifiche", "bell"))
        .child(theme)
        .child(ui::user(&k, "francescodibella", 26.0).ml(6.0)));
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
    false
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn start() {
    rune::run(AppSpec { on_key: Some(on_key), ..AppSpec::new("RepoHub", (1180.0, 760.0), init, view) });
}
