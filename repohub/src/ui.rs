//! Small building blocks shared by the pages.
use rune::{Color, El, Icon, TextStyle, col, el, icon, oklch, row, text};

pub const H1: TextStyle = TextStyle::new(24.0, 650.0);
pub const H2: TextStyle = TextStyle::new(21.0, 620.0);
use rune_kit::Kit;

use crate::model::{Checks, PullState};
use crate::icons;

/// A rounded panel on the page background.
pub fn panel(k: &Kit) -> El {
    col().w_full().radius(k.radius.lg).fill(k.c.bg_base).border(1.0, k.c.border_weak)
}

/// A panel's title row: a label on the left, anything on the right.
pub fn panel_head(k: &Kit, title: &str, trailing: Option<El>) -> El {
    let mut r = row().w_full().h(44.0).px(16.0).center_y().justify_between()
        .child(text(title.to_owned()).font(k.ty.ui_medium).color(k.c.text_base));
    if let Some(t) = trailing {
        r = r.child(t);
    }
    r
}

pub fn rule(k: &Kit) -> El {
    el().w_full().h(1.0).fill(k.c.border_weak)
}

pub fn ico(i: &'static Icon, size: f32, c: Color) -> El {
    icon(i, size).color(c)
}

/// The language swatch and name of a repository.
pub fn language(k: &Kit, name: &str, hue: f32) -> El {
    row().gap(6.0).center_y()
        .child(el().size(10.0, 10.0).radius(5.0).fill(oklch(0.68, 0.15, hue)))
        .child(text(name.to_owned()).font(k.ty.text_small).color(k.c.text_muted))
}

/// An icon followed by a number or short label, in muted ink.
pub fn stat(k: &Kit, i: &'static Icon, label: impl Into<String>) -> El {
    row().gap(4.0).center_y()
        .child(ico(i, 14.0, k.c.icon_muted))
        .child(text(label.into()).font(k.ty.text_small).color(k.c.text_muted))
}

/// A rounded tag in the accent tint, as GitHub shows topics.
pub fn topic(k: &Kit, name: &str) -> El {
    text(name.to_owned()).font(k.ty.text_xsmall_medium).color(k.c.text_accent)
        .px(9.0).py(3.0).radius(10.0).fill(k.c.bg_accent.alpha(0.14))
}

/// A coloured label chip (issue labels).
pub fn label_chip(k: &Kit, name: &str, hue: f32) -> El {
    text(name.to_owned()).font(k.ty.text_xsmall_medium).color(oklch(0.72, 0.13, hue))
        .px(8.0).py(2.0).radius(10.0).fill(oklch(0.62, 0.12, hue).alpha(0.16)).border(1.0, oklch(0.62, 0.12, hue).alpha(0.45))
}

/// A square avatar with the first letter of `name`.
pub fn avatar(k: &Kit, name: &str, hue: f32, size: f32) -> El {
    let initial = name.chars().find(|c| c.is_alphanumeric()).unwrap_or('?').to_ascii_uppercase().to_string();
    row().size(size, size).radius(size * 0.28).center().fill(oklch(0.6, 0.13, hue))
        .child(text(initial).font(if size > 24.0 { k.ty.body_medium } else { k.ty.small_medium }).color(Color::WHITE))
}

/// A round user avatar.
pub fn user(k: &Kit, name: &str, size: f32) -> El {
    let hue = name.bytes().fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32)) % 360;
    avatar(k, name, hue as f32, size).radius(size / 2.0)
}

/// The CI state as a coloured icon.
pub fn checks(k: &Kit, c: Checks) -> El {
    match c {
        Checks::Passing => ico(&icons::CHECK, 14.0, k.c.success.icon),
        Checks::Failing => ico(&icons::CROSS, 14.0, k.c.danger.icon),
        Checks::Running => ico(&icons::DOT, 14.0, k.c.warning.icon),
    }
}

/// The pull request state icon.
pub fn pull_icon(k: &Kit, s: PullState) -> El {
    match s {
        PullState::Open => ico(&icons::PULL, 16.0, k.c.success.icon),
        PullState::Draft => ico(&icons::PULL, 16.0, k.c.icon_faint),
        PullState::Merged => ico(&icons::MERGE, 16.0, oklch(0.62, 0.17, 300.0)),
        PullState::Closed => ico(&icons::PULL, 16.0, k.c.danger.icon),
    }
}

/// A rounded pill with text, for states such as "Public" or "Draft".
pub fn pill(k: &Kit, label: &str, ink: Color) -> El {
    text(label.to_owned()).font(k.ty.text_xsmall_medium).color(ink)
        .px(8.0).py(2.0).radius(10.0).border(1.0, ink.alpha(0.5))
}

/// Weekly commit bars, newest on the right.
pub fn sparkline(k: &Kit, values: &[u32], height: f32) -> El {
    let max = values.iter().copied().max().unwrap_or(1).max(1) as f32;
    let n = values.len();
    let bars = values.iter().enumerate().map(|(i, v)| {
        let h = (*v as f32 / max * height).max(2.0);
        let a = 0.35 + 0.65 * (i as f32 / (n.max(2) - 1) as f32);
        col().grow(1.0).h(height).justify_end()
            .child(el().w_full().h(h).radius(2.0).fill(k.c.success.icon.alpha(a)))
    });
    row().w_full().h(height).gap(3.0).children(bars)
}

/// A big number with a caption under it.
pub fn metric(k: &Kit, i: &'static Icon, value: &str, caption: &str, tint: Color) -> El {
    panel(k).grow(1.0).min_w(150.0).padding(16.0).gap(10.0)
        .child(row().size(30.0, 30.0).radius(8.0).center().fill(tint.alpha(0.15)).child(ico(i, 16.0, tint)))
        .child(text(value.to_owned()).font(H2).color(k.c.text_strong))
        .child(text(caption.to_owned()).font(k.ty.text_small).color(k.c.text_muted))
}
