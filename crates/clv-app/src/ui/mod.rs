//! Shared UI building blocks.

mod controls;
mod icons;
mod list;
mod security;

pub use controls::*;
pub use icons::*;
pub use list::*;
pub use security::*;

use crate::prelude::*;
use crate::theme::{colors, corner_md, corner_sm};
use clv_core::RiskLevel;
use gpui::Hsla;
use gpui_component::{
    button::ButtonCustomVariant,
    Icon, IconName,
};

// ── Layout ────────────────────────────────────────────────────────────────────

/// Vertical scroll region — place inside a `flex_1 min_h_0` parent.
pub fn scroll_y(content: impl IntoElement) -> impl IntoElement {
    div()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .overflow_y_scrollbar()
        .child(content)
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
        btn = btn.icon(Icon::new(name).with_size(px(20.)));
    }
    if primary {
        lg_button(btn.primary())
    } else {
        btn.custom(
            ButtonCustomVariant::new(cx)
                .color(colors::bg_card())
                .foreground(colors::text_primary())
                .border(colors::border())
                .hover(colors::accent_blue_bg())
                .active(colors::accent_blue_bg()),
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
                .hover(colors::accent_blue_bg().lighten(0.08))
                .active(colors::accent_blue_bg()),
        )
    } else {
        std_button(Button::new(id).label(label))
            .rounded(corner_sm())
            .ghost()
    }
}

pub fn open_path_button(id: SharedString, path: &std::path::Path) -> Button {
    let path = path.to_path_buf();
    std_button(
        Button::new(id)
            .icon(Icon::new(ACTION_OPEN_FOLDER).with_size(px(20.)))
            .label("打开位置")
            .ghost(),
    )
    .on_click(move |_, _, _| {
        open::that(&path).ok();
    })
}

// ── Badges ──────────────────────────────────────────────────────────────────

pub fn risk_badge(risk: RiskLevel) -> Div {
    let (bg, border, fg) = match risk {
        RiskLevel::Safe => (
            colors::from_hex(0x0f2a1a),
            colors::from_hex(0x1a4d2e),
            colors::green(),
        ),
        RiskLevel::Caution => (
            colors::from_hex(0x2a2414),
            colors::from_hex(0x4d3d1a),
            colors::from_hex(0xf59e0b),
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
                .child(risk.label().to_string()),
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
            el.hover(|s| s.bg(colors::accent_blue_bg().opacity(0.55)))
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
