//! Shared UI building blocks.

mod controls;
mod icons;
mod list;
mod security;
mod text;

pub use controls::*;
pub use icons::*;
pub use list::*;
pub use security::*;
pub use text::*;

use crate::i18n::{self, I18n};
use crate::prelude::*;
use crate::theme::{colors, corner_sm};
use clv_core::RiskLevel;
use gpui_kit::{Animation, AnimationExt, ease_in_out, ElementId, Stateful};
use std::time::Duration;
use gpui_kit::component::{Icon, IconName};

// ── Layout ────────────────────────────────────────────────────────────────────

/// Hover + pressed background for custom clickable surfaces (not gpui-kit Button).
pub fn surface_pressable(el: Stateful<Div>) -> Stateful<Div> {
    el.hover(|s| s.bg(colors::accent_blue_bg_hover().opacity(0.55)))
        .active(|s| s.bg(colors::accent_blue_bg_pressed().opacity(0.75)))
}

/// Vertical scroll region — place inside a `flex_1 min_h_0` parent.
pub fn scroll_y(content: impl IntoElement) -> impl IntoElement {
    div()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .overflow_y_scrollbar()
        .child(content)
}

/// Fade-in wrapper when switching main content pages.
pub fn page_transition(page_key: impl Into<ElementId>, content: impl IntoElement) -> impl IntoElement {
    div()
        .size_full()
        .min_h_0()
        .min_w_0()
        .child(content)
        .with_animation(
            page_key,
            Animation::new(Duration::from_millis(220)).with_easing(ease_in_out),
            |el, delta| el.opacity(delta),
        )
}

/// Subtle horizontal rule between panels.
pub fn panel_divider() -> Div {
    div().w_full().h(px(1.)).bg(colors::panel_divider())
}

// ── Typography ──────────────────────────────────────────────────────────────

pub fn page_title(text: impl Into<SharedString>) -> Div {
    div()
        .text_xl()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(colors::text_primary())
        .child(text.into())
}

pub fn page_subtitle(text: impl Into<SharedString>) -> Div {
    div()
        .text_base()
        .text_color(colors::text_secondary())
        .child(text.into())
}

pub fn page_header(title: impl Into<SharedString>, subtitle: impl Into<SharedString>) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(page_title(title))
        .child(page_subtitle(subtitle))
}

// ── Cards ───────────────────────────────────────────────────────────────────

pub fn card() -> Div {
    glass_card()
}

pub fn empty_state(icon: IconName, title: impl Into<SharedString>, hint: impl Into<SharedString>) -> Div {
    security::empty_state(icon, title, hint)
}

pub fn empty_state_loading(title: impl Into<SharedString>, hint: impl Into<SharedString>) -> Div {
    security::empty_state_loading(title, hint)
}

/// Animated loading spinner (rotates continuously).
pub fn loading_spinner(size: f32, color: gpui_kit::Hsla) -> gpui_kit::component::spinner::Spinner {
    gpui_kit::component::spinner::Spinner::new()
        .with_size(gpui_kit::component::Size::Size(px(size)))
        .color(color)
}

// ── Buttons ─────────────────────────────────────────────────────────────────

/// Filled primary / CTA button — white label & icon on accent background.
///
/// Uses the **built-in** `Primary` variant so the fill comes from `theme.tokens`
/// as a solid `Background`. Deliberately NOT `ButtonCustomVariant`: gpui-kit
/// mixes a custom `color` with transparency for the resting state
/// (`button.rs`: `colors.color.mix_oklab(transparent, 0.2)`), so the un-clicked
/// button shows ~80% alpha — it bleeds the page background and reads as washed
/// out / dirty, while hover & active stay fully opaque. The built-in tokens
/// have no such blend.
pub fn hero_scan_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    scanning: bool,
    cx: &App,
) -> Button {
    action_button(id, label, Some(ACTION_SCAN), true, cx).loading(scanning)
}

pub fn action_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    icon: Option<IconName>,
    primary: bool,
    _cx: &App,
) -> Button {
    let id: SharedString = id.into();
    let label: SharedString = label.into();
    let mut btn = std_button(Button::new(id).label(label));
    if let Some(name) = icon {
        let icon_color = if primary {
            colors::on_accent()
        } else {
            colors::accent_blue()
        };
        btn = btn.icon(Icon::new(name).with_size(px(20.)).text_color(icon_color));
    }
    if primary {
        // Solid accent fill straight from the theme tokens (`button_primary`),
        // no blend, no stacked border, no drop shadow.
        lg_button(btn.primary())
    } else {
        // Built-in `Default` variant: token-driven solid card fill plus the
        // theme's 1px input border.
        btn.border_1().border_color(colors::border())
    }
}

/// Hero-row secondary action: outline styling, but the **large** CTA box
/// (`BTN_H_LG`) so it sits flush with [`hero_scan_button`] in the same row.
///
/// Plain [`action_button`] uses the standard 38px control height; used next to
/// the 42px CTA the row reads as misaligned.
pub fn hero_action_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    icon: Option<IconName>,
    cx: &App,
) -> Button {
    lg_button(action_button(id, label, icon, false, cx))
}

pub fn ghost_pill(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    active: bool,
    _cx: &App,
) -> Button {
    let id: SharedString = id.into();
    let label: SharedString = label.into();
    let btn = std_button(Button::new(id).label(label)).rounded(corner_sm());
    if active {
        // Built-in `Secondary` variant gives the token-driven solid tint; the
        // accent border is drawn on the element. Avoids `ButtonCustomVariant`,
        // whose resting color is blended 20% toward transparent (see button.rs).
        btn.border_1()
            .border_color(colors::accent_blue())
            .secondary()
    } else {
        btn.ghost()
    }
}

pub fn open_path_button(id: SharedString, path: &std::path::Path, i18n: &I18n) -> Button {
    let open_path = folder_open_target(path);
    std_button(
        Button::new(id)
            .icon(Icon::new(ACTION_OPEN_FOLDER).with_size(px(20.)))
            .label(i18n.open_location())
            .ghost(),
    )
    .on_click(move |_, _, _| {
        open::that(&open_path).ok();
    })
}

/// Open a directory as-is; for a file, open its parent folder.
pub fn folder_open_target(path: &std::path::Path) -> std::path::PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf())
    }
}

// ── Badges ──────────────────────────────────────────────────────────────────

pub fn risk_badge(risk: RiskLevel, lang: clv_core::Language) -> Div {
    let (bg, border, fg) = match risk {
        RiskLevel::Safe => (
            colors::risk_safe_bg(),
            colors::risk_safe_border(),
            colors::green(),
        ),
        RiskLevel::Caution => (
            colors::risk_caution_bg(),
            colors::risk_caution_border(),
            colors::risk_caution_fg(),
        ),
        RiskLevel::Protected => (
            colors::red_bg(),
            colors::red_border(),
            colors::red(),
        ),
    };
    // Tinted background + 1px border already carry the risk level; the previous
    // 4px solid bar stacked a third redundant accent on top of it.
    h_flex()
        .items_center()
        .gap_2()
        .px(px(12.))
        .py(px(6.))
        .rounded(corner_sm())
        .bg(bg)
        .border_1()
        .border_color(border)
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(fg)
                .child(i18n::risk_label(lang, risk).to_string()),
        )
}

pub fn setting_row(label: &str, desc: &str, control: Switch) -> Div {
    card()
        .p_4()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors::text_primary())
                                .child(label.to_string()),
                        )
                        .child(
                            div()
                                .text_base()
                                .text_color(colors::text_secondary())
                                .child(desc.to_string()),
                        ),
                )
                .child(control),
        )
}
