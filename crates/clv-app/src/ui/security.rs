//! Security-software style visual components — lively consumer aesthetic.

use crate::app::state::AppStore;
use crate::prelude::*;
use crate::theme::{colors, corner, corner_md, corner_sm};
use crate::ui::controls::lg_button;
use crate::ui::icons::*;
use clv_core::format_bytes;
use gpui::{img, linear_color_stop, linear_gradient, Hsla};
use gpui_component::{progress::Progress, Icon, IconName};

// ── Gradients & atmosphere ───────────────────────────────────────────────────

pub fn hero_gradient() -> gpui::Background {
    linear_gradient(
        128.,
        linear_color_stop(colors::from_hex(0x1e3a5f), 0.0),
        linear_color_stop(colors::accent_blue_bg(), 0.45),
    )
}

pub fn hero_gradient_alt() -> gpui::Background {
    linear_gradient(
        145.,
        linear_color_stop(colors::from_hex(0x0f172a), 0.0),
        linear_color_stop(colors::accent_blue_bg(), 1.0),
    )
}

pub fn cta_gradient() -> gpui::Background {
    linear_gradient(
        100.,
        linear_color_stop(colors::accent_blue_bg(), 0.0),
        linear_color_stop(colors::accent_blue(), 1.0),
    )
}

pub fn warm_gradient() -> gpui::Background {
    linear_gradient(
        120.,
        linear_color_stop(colors::from_hex(0x818cf8), 0.0),
        linear_color_stop(colors::accent_blue(), 1.0),
    )
}

pub fn sidebar_gradient() -> gpui::Background {
    linear_gradient(
        175.,
        linear_color_stop(colors::from_hex(0x0c1222), 0.0),
        linear_color_stop(colors::from_hex(0x101c30), 1.0),
    )
}

pub fn content_gradient() -> gpui::Background {
    linear_gradient(
        168.,
        linear_color_stop(colors::from_hex(0x0b1120), 0.0),
        linear_color_stop(colors::from_hex(0x111b2e), 1.0),
    )
}

pub fn glow_orb_public(size: f32, color: Hsla, top: f32, right: f32) -> Div {
    glow_orb(size, color, top, right)
}

fn glow_orb(size: f32, color: Hsla, top: f32, right: f32) -> Div {
    div()
        .absolute()
        .top(px(top))
        .right(px(right))
        .size(px(size))
        .rounded_full()
        .bg(color.opacity(0.22))
}

fn glow_orb_left(size: f32, color: Hsla, bottom: f32, left: f32) -> Div {
    div()
        .absolute()
        .bottom(px(bottom))
        .left(px(left))
        .size(px(size))
        .rounded_full()
        .bg(color.opacity(0.16))
}

pub fn glass_card() -> Div {
    div()
        .rounded(corner())
        .border_1()
        .border_color(colors::from_hex(0xffffff).opacity(0.1))
        .bg(colors::from_hex(0xffffff).opacity(0.06))
        .shadow_lg()
}

pub fn soft_card() -> Div {
    div()
        .rounded(corner())
        .bg(colors::from_hex(0xffffff).opacity(0.04))
}

// ── Health score ─────────────────────────────────────────────────────────────

pub fn compute_health(store: &AppStore) -> (u8, &'static str, Hsla) {
    if store.scanning {
        return (0, "正在为你体检…", colors::accent_cyan());
    }
    let Some(report) = &store.last_report else {
        return (0, "点一下，马上知道电脑状态", colors::text_secondary());
    };
    let disk_penalty = (store.disk_used_percent() * 0.25).min(30.) as u8;
    let junk_penalty = ((report.items.len().min(60) as f32 / 60.0) * 30.0) as u8;
    let score = 100u8.saturating_sub(disk_penalty).saturating_sub(junk_penalty);
    let (msg, color) = if score >= 90 {
        ("状态很棒，继续保持", colors::safe_green())
    } else if score >= 75 {
        ("整体不错，还能更轻快", colors::accent_cyan())
    } else if score >= 55 {
        ("清理一下会更流畅", colors::from_hex(0xfbbf24))
    } else {
        ("建议尽快体检清理", colors::warn_orange())
    };
    (score, msg, color)
}

pub fn health_ring(score: u8, status: &str, accent: Hsla, scanning: bool) -> Div {
    let ring_color = if scanning {
        colors::accent_cyan()
    } else if score == 0 {
        colors::from_hex(0x64748b)
    } else {
        accent
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_3()
        .child(
            div()
                .relative()
                .size(px(156.))
                .child(
                    div()
                        .absolute()
                        .inset(px(-4.))
                        .rounded_full()
                        .bg(ring_color.opacity(0.12)),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded_full()
                        .border_4()
                        .border_color(ring_color.opacity(0.35)),
                )
                .child(
                    div()
                        .absolute()
                        .inset(px(8.))
                        .rounded_full()
                        .bg(colors::from_hex(0x0a2540).opacity(0.75))
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_1()
                        .when(scanning, |el| {
                            el.child(crate::ui::loading_spinner(40., colors::accent_cyan()))
                        })
                        .when(!scanning, |el| {
                            el.child(
                                div()
                                    .text_2xl()
                                    .text_size(px(44.))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if score > 0 {
                                        ring_color
                                    } else {
                                        colors::text_muted()
                                    })
                                    .child(if score > 0 {
                                        score.to_string()
                                    } else {
                                        "—".into()
                                    }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors::text_muted())
                                    .child("健康分"),
                            )
                        }),
                ),
        )
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors::text_primary())
                .child(status.to_string()),
        )
}

// ── Hero banner ──────────────────────────────────────────────────────────────

pub fn hero_banner(
    score: u8,
    status: &str,
    accent: Hsla,
    scanning: bool,
    scan_label: impl Into<SharedString>,
    on_scan: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    let scan_label: SharedString = scan_label.into();

    div()
        .w_full()
        .rounded(corner())
        .relative()
        .overflow_hidden()
        .bg(hero_gradient())
        .shadow_lg()
        .child(glow_orb(200., colors::accent_cyan(), -50., -30.))
        .child(glow_orb(140., colors::from_hex(0xa78bfa), 30., 60.))
        .child(glow_orb_left(100., colors::accent_blue(), -20., -30.))
        .child(
            div()
                .px_8()
                .py_7()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .gap_10()
                        .child(health_ring(score, status, accent, scanning))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap_5()
                                .child(
                                    div()
                                        .text_sm()
                                        .px(px(12.))
                                        .py(px(4.))
                                        .rounded(corner_sm())
                                        .bg(colors::accent_blue_bg())
                                        .text_color(colors::accent_blue())
                                        .child("实时守护中"),
                                )
                                .child(
                                    div()
                                        .text_2xl()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(colors::text_primary())
                                        .child("让电脑保持轻快"),
                                )
                                .child(
                                    div()
                                        .text_base()
                                        .text_color(colors::accent_blue().opacity(0.75))
                                        .child("智能找出 Agent 试验项目与开发缓存，一键释放空间"),
                                )
                                .child(
                                    h_flex()
                                        .gap_3()
                                        .items_center()
                                        .child(
                                            scan_cta_button("hero-scan", scan_label, scanning, cx)
                                                .on_click(on_scan),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(colors::accent_blue().opacity(0.85))
                                                .child("后台扫描，可继续操作其他功能"),
                                        ),
                                ),
                        ),
                ),
        )
}

pub fn scan_cta_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    disabled: bool,
    _cx: &App,
) -> Button {
    let id: SharedString = id.into();
    let label: SharedString = label.into();
    lg_button(
        Button::new(id)
            .label(label)
            .icon(
                Icon::new(ACTION_SCAN)
                    .with_size(px(18.))
                    .text_color(colors::text_primary()),
            )
            .text_size(px(16.))
            .font_weight(FontWeight::SEMIBOLD)
            .disabled(disabled)
            .primary()
            .shadow_lg(),
    )
}

// ── Quick-action tiles ───────────────────────────────────────────────────────

pub fn quick_tile(
    id: &'static str,
    title: &'static str,
    value: impl Into<SharedString>,
    hint: impl Into<SharedString>,
    tint: Hsla,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let value: SharedString = value.into();
    let hint: SharedString = hint.into();

    div()
        .id(id)
        .flex_1()
        .min_w(px(160.))
        .min_h(px(88.))
        .p_4()
        .rounded(corner())
        .border_1()
        .border_color(colors::from_hex(0xffffff).opacity(0.07))
        .bg(colors::from_hex(0xffffff).opacity(0.04))
        .cursor_pointer()
        .shadow_md()
        .on_click(on_click)
        .hover(|s| {
            s.bg(colors::from_hex(0xffffff).opacity(0.09))
                .border_color(tint.opacity(0.4))
                .shadow_lg()
        })
        .child(
            h_flex()
                .h_full()
                .gap_3()
                .child(
                    div()
                        .w(px(4.))
                        .h_full()
                        .min_h(px(56.))
                        .rounded(corner())
                        .bg(tint),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(1.))
                        .child(
                            div()
                                .text_sm()
                                .line_height(px(18.))
                                .text_color(colors::text_muted())
                                .child(title),
                        )
                        .child(
                            div()
                                .text_xl()
                                .line_height(px(26.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(colors::text_primary())
                                .child(value),
                        )
                        .child(
                            div()
                                .text_sm()
                                .line_height(px(18.))
                                .text_color(colors::text_secondary())
                                .child(hint),
                        ),
                ),
        )
}

pub fn metric_bar(label: &str, value_pct: f32, color: Hsla) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            h_flex()
                .justify_between()
                .child(
                    div()
                        .text_base()
                        .text_color(colors::text_secondary())
                        .child(label.to_string()),
                )
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(color)
                        .child(format!("{value_pct:.0}%")),
                ),
        )
        .child(
            Progress::new()
                .value(value_pct)
                .bg(color)
                .h(px(8.))
                .rounded(corner()),
        )
}

// ── Vertical icon navigation ─────────────────────────────────────────────────

const NAV_ITEM_WIDTH: f32 = 72.;
const NAV_ICON_BOX: f32 = 48.;

pub fn nav_icon(
    id: &'static str,
    icon: impl Into<Icon>,
    label: &'static str,
    active: bool,
    window: &mut Window,
    cx: &mut App,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let hovered = window.use_keyed_state(id, cx, |_, _| false);
    let is_hovered = *hovered.read(cx);
    let icon_color = if active {
        colors::accent_blue()
    } else if is_hovered {
        colors::accent_blue()
    } else {
        colors::text_muted()
    };
    let label_color = if active {
        colors::accent_blue()
    } else if is_hovered {
        colors::accent_blue()
    } else {
        colors::text_muted()
    };
    let icon = icon.into().with_size(px(24.)).text_color(icon_color);

    div()
        .id(id)
        .w(px(NAV_ITEM_WIDTH))
        .min_h(px(72.))
        .py(px(8.))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.))
        .cursor_pointer()
        .on_click(on_click)
        .on_hover({
            let hovered = hovered.clone();
            move |&h, _, cx| {
                hovered.update(cx, |state, _| *state = h);
            }
        })
        .child(
            div()
                .size(px(NAV_ICON_BOX))
                .flex_shrink_0()
                .rounded(corner_md())
                .when(active, |el| {
                    el.bg(colors::accent_blue_bg())
                        .border_1()
                        .border_color(colors::accent_blue().opacity(0.45))
                })
                .flex()
                .items_center()
                .justify_center()
                .child(icon),
        )
        .child(
            div()
                .w_full()
                .text_center()
                .text_sm()
                .line_height(px(18.))
                .font_weight(if active {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(label_color)
                .child(label),
        )
}

pub fn brand_logo(size: f32) -> impl IntoElement {
    img(ICON_APP_LOGO)
        .size(px(size))
        .rounded(corner_md())
        .shadow_lg()
}

pub fn sidebar_logo() -> Div {
    div()
        .w_full()
        .py_5()
        .flex()
        .flex_col()
        .items_center()
        .child(brand_logo(48.))
}

pub fn page_banner(title: impl Into<SharedString>, subtitle: impl Into<SharedString>) -> Div {
    h_flex()
        .w_full()
        .mb_5()
        .gap_3()
        .items_start()
        .child(
            div()
                .w(px(4.))
                .h(px(36.))
                .rounded(corner())
                .bg(colors::accent_blue())
                .mt(px(4.)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::text_primary())
                        .child(title.into()),
                )
                .child(
                    div()
                        .text_base()
                        .text_color(colors::text_secondary())
                        .child(subtitle.into()),
                ),
        )
}

pub fn empty_state_loading(title: impl Into<SharedString>, hint: impl Into<SharedString>) -> Div {
    glass_card()
        .p_12()
        .flex()
        .flex_col()
        .items_center()
        .gap_4()
        .child(crate::ui::loading_spinner(48., colors::accent_cyan().opacity(0.85)))
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors::text_secondary())
                .child(title.into()),
        )
        .child(
            div()
                .text_base()
                .text_color(colors::text_muted())
                .child(hint.into()),
        )
}

pub fn empty_state(icon: IconName, title: impl Into<SharedString>, hint: impl Into<SharedString>) -> Div {
    glass_card()
        .p_12()
        .flex()
        .flex_col()
        .items_center()
        .gap_4()
        .child(
            Icon::new(icon)
                .with_size(px(48.))
                .text_color(colors::accent_cyan().opacity(0.85)),
        )
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors::text_secondary())
                .child(title.into()),
        )
        .child(
            div()
                .text_base()
                .text_color(colors::text_muted())
                .child(hint.into()),
        )
}

/// Inline scan progress — shown while background scan is running.
pub fn scan_progress_bar(
    phase: &str,
    items_found: usize,
    bytes_found: u64,
    current_path: Option<&str>,
) -> Div {
    let pct = (items_found.min(80) as f32 / 80.0 * 100.0).min(99.0);
    let detail = if let Some(path) = current_path {
        format!(
            "{} · 已发现 {} 项（{}）\n{}",
            phase,
            items_found,
            format_bytes(bytes_found),
            path
        )
    } else {
        format!(
            "{} · 已发现 {} 项（{}）",
            phase,
            items_found,
            format_bytes(bytes_found)
        )
    };

    glass_card()
        .mb_4()
        .p_4()
        .border_color(colors::accent_cyan().opacity(0.35))
        .child(
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(crate::ui::loading_spinner(18., colors::accent_cyan()))
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(colors::text_primary())
                                        .child("正在后台扫描"),
                                ),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors::text_muted())
                                .child("可切换页面，扫描不会中断"),
                        ),
                )
                .child(
                    Progress::new()
                        .value(pct)
                        .bg(colors::accent_cyan())
                        .h(px(8.))
                        .rounded(corner()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_secondary())
                        .child(detail),
                ),
        )
}
