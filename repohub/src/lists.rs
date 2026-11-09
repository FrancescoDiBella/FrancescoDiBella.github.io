//! Pull requests, issues, Actions and settings pages.
use rune::{El, col, row, text};
use rune_kit::{Button, ButtonVariant, ControlSize, Kit, SegmentedControl, Switch};

use crate::model::{Checks, Issue, Pull, PullState, Repo, Run};
use crate::pages::{heading, open_button};
use crate::ui::{self, panel, panel_head, rule};
use crate::{App, Status, icons};

/// Makes a row open `url` in the browser when clicked.
fn link(e: El, url: &str) -> El {
    if url.is_empty() {
        return e;
    }
    let url = url.to_owned();
    e.on_press_tx(move |_: &mut App, tx| { let _ = tx.open_url(&url); })
}

pub fn pull_row(k: &Kit, p: &Pull, compact: bool) -> El {
    let mut meta = row().gap(10.0).center_y().flex_wrap()
        .child(text(format!("#{} · {} · {}", p.number, p.repo, p.age)).font(k.ty.text_small).color(k.c.text_muted));
    match p.state {
        PullState::Draft => meta = meta.child(ui::pill(k, "Bozza", k.c.text_faint)),
        PullState::Closed => meta = meta.child(ui::pill(k, "Chiusa", k.c.danger.icon)),
        _ => {}
    }
    let mut r = row().w_full().px(16.0).py(11.0).gap(12.0).center_y().hover(k.c.hover).press(k.c.pressed)
        .child(ui::pull_icon(k, p.state))
        .child(col().grow(1.0).shrink().gap(3.0)
            .child(text(p.title.clone()).font(k.ty.ui_medium).color(k.c.text_base).wrap())
            .child(meta));
    if let Some(c) = p.checks {
        r = r.child(ui::checks(k, c));
    }
    if !compact {
        r = r.child(ui::stat(k, &icons::COMMENT, p.comments.to_string())).child(ui::user(k, &p.author, 22.0));
    }
    link(r.key(format!("pr-{}-{}", p.repo, p.number)), &p.url)
}

pub fn issue_row(k: &Kit, i: &Issue) -> El {
    let mut title = row().gap(8.0).center_y().flex_wrap().child(text(i.title.clone()).font(k.ty.ui_medium).color(k.c.text_base));
    if let Some((name, hue)) = &i.label {
        title = title.child(ui::label_chip(k, name, *hue));
    }
    let r = row().w_full().px(16.0).py(11.0).gap(12.0).center_y().hover(k.c.hover).press(k.c.pressed)
        .child(ui::ico(&icons::ISSUE, 16.0, k.c.success.icon))
        .child(col().grow(1.0).shrink().gap(3.0)
            .child(title)
            .child(text(format!("#{} · {} · aperta {}", i.number, i.repo, i.age)).font(k.ty.text_small).color(k.c.text_muted)))
        .child(ui::stat(k, &icons::COMMENT, i.comments.to_string()));
    link(r.key(format!("issue-{}-{}", i.repo, i.number)), &i.url)
}

pub fn run_row(k: &Kit, r: &Run) -> El {
    let state = match r.checks {
        Checks::Passing => ("Riuscito", k.c.success.icon),
        Checks::Failing => ("Fallito", k.c.danger.icon),
        Checks::Running => ("In corso", k.c.warning.icon),
    };
    let e = row().w_full().px(16.0).py(11.0).gap(12.0).center_y().hover(k.c.hover).press(k.c.pressed)
        .child(ui::checks(k, r.checks))
        .child(col().grow(1.0).shrink().gap(3.0)
            .child(text(r.workflow.clone()).font(k.ty.ui_medium).color(k.c.text_base))
            .child(row().gap(10.0).center_y().flex_wrap()
                .child(text(r.repo.clone()).font(k.ty.text_small).color(k.c.text_muted))
                .child(ui::stat(k, &icons::BRANCH, r.branch.clone()))))
        .child(ui::pill(k, state.0, state.1))
        .child(text(format!("{} · {}", r.duration, r.age)).font(k.ty.text_small).color(k.c.text_faint).w(140.0));
    link(e, &r.url)
}

fn listing(k: &Kit, rows: Vec<El>, empty: &str) -> El {
    if rows.is_empty() {
        return panel(k).padding(28.0).items_center().child(text(empty.to_owned()).font(k.ty.ui).color(k.c.text_muted));
    }
    rows.into_iter().enumerate().fold(panel(k), |p, (i, r)| if i == 0 { p.child(r) } else { p.child(rule(k)).child(r) })
}

pub const PR_FILTERS: [&str; 4] = ["Aperte", "Bozze", "Unite", "Chiuse"];

pub fn pulls(s: &App, k: &Kit) -> El {
    let want = [PullState::Open, PullState::Draft, PullState::Merged, PullState::Closed][s.pr_filter.min(3)];
    let rows: Vec<El> = s.data.pulls.iter().filter(|p| p.state == want).map(|p| pull_row(k, p, false)).collect();
    let sub = if s.data.sample { "Le PR create da te o in attesa della tua revisione." } else { "Le PR in cui sei coinvolto (autore, revisore, assegnatario o menzionato). Clic per aprirle su GitHub." };
    col().w_full().gap(16.0)
        .child(heading(k, "Pull request", sub, None))
        .child(SegmentedControl::new(&PR_FILTERS, Some(s.pr_filter)).label("Stato").fit()
            .on_pick(|s: &mut App, i, _| s.pr_filter = i).build(k).key("pr-filter"))
        .child(listing(k, rows, "Nessuna pull request in questo stato"))
}

pub fn issues(s: &App, k: &Kit) -> El {
    col().w_full().gap(16.0)
        .child(heading(k, "Issue", "Issue aperte nei tuoi repository. Clic per aprirle su GitHub.", None))
        .child(listing(k, s.data.issues.iter().map(|i| issue_row(k, i)).collect(), "Nessuna issue aperta"))
}

pub fn actions(s: &App, k: &Kit) -> El {
    let ok = s.data.runs.iter().filter(|r| r.checks == Checks::Passing).count();
    let sub = format!("{ok} di {} esecuzioni recenti riuscite · ultimi {} repository aggiornati", s.data.runs.len(), s.cfg.activity_repos);
    col().w_full().gap(16.0)
        .child(heading(k, "Actions", &sub, None))
        .child(listing(k, s.data.runs.iter().map(|r| run_row(k, r)).collect(), "Nessuna esecuzione recente"))
}

fn setting(k: &Kit, title: &str, desc: &str, on: bool, i: usize) -> El {
    row().w_full().px(16.0).py(12.0).gap(16.0).center_y().justify_between()
        .child(col().grow(1.0).shrink().gap(2.0)
            .child(text(title.to_owned()).font(k.ty.ui_medium).color(k.c.text_base))
            .child(text(desc.to_owned()).font(k.ty.text_small).color(k.c.text_muted).wrap()))
        .child(Switch::new(on).on_toggle(move |s: &mut App| s.prefs[i] = !s.prefs[i]).build(k).key(format!("pref-{i}")))
}

fn info(k: &Kit, label: &str, value: String) -> El {
    row().w_full().px(16.0).py(10.0).gap(16.0).center_y()
        .child(text(label.to_owned()).font(k.ty.small).color(k.c.text_muted).w(170.0))
        .child(text(value).font(k.ty.small_medium).color(k.c.text_base).grow(1.0).shrink().wrap())
}

pub const PREFS: [(&str, &str); 3] = [
    ("Tema scuro", "Usa la palette scura indipendentemente dal sistema (REPOHUB_THEME nel .env)."),
    ("Notifiche email", "Ricevi un'email per review richieste e menzioni (solo interfaccia)."),
    ("Mostra repository privati", "Includi i repository privati nel profilo pubblico (solo interfaccia)."),
];

pub fn settings(s: &App, k: &Kit) -> El {
    let p = &s.data.profile;
    let cfg = &s.cfg;
    let stars = s.starred_count();
    let profile = panel(k).padding(20.0).gap(16.0)
        .child(row().gap(14.0).center_y()
            .child(ui::user(k, if p.login.is_empty() { "?" } else { &p.login }, 56.0))
            .child(col().gap(3.0).shrink()
                .child(text(if p.name.is_empty() { "—".to_owned() } else { p.name.clone() }).font(k.ty.body_medium).color(k.c.text_strong))
                .child(text(format!("@{}{}", p.login, if p.bio.is_empty() { String::new() } else { format!(" · {}", p.bio) })).font(k.ty.small).color(k.c.text_muted).wrap())
                .child(row().gap(14.0).child(ui::stat(k, &icons::REPO, format!("{} repository", s.data.repos.len()))).child(ui::stat(k, &icons::STAR, format!("{stars} stelle"))))));
    let source = match (&s.status, cfg.live()) {
        (Status::Sample, _) if cfg.demo => "Dati di esempio (REPOHUB_DEMO=true)".to_owned(),
        (Status::Sample, _) => "Dati di esempio: nessuna configurazione trovata".to_owned(),
        (Status::Loading, _) => "Caricamento in corso…".to_owned(),
        (Status::Error(e), _) => format!("Errore: {e}"),
        (Status::Ready, _) => "GitHub, dati aggiornati".to_owned(),
    };
    let mut conn = panel(k).child(panel_head(k, "Connessione a GitHub", if cfg.live() {
        Some(Button::new("Aggiorna").icon(&icons::REFRESH).control_size(ControlSize::Small).disabled(s.status == Status::Loading)
            .on_press(|s: &mut App| s.refresh()).build(k).key("refresh-settings"))
    } else { None }));
    for (label, value) in [
        ("Stato", source),
        ("File .env", cfg.env_file.clone().unwrap_or_else(|| "non trovato".into())),
        ("Utente (GITHUB_USER)", cfg.user.clone().unwrap_or_else(|| "dal token".into())),
        ("Token (GITHUB_TOKEN)", if cfg.token.is_some() { "configurato (non mostrato)".into() } else { "assente: solo dati pubblici, 60 richieste l'ora".into() }),
        ("API", cfg.api.clone()),
        ("Fissati (REPOHUB_PINNED)", if cfg.pinned.is_empty() { "i 3 aggiornati più di recente".into() } else { cfg.pinned.join(", ") }),
        ("Filtri", format!("fork {}, archiviati {}, massimo {} repository", if cfg.include_forks { "sì" } else { "no" }, if cfg.include_archived { "sì" } else { "no" }, cfg.max_repos)),
    ] {
        conn = conn.child(rule(k)).child(info(k, label, value));
    }
    let prefs = PREFS.iter().enumerate().fold(panel(k).child(panel_head(k, "Preferenze", None)), |p, (i, (t, d))| {
        p.child(rule(k)).child(setting(k, t, d, s.prefs[i], i))
    });
    let vis = panel(k).padding(16.0).gap(10.0)
        .child(text("Visibilità predefinita dei nuovi repository").font(k.ty.ui_medium).color(k.c.text_base))
        .child(SegmentedControl::new(&["Pubblico", "Privato", "Interno"], Some(s.default_vis)).fit()
            .on_pick(|s: &mut App, i, _| s.default_vis = i).build(k).key("default-vis"));
    col().w_full().gap(16.0).max_w(760.0)
        .child(heading(k, "Impostazioni", "Profilo, connessione a GitHub e preferenze.", None))
        .child(profile)
        .child(conn)
        .child(prefs)
        .child(vis)
}

pub fn repo_settings(s: &App, k: &Kit, r: &Repo) -> El {
    let settings_url = format!("{}/settings", r.url);
    let danger = panel(k).border(1.0, k.c.danger.icon.alpha(0.5))
        .child(panel_head(k, "Zona pericolosa", None))
        .child(rule(k))
        .child(row().w_full().px(16.0).py(12.0).gap(12.0).center_y().justify_between()
            .child(col().grow(1.0).shrink().gap(2.0)
                .child(text("Archivia o elimina il repository").font(k.ty.ui_medium).color(k.c.text_base))
                .child(text("Per sicurezza RepoHub non modifica i repository: queste azioni si fanno su GitHub.").font(k.ty.text_small).color(k.c.text_muted).wrap()))
            .child(open_button(k, "Impostazioni su GitHub", &settings_url, "gh-settings")));
    let general = panel(k).child(panel_head(k, &format!("Generale · {}", r.name), None))
        .child(rule(k)).child(info(k, "Nome completo", r.full_name.clone()))
        .child(rule(k)).child(info(k, "Visibilità", if r.private { "privato".into() } else { "pubblico".into() }))
        .child(rule(k)).child(info(k, "Branch predefinito", r.branch.clone()))
        .child(rule(k)).child(info(k, "Ultimo push", r.updated.clone()));
    let _ = s;
    col().w_full().gap(16.0).child(general).child(danger.child(row().h(0.0)))
        .child(Button::new("Apri il repository su GitHub").variant(ButtonVariant::Ghost).icon(&icons::EXTERNAL).control_size(ControlSize::Small)
            .on_press_tx({ let u = r.url.clone(); move |_: &mut App, tx| { let _ = tx.open_url(&u); } }).build(k).self_start())
}
