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
use crate::theme::{colors, corner_md, corner_sm};
use clv_core::RiskLevel;
use gpui::{Animation, AnimationExt, ease_in_out, ElementId, Hsla, Stateful};
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

pub fn stat_card(title: &str, value: &str, subtitle: &str, accent: Hsla) -> Div {
    card()
        .flex_1()
        .min_w(px(170.))
        .p_4()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            h_flex()
                .justify_between()
                .items_start()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_secondary())
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .w(px(8.))
                        .h(px(8.))
                        .rounded(corner_sm())
                        .bg(accent),
                ),
        )
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .text_color(colors::text_primary())
                .child(value.to_string()),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors::text_muted())
                .child(subtitle.to_string()),
        )
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
            colors::text_primary()
        } else {
            colors::accent_blue()
        };
        btn = btn.icon(Icon::new(name).with_size(px(20.)).text_color(icon_color));
    }
    if primary {
        lg_button(btn.primary())
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
    let path = path.to_path_buf();
    std_button(
        Button::new(id)
            .icon(Icon::new(ACTION_OPEN_FOLDER).with_size(px(20.)))
            .label(i18n.open_location())
            .ghost(),
    )
    .on_click(move |_, _, _| {
        open::that(&path).ok();
    })
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

// ── Navigation ──────────────────────────────────────────────────────────────

pub fn nav_item(
    id: &'static str,
    icon: IconName,
    label: &'static str,
    active: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let icon_color = if active {
        colors::accent_blue()
    } else {
        colors::text_secondary()
    };
    let text_color = if active {
        colors::text_primary()
    } else {
        colors::text_secondary()
    };
    let bg = if active {
        colors::accent_blue_bg()
    } else {
        Hsla::transparent_black()
    };

    div()
        .id(id)
        .w_full()
        .h(px(40.))
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .rounded(corner_md())
        .bg(bg)
        .cursor_pointer()
        .on_click(on_click)
        .when(!active, |el| {
            surface_pressable(el)
        })
        .child(
            div()
                .size(px(40.))
                .flex()
                .flex_shrink_0()
                .items_center()
                .justify_center()
                .child(Icon::new(icon).with_size(px(21.)).text_color(icon_color)),
        )
        .child(
            div()
                .text_base()
                .font_weight(if active {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(text_color)
                .child(label),
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
