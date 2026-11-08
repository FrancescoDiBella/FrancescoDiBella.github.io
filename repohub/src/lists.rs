//! Pull requests, issues, Actions and settings pages.
use rune::{El, col, row, text};
use rune_kit::{Kit, SegmentedControl, Switch};

use crate::data::{Checks, ISSUES, Issue, PULLS, Pull, PullState, RUNS, Repo, Run};
use crate::pages::heading;
use crate::ui::{self, panel, panel_head, rule};
use crate::{App, icons};

pub fn pull_row(k: &Kit, p: &Pull, compact: bool) -> El {
    let mut meta = row().gap(10.0).center_y().flex_wrap()
        .child(text(format!("#{} · {} · {}", p.number, p.repo, p.age)).font(k.ty.text_small).color(k.c.text_muted));
    if p.state == PullState::Draft {
        meta = meta.child(ui::pill(k, "Bozza", k.c.text_faint));
    }
    let mut r = row().w_full().px(16.0).py(11.0).gap(12.0).center_y().hover(k.c.hover)
        .child(ui::pull_icon(k, p.state))
        .child(col().grow(1.0).shrink().gap(3.0)
            .child(text(p.title.to_owned()).font(k.ty.ui_medium).color(k.c.text_base).wrap())
            .child(meta))
        .child(ui::checks(k, p.checks));
    if !compact {
        r = r.child(ui::stat(k, &icons::COMMENT, p.comments.to_string()))
            .child(ui::user(k, p.author, 22.0));
    }
    r
}

pub fn issue_row(k: &Kit, i: &Issue) -> El {
    row().w_full().px(16.0).py(11.0).gap(12.0).center_y().hover(k.c.hover)
        .child(ui::ico(&icons::ISSUE, 16.0, k.c.success.icon))
        .child(col().grow(1.0).shrink().gap(3.0)
            .child(row().gap(8.0).center_y().flex_wrap()
                .child(text(i.title.to_owned()).font(k.ty.ui_medium).color(k.c.text_base))
                .child(ui::label_chip(k, i.label, i.hue)))
            .child(text(format!("#{} · {} · aperta {}", i.number, i.repo, i.age)).font(k.ty.text_small).color(k.c.text_muted)))
}

pub fn run_row(k: &Kit, r: &Run) -> El {
    let state = match r.checks {
        Checks::Passing => ("Riuscito", k.c.success.icon),
        Checks::Failing => ("Fallito", k.c.danger.icon),
        Checks::Running => ("In corso", k.c.warning.icon),
    };
    row().w_full().px(16.0).py(11.0).gap(12.0).center_y().hover(k.c.hover)
        .child(ui::checks(k, r.checks))
        .child(col().grow(1.0).shrink().gap(3.0)
            .child(text(r.workflow.to_owned()).font(k.ty.ui_medium).color(k.c.text_base))
            .child(row().gap(10.0).center_y().flex_wrap()
                .child(text(r.repo.to_owned()).font(k.ty.text_small).color(k.c.text_muted))
                .child(ui::stat(k, &icons::BRANCH, r.branch))))
        .child(ui::pill(k, state.0, state.1))
        .child(text(format!("{} · {}", r.duration, r.age)).font(k.ty.text_small).color(k.c.text_faint).w(130.0))
}

fn listing(k: &Kit, rows: Vec<El>) -> El {
    rows.into_iter().enumerate().fold(panel(k), |p, (i, r)| if i == 0 { p.child(r) } else { p.child(rule(k)).child(r) })
}

pub const PR_FILTERS: [&str; 3] = ["Aperte", "Bozze", "Unite"];

pub fn pulls(s: &App, k: &Kit) -> El {
    let want = [PullState::Open, PullState::Draft, PullState::Merged][s.pr_filter];
    let rows: Vec<El> = PULLS.iter().filter(|p| p.state == want).map(|p| pull_row(k, p, false)).collect();
    let n = rows.len();
    col().w_full().gap(16.0)
        .child(heading(k, "Pull request", "Le PR create da te o in attesa della tua revisione.", None))
        .child(SegmentedControl::new(&PR_FILTERS, Some(s.pr_filter)).label("Stato").fit()
            .on_pick(|s: &mut App, i, _| s.pr_filter = i).build(k).key("pr-filter"))
        .child(listing(k, rows).key(format!("prs-{n}")))
}

pub fn issues(_: &App, k: &Kit) -> El {
    col().w_full().gap(16.0)
        .child(heading(k, "Issue", "Issue aperte nei tuoi repository.", None))
        .child(listing(k, ISSUES.iter().map(|i| issue_row(k, i)).collect()))
}

pub fn actions(_: &App, k: &Kit) -> El {
    let ok = RUNS.iter().filter(|r| r.checks == Checks::Passing).count();
    col().w_full().gap(16.0)
        .child(heading(k, "Actions", &format!("{ok} di {} esecuzioni recenti riuscite", RUNS.len()), None))
        .child(listing(k, RUNS.iter().map(|r| run_row(k, r)).collect()))
}

fn setting(k: &Kit, title: &str, desc: &str, on: bool, i: usize) -> El {
    row().w_full().px(16.0).py(12.0).gap(16.0).center_y().justify_between()
        .child(col().grow(1.0).shrink().gap(2.0)
            .child(text(title.to_owned()).font(k.ty.ui_medium).color(k.c.text_base))
            .child(text(desc.to_owned()).font(k.ty.text_small).color(k.c.text_muted).wrap()))
        .child(Switch::new(on).on_toggle(move |s: &mut App| s.prefs[i] = !s.prefs[i]).build(k).key(format!("pref-{i}")))
}

pub const PREFS: [(&str, &str); 5] = [
    ("Tema scuro", "Usa la palette scura indipendentemente dal sistema."),
    ("Notifiche email", "Ricevi un'email per review richieste e menzioni."),
    ("Mostra repository privati", "Includi i repository privati nel profilo pubblico (solo conteggio)."),
    ("Autenticazione a due fattori", "Richiedi un secondo fattore a ogni nuovo accesso."),
    ("Elimina i branch dopo il merge", "Cancella automaticamente il branch della PR unita."),
];

pub fn settings(s: &App, k: &Kit) -> El {
    let profile = panel(k).padding(20.0).gap(16.0)
        .child(row().gap(14.0).center_y()
            .child(ui::user(k, "francescodibella", 56.0))
            .child(col().gap(3.0)
                .child(text("Francesco Di Bella").font(k.ty.body_medium).color(k.c.text_strong))
                .child(text("@francescodibella · XR & IoT developer").font(k.ty.small).color(k.c.text_muted))
                .child(row().gap(14.0).child(ui::stat(k, &icons::REPO, "7 repository")).child(ui::stat(k, &icons::STAR, "106 stelle")))));
    let prefs = PREFS.iter().enumerate().fold(panel(k).child(panel_head(k, "Preferenze", None)), |p, (i, (t, d))| {
        p.child(rule(k)).child(setting(k, t, d, s.prefs[i], i))
    });
    let vis = panel(k).padding(16.0).gap(10.0)
        .child(text("Visibilità predefinita dei nuovi repository").font(k.ty.ui_medium).color(k.c.text_base))
        .child(SegmentedControl::new(&["Pubblico", "Privato", "Interno"], Some(s.default_vis)).fit()
            .on_pick(|s: &mut App, i, _| s.default_vis = i).build(k).key("default-vis"));
    col().w_full().gap(16.0).max_w(760.0)
        .child(heading(k, "Impostazioni", "Profilo e preferenze dell'account (solo interfaccia).", None))
        .child(profile)
        .child(prefs)
        .child(vis)
}

pub fn repo_settings(s: &App, k: &Kit, r: &Repo) -> El {
    let danger = panel(k).border(1.0, k.c.danger.icon.alpha(0.5))
        .child(panel_head(k, "Zona pericolosa", None))
        .child(rule(k))
        .child(row().w_full().px(16.0).py(12.0).gap(12.0).center_y().justify_between()
            .child(col().grow(1.0).shrink().gap(2.0)
                .child(text("Archivia questo repository").font(k.ty.ui_medium).color(k.c.text_base))
                .child(text("Diventa di sola lettura per tutti.").font(k.ty.text_small).color(k.c.text_muted)))
            .child(rune_kit::Button::new("Archivia").variant(rune_kit::ButtonVariant::Danger).control_size(rune_kit::ControlSize::Small).build(k)));
    col().w_full().gap(16.0)
        .child(panel(k).child(panel_head(k, &format!("Generale · {}", r.name), None))
            .child(rule(k)).child(setting(k, "Issue", "Abilita il tracciamento delle issue.", s.prefs[3], 3))
            .child(rule(k)).child(setting(k, "Elimina i branch dopo il merge", "Pulizia automatica dei branch.", s.prefs[4], 4)))
        .child(danger)
}
