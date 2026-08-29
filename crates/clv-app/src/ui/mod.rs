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
use gpui::{Animation, AnimationExt, ease_in_out, ElementId, Stateful};
use std::time::Duration;
use gpui_component::{
    button::ButtonCustomVariant,
    Icon, IconName,
};

// ── Layout ────────────────────────────────────────────────────────────────────

/// Hover + pressed background for custom clickable surfaces (not gpui-component Button).
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
pub fn loading_spinner(size: f32, color: gpui::Hsla) -> gpui_component::spinner::Spinner {
    gpui_component::spinner::Spinner::new()
        .with_size(gpui_component::Size::Size(px(size)))
        .color(color)
}

// ── Buttons ─────────────────────────────────────────────────────────────────

/// Filled primary / CTA button — white label & icon on accent background.
pub fn primary_button_variant(cx: &App) -> ButtonCustomVariant {
    ButtonCustomVariant::new(cx)
        .color(colors::accent_blue())
        .foreground(colors::on_accent())
        .border(colors::accent_blue())
        .hover(colors::accent_filled_hover())
        .active(colors::accent_filled_pressed())
}

/// Dashboard hero "Scan Now" — filled accent with forced white label & icon.
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
    cx: &App,
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
        lg_button(
            btn.custom(primary_button_variant(cx))
                .shadow_lg()
                .text_color(colors::on_accent()),
        )
    } else {
        btn.custom(
            ButtonCustomVariant::new(cx)
                .color(colors::bg_card())
                .foreground(colors::text_primary())
                .border(colors::border())
                .hover(colors::accent_blue_bg_hover())
                .active(colors::accent_blue_bg_pressed()),
        )
    }
}

pub fn ghost_pill(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    active: bool,
    cx: &App,
) -> Button {
    let id: SharedString = id.into();
    let label: SharedString = label.into();
    if active {
        std_button(Button::new(id).label(label))
            .rounded(corner_sm())
            .custom(
            ButtonCustomVariant::new(cx)
                .color(colors::accent_blue_bg())
                .foreground(colors::accent_blue())
                .border(colors::accent_blue())
                .hover(colors::accent_blue_bg_hover())
                .active(colors::accent_blue_bg_pressed()),
        )
    } else {
        std_button(Button::new(id).label(label))
            .rounded(corner_sm())
            .ghost()
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
                .w(px(4.))
                .h(px(14.))
                .rounded(corner_sm())
                .bg(fg),
        )
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
