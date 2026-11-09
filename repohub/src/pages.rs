//! Dashboard, repository list and repository detail.
use rune::{El, col, el, row, text};
use rune_kit::{Badge, BadgeVariant, Button, ButtonVariant, ControlSize, Kit, SegmentedControl, TextInput};

use crate::data::{self, ACTIVITY, FILES, ISSUES, PULLS, REPOS, RUNS, Repo, Visibility};
use crate::lists::{issue_row, pull_row, run_row};
use crate::ui::{self, panel, panel_head, rule};
use crate::{App, Page, icons};

pub fn heading(k: &Kit, title: &str, subtitle: &str, action: Option<El>) -> El {
    let mut r = row().w_full().center_y().justify_between().gap(16.0)
        .child(col().gap(4.0).shrink()
            .child(text(title.to_owned()).font(ui::H1).color(k.c.text_strong))
            .child(text(subtitle.to_owned()).font(k.ty.ui).color(k.c.text_muted)));
    if let Some(a) = action {
        r = r.child(a);
    }
    r
}

fn new_repo_button(k: &Kit) -> El {
    Button::new("Nuovo repository").variant(ButtonVariant::Submit).icon(&icons::PLUS)
        .on_press(|s: &mut App| s.toast = Some("Un repository vero richiederebbe l'API di GitHub".into()))
        .build(k).key("new-repo")
}

// ---------------------------------------------------------------- dashboard

pub fn overview(s: &App, k: &Kit) -> El {
    let stars: u32 = REPOS.iter().map(|r| r.stars).sum::<u32>() + s.starred.iter().filter(|x| **x).count() as u32;
    let open_prs = PULLS.iter().filter(|p| p.state != data::PullState::Merged).count();
    let issues: u32 = REPOS.iter().map(|r| r.issues).sum();
    let metrics = row().w_full().gap(12.0).flex_wrap()
        .child(ui::metric(k, &icons::REPO, &REPOS.len().to_string(), "Repository", k.c.info.icon))
        .child(ui::metric(k, &icons::STAR, &stars.to_string(), "Stelle ricevute", k.c.warning.icon))
        .child(ui::metric(k, &icons::PULL, &open_prs.to_string(), "Pull request aperte", k.c.success.icon))
        .child(ui::metric(k, &icons::ISSUE, &issues.to_string(), "Issue aperte", k.c.danger.icon));
    let activity = panel(k)
        .child(panel_head(k, "Attività · ultime 26 settimane", Some(text("166 commit").font(k.ty.text_small).color(k.c.text_muted))))
        .child(col().w_full().px(16.0).pb(16.0).child(ui::sparkline(k, &ACTIVITY, 72.0)));
    let recent = REPOS.iter().enumerate().take(4).fold(
        panel(k).child(panel_head(k, "Repository recenti", Some(see_all(k, "Vedi tutti", Page::Repos)))),
        |p, (i, r)| p.child(rule(k)).child(compact_repo(s, k, i, r)),
    );
    let review = PULLS.iter().filter(|p| p.state != data::PullState::Merged).take(3).fold(
        panel(k).child(panel_head(k, "Da rivedere", Some(see_all(k, "Tutte le PR", Page::Pulls)))),
        |p, pr| p.child(rule(k)).child(pull_row(k, pr, true)),
    );
    col().w_full().gap(16.0)
        .child(heading(k, "Bentornato, Francesco", "Ecco cosa succede nei tuoi repository oggi.", Some(new_repo_button(k))))
        .child(metrics)
        .child(activity)
        .child(row().w_full().gap(16.0).flex_wrap()
            .child(col().grow(1.0).min_w(320.0).child(recent))
            .child(col().grow(1.0).min_w(320.0).child(review)))
}

fn see_all(k: &Kit, label: &str, page: Page) -> El {
    Button::new(label).variant(ButtonVariant::Ghost).control_size(ControlSize::Small)
        .on_press(move |s: &mut App| s.go(page)).build(k)
}

fn compact_repo(s: &App, k: &Kit, i: usize, r: &Repo) -> El {
    row().w_full().px(16.0).py(10.0).gap(12.0).center_y().hover(k.c.hover).press(k.c.pressed)
        .on_press(move |s: &mut App| s.open(i)).key(format!("recent-{}", r.id))
        .child(ui::avatar(k, r.name, r.hue, 28.0))
        .child(col().grow(1.0).shrink().gap(2.0)
            .child(text(r.name.to_owned()).font(k.ty.ui_medium).color(k.c.text_base))
            .child(row().gap(12.0).child(ui::language(k, r.language, r.hue)).child(ui::stat(k, &icons::STAR, stars(s, i).to_string()))))
        .child(text(r.updated.to_owned()).font(k.ty.text_small).color(k.c.text_faint))
}

fn stars(s: &App, i: usize) -> u32 {
    REPOS[i].stars + s.starred[i] as u32
}

// ---------------------------------------------------------------- repositories

pub const FILTERS: [&str; 4] = ["Tutti", "Pubblici", "Privati", "Fork"];

pub fn repos(s: &App, k: &Kit) -> El {
    let q = s.search.text.to_lowercase();
    let shown: Vec<usize> = (0..REPOS.len()).filter(|&i| {
        let r = &REPOS[i];
        let f = match s.filter {
            1 => r.visibility == Visibility::Public,
            2 => r.visibility == Visibility::Private,
            3 => r.fork,
            _ => true,
        };
        f && (q.is_empty() || r.name.contains(&q) || r.description.to_lowercase().contains(&q))
    }).collect();
    let search = TextInput::new(&s.search).placeholder("Cerca un repository…").leading_icon(k.glyph.search)
        .on_edit(|s: &mut App| &mut s.search).build(k).key("search").grow(1.0).min_w(200.0);
    let seg = SegmentedControl::new(&FILTERS, Some(s.filter)).label("Filtro").fit()
        .on_pick(|s: &mut App, i, _| s.filter = i).build(k).key("filter");
    let mut list = col().w_full().gap(12.0);
    for &i in &shown {
        list = list.child(repo_card(s, k, i));
    }
    if shown.is_empty() {
        list = list.child(panel(k).padding(32.0).items_center().gap(6.0)
            .child(ui::ico(&icons::REPO, 24.0, k.c.icon_faint))
            .child(text("Nessun repository corrisponde alla ricerca").font(k.ty.ui).color(k.c.text_muted)));
    }
    col().w_full().gap(16.0)
        .child(heading(k, "Repository", &format!("{} di {} repository", shown.len(), REPOS.len()), Some(new_repo_button(k))))
        .child(row().w_full().gap(12.0).center_y().flex_wrap().child(search).child(seg))
        .child(list)
}

fn repo_card(s: &App, k: &Kit, i: usize) -> El {
    let r = &REPOS[i];
    let on = s.starred[i];
    let vis = match r.visibility {
        Visibility::Public => Badge::new("Pubblico").build(k),
        Visibility::Private => Badge::new("Privato").variant(BadgeVariant::Subtle).build(k),
    };
    let mut title = row().gap(8.0).center_y().flex_wrap()
        .child(text(r.name.to_owned()).font(k.ty.body_medium).color(k.c.text_accent)
            .on_press(move |s: &mut App| s.open(i)).key(format!("open-{}", r.id)))
        .child(vis);
    if r.fork {
        title = title.child(Badge::new("Fork").variant(BadgeVariant::Accent).build(k));
    }
    let star = Button::new(if on { "Starred" } else { "Star" })
        .icon(if on { &icons::STAR_ON } else { &icons::STAR })
        .variant(if on { ButtonVariant::Warning } else { ButtonVariant::Neutral }).control_size(ControlSize::Small)
        .on_press(move |s: &mut App| s.starred[i] = !s.starred[i]).build(k).key(format!("star-{}", r.id));
    let topics = row().gap(6.0).flex_wrap().children(r.topics.iter().map(|t| ui::topic(k, t)));
    let foot = row().gap(16.0).center_y().flex_wrap()
        .child(ui::language(k, r.language, r.hue))
        .child(ui::stat(k, &icons::STAR, stars(s, i).to_string()))
        .child(ui::stat(k, &icons::FORK, r.forks.to_string()))
        .child(ui::stat(k, &icons::ISSUE, r.issues.to_string()))
        .child(text(format!("Aggiornato {}", r.updated)).font(k.ty.text_small).color(k.c.text_faint));
    panel(k).padding(16.0).gap(10.0).key(format!("card-{}", r.id))
        .child(row().w_full().justify_between().gap(12.0)
            .child(col().grow(1.0).shrink().gap(6.0)
                .child(title)
                .child(text(r.description.to_owned()).font(k.ty.ui).color(k.c.text_muted).wrap()))
            .child(star))
        .child(topics)
        .child(foot)
}

// ---------------------------------------------------------------- repository detail

pub const TABS: [&str; 5] = ["Codice", "Issue", "Pull request", "Actions", "Impostazioni"];

pub fn repo_detail(s: &App, k: &Kit, i: usize) -> El {
    let r = &REPOS[i];
    let on = s.starred[i];
    let back = Button::new("Repository").icon(&icons::BACK).variant(ButtonVariant::Ghost).control_size(ControlSize::Small)
        .on_press(|s: &mut App| s.go(Page::Repos)).build(k).key("back").self_start();
    let action = |label: String, ic: &'static rune::Icon, key: &str| {
        Button::new(label).icon(ic).control_size(ControlSize::Small).build(k).key(key.to_owned())
    };
    let star = Button::new(format!("{} · {}", if on { "Starred" } else { "Star" }, stars(s, i)))
        .icon(if on { &icons::STAR_ON } else { &icons::STAR })
        .variant(if on { ButtonVariant::Warning } else { ButtonVariant::Neutral }).control_size(ControlSize::Small)
        .on_press(move |s: &mut App| s.starred[i] = !s.starred[i]).build(k).key("detail-star");
    let vis = match r.visibility {
        Visibility::Public => ui::pill(k, "Pubblico", k.c.text_muted),
        Visibility::Private => ui::pill(k, "Privato", k.c.warning.icon),
    };
    let header = row().w_full().center_y().justify_between().gap(12.0).flex_wrap()
        .child(row().gap(10.0).center_y()
            .child(ui::avatar(k, r.name, r.hue, 36.0))
            .child(col().gap(2.0)
                .child(text("francescodibella /").font(k.ty.text_small).color(k.c.text_muted))
                .child(row().gap(8.0).center_y().child(text(r.name.to_owned()).font(k.ty.body_medium).color(k.c.text_strong)).child(vis))))
        .child(row().gap(8.0)
            .child(action("Watch".into(), &icons::BELL, "watch"))
            .child(action(format!("Fork · {}", r.forks), &icons::FORK, "fork"))
            .child(star));
    let tabs = SegmentedControl::new(&TABS, Some(s.tab)).label("Sezione").fit().on_pick(|s: &mut App, t, _| s.tab = t).build(k).key("tabs");
    let body = match s.tab {
        0 => code_tab(s, k, r),
        1 => list_of(k, "Issue aperte", ISSUES.iter().filter(|x| x.repo == r.name).map(|x| issue_row(k, x)).collect()),
        2 => list_of(k, "Pull request", PULLS.iter().filter(|x| x.repo == r.name).map(|x| pull_row(k, x, false)).collect()),
        3 => list_of(k, "Esecuzioni dei workflow", RUNS.iter().filter(|x| x.repo == r.name).map(|x| run_row(k, x)).collect()),
        _ => crate::lists::repo_settings(s, k, r),
    };
    col().w_full().gap(16.0)
        .child(back)
        .child(header)
        .child(tabs)
        .child(body)
}

fn list_of(k: &Kit, title: &str, rows: Vec<El>) -> El {
    let empty = rows.is_empty();
    let mut p = panel(k).child(panel_head(k, title, None));
    for r in rows {
        p = p.child(rule(k)).child(r);
    }
    if empty {
        p = p.child(rule(k)).child(col().w_full().padding(28.0).items_center()
            .child(text("Niente da mostrare qui").font(k.ty.ui).color(k.c.text_muted)));
    }
    p
}

fn code_tab(s: &App, k: &Kit, r: &Repo) -> El {
    let branch = Button::new(r.branch).icon(&icons::BRANCH).control_size(ControlSize::Small).build(k).key("branch");
    let clone = Button::new("Code").variant(ButtonVariant::Submit).control_size(ControlSize::Small).build(k).key("clone");
    let commit = row().w_full().px(16.0).py(10.0).gap(10.0).center_y().fill(k.c.layer_01).radius(k.radius.lg)
        .child(ui::user(k, "francescodibella", 22.0))
        .child(text("francescodibella").font(k.ty.small_medium).color(k.c.text_base))
        .child(text("Add procedural Three.js Blue Bunny model").font(k.ty.small).color(k.c.text_muted).grow(1.0).shrink())
        .child(text("0a14552 · 2 ore fa").font(k.ty.mono).color(k.c.text_faint));
    let mut files = panel(k).child(commit);
    for (name, dir, msg, age) in FILES {
        files = files.child(rule(k)).child(
            row().w_full().px(16.0).py(9.0).gap(10.0).center_y().hover(k.c.hover)
                .child(ui::ico(if *dir { &icons::FOLDER } else { &icons::FILE }, 16.0, if *dir { k.c.info.icon } else { k.c.icon_muted }))
                .child(text((*name).to_owned()).font(k.ty.small).color(k.c.text_base).w(180.0))
                .child(text((*msg).to_owned()).font(k.ty.small).color(k.c.text_muted).grow(1.0).shrink())
                .child(text((*age).to_owned()).font(k.ty.small).color(k.c.text_faint)),
        );
    }
    let readme = panel(k)
        .child(panel_head(k, "README.md", Some(ui::ico(&icons::BOOK, 16.0, k.c.icon_muted))))
        .child(rule(k))
        .child(col().w_full().padding(20.0).gap(10.0)
            .child(text(r.name.to_owned()).font(ui::H2).color(k.c.text_strong))
            .child(text(r.description.to_owned()).font(k.ty.body).color(k.c.text_muted).wrap())
            .child(text("Avvio rapido").font(k.ty.body_medium).color(k.c.text_base))
            .child(text(format!("git clone https://github.com/francescodibella/{}.git", r.name)).font(k.ty.mono).color(k.c.text_base)
                .w_full().padding(12.0).radius(k.radius.md).fill(k.c.layer_01)));
    let about = panel(k).padding(16.0).gap(12.0)
        .child(text("Informazioni").font(k.ty.ui_medium).color(k.c.text_base))
        .child(text(r.description.to_owned()).font(k.ty.small).color(k.c.text_muted).wrap())
        .child(row().gap(6.0).flex_wrap().children(r.topics.iter().map(|t| ui::topic(k, t))))
        .child(col().gap(8.0)
            .child(ui::stat(k, &icons::STAR, format!("{} stelle", r.stars + s.starred.get(REPOS.iter().position(|x| x.id == r.id).unwrap_or(0)).copied().unwrap_or(false) as u32)))
            .child(ui::stat(k, &icons::FORK, format!("{} fork", r.forks)))
            .child(ui::stat(k, &icons::BRANCH, format!("branch predefinito: {}", r.branch))))
        .child(rule(k))
        .child(text("Linguaggi").font(k.ty.ui_medium).color(k.c.text_base))
        .child(row().w_full().h(8.0).radius(4.0).clip_rounded(4.0)
            .child(el().grow(7.0).h_full().fill(rune::oklch(0.68, 0.15, r.hue)))
            .child(el().grow(2.0).h_full().fill(rune::oklch(0.68, 0.15, 30.0)))
            .child(el().grow(1.0).h_full().fill(rune::oklch(0.68, 0.15, 200.0))))
        .child(row().gap(12.0).flex_wrap().child(ui::language(k, &format!("{} 70%", r.language), r.hue)).child(ui::language(k, "HTML 20%", 30.0)).child(ui::language(k, "CSS 10%", 200.0)))
        .child(rule(k))
        .child(text("Commit settimanali").font(k.ty.ui_medium).color(k.c.text_base))
        .child(ui::sparkline(k, &ACTIVITY[10..], 40.0));
    row().w_full().gap(16.0).flex_wrap()
        .child(col().grow(3.0).min_w(380.0).gap(12.0)
            .child(row().w_full().justify_between().child(branch).child(clone))
            .child(files)
            .child(readme))
        .child(col().grow(1.0).min_w(240.0).child(about))
}
