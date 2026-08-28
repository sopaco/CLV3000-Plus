//! Security-software style visual components — lively consumer aesthetic.

use crate::app::state::AppStore;
use crate::i18n::I18n;
use crate::prelude::*;
use crate::theme::{colors, corner, corner_md, corner_sm};
use crate::ui::icons::*;
use gpui::{img, linear_color_stop, linear_gradient, Hsla};
use gpui_component::{progress::Progress, Icon, IconName};

// ── Gradients & atmosphere ───────────────────────────────────────────────────

pub fn hero_gradient() -> gpui::Background {
    linear_gradient(
        128.,
        linear_color_stop(colors::gradient_hero_start(), 0.0),
        linear_color_stop(colors::accent_blue_bg(), 0.45),
    )
}

pub fn hero_gradient_alt() -> gpui::Background {
    linear_gradient(
        145.,
        linear_color_stop(colors::gradient_hero_alt_start(), 0.0),
        linear_color_stop(colors::accent_blue_bg(), 1.0),
    )
}

pub fn sidebar_gradient() -> gpui::Background {
    linear_gradient(
        175.,
        linear_color_stop(colors::gradient_sidebar_start(), 0.0),
        linear_color_stop(colors::gradient_sidebar_end(), 1.0),
    )
}

pub fn content_gradient() -> gpui::Background {
    linear_gradient(
        168.,
        linear_color_stop(colors::gradient_content_start(), 0.0),
        linear_color_stop(colors::gradient_content_end(), 1.0),
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
    let card = div()
        .rounded(corner())
        .border_1()
        .border_color(colors::glass_border())
        .bg(colors::glass_bg());
    if colors::is_light() {
        card.shadow_md()
    } else {
        card.shadow_lg()
    }
}

pub fn soft_card() -> Div {
    div().rounded(corner()).bg(colors::glass_bg_soft())
}

// ── Health score ─────────────────────────────────────────────────────────────

pub fn compute_health(store: &AppStore) -> (u8, String, Hsla) {
    let i18n = store.i18n();
    let scanning = store.scanning;
    // Disk stats start at 0 and would otherwise yield a fake 100.
    let disk_ready = store.disk_total > 0;
    let Some(report) = &store.last_report else {
        if scanning {
            return (0, i18n.health_scanning().to_string(), colors::accent_cyan());
        }
        return (
            0,
            i18n.health_no_scan().to_string(),
            colors::text_secondary(),
        );
    };
    if !disk_ready {
        if scanning {
            return (0, i18n.health_scanning().to_string(), colors::accent_cyan());
        }
        return (
            0,
            i18n.health_no_scan().to_string(),
            colors::text_secondary(),
        );
    }

    let disk_penalty = (store.disk_used_percent() * 0.30).min(30.) as u8;

    let reclaim_ratio = if store.disk_total > 0 {
        report.safe_reclaimable() as f32 / store.disk_total as f32
    } else {
        0.0
    };
    let junk_penalty = (reclaim_ratio * 40.0).min(40.) as u8;

    let agent_bytes: u64 = report.agent_projects.iter().map(|p| p.total_bytes).sum();
    let agent_ratio = if store.disk_total > 0 {
        agent_bytes as f32 / store.disk_total as f32
    } else {
        0.0
    };
    let agent_penalty = (agent_ratio * 40.0).min(20.) as u8;

    let startup_penalty = ((store.startup_count.min(20) as f32 / 20.0) * 10.0).round() as u8;

    let score = 100u8
        .saturating_sub(disk_penalty)
        .saturating_sub(junk_penalty)
        .saturating_sub(agent_penalty)
        .saturating_sub(startup_penalty);
    let (msg, color) = if scanning {
        (i18n.health_scanning(), colors::accent_cyan())
    } else if score >= 90 {
        (i18n.health_excellent(), colors::safe_green())
    } else if score >= 75 {
        (i18n.health_good(), colors::accent_cyan())
    } else if score >= 55 {
        (i18n.health_fair(), colors::from_hex(0xfbbf24))
    } else {
        (i18n.health_poor(), colors::warn_orange())
    };
    (score, msg.to_string(), color)
}

pub fn health_ring(
    score: u8,
    status: impl Into<SharedString>,
    accent: Hsla,
    scanning: bool,
    i18n: &I18n,
) -> Div {
    let status: SharedString = status.into();
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
                        .bg(colors::ring_center_bg())
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
                                    .child(i18n.health_score_label()),
                            )
                        }),
                ),
        )
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors::text_primary())
                .child(status),
        )
}

// ── Hero banner ──────────────────────────────────────────────────────────────

pub fn hero_banner(
    score: u8,
    status: impl Into<SharedString>,
    accent: Hsla,
    scanning: bool,
    scan_label: impl Into<SharedString>,
    on_scan: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    reclaim_summary: Option<impl Into<SharedString>>,
    show_view_details: bool,
    details_label: impl Into<SharedString>,
    on_view_details: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    i18n: &I18n,
    cx: &App,
) -> impl IntoElement {
    let scan_label: SharedString = scan_label.into();
    let details_label: SharedString = details_label.into();
    let reclaim_summary: Option<SharedString> = reclaim_summary.map(Into::into);

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
            div().px_8().py_7().child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .gap_10()
                    .child(health_ring(score, status, accent, scanning, i18n))
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
                                    .child(i18n.realtime_guard()),
                            )
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(colors::text_primary())
                                    .child(i18n.hero_title()),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::accent_blue().opacity(0.75))
                                    .child(i18n.hero_subtitle()),
                            )
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .flex_wrap()
                                    .child(
                                        crate::ui::hero_scan_button(
                                            "hero-scan",
                                            scan_label,
                                            scanning,
                                            cx,
                                        )
                                        .on_click(on_scan),
                                    )
                                    .when(show_view_details, |row| {
                                        row.child(
                                            crate::ui::action_button(
                                                "hero-view-details",
                                                details_label,
                                                Some(crate::ui::icons::ACTION_OPEN_FOLDER),
                                                false,
                                                cx,
                                            )
                                            .on_click(on_view_details),
                                        )
                                    })
                                    .when(reclaim_summary.is_some(), |row| {
                                        let summary = reclaim_summary.clone().unwrap();
                                        row.child(
                                            div()
                                                .text_sm()
                                                .text_color(colors::accent_blue().opacity(0.9))
                                                .child(summary),
                                        )
                                    })
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors::accent_blue().opacity(0.85))
                                            .child(i18n.hero_scan_hint()),
                                    ),
                            ),
                    ),
            ),
        )
}

pub fn scan_cta_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    scanning: bool,
    cx: &App,
) -> Button {
    crate::ui::hero_scan_button(id, label, scanning, cx)
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
        .border_color(colors::glass_border())
        .bg(colors::glass_bg_soft())
        .cursor_pointer()
        .shadow_md()
        .on_click(on_click)
        .hover(|s| {
            s.bg(colors::glass_bg_hover())
                .border_color(tint.opacity(0.4))
                .shadow_lg()
        })
        .active(|s| {
            s.bg(colors::glass_bg_active())
                .border_color(tint.opacity(0.55))
                .shadow_sm()
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
    let icon_box_bg = if active {
        colors::accent_blue_bg()
    } else if is_hovered {
        colors::accent_blue_bg_hover().opacity(0.55)
    } else {
        Hsla::transparent_black()
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
        .rounded(corner_md())
        .on_click(on_click)
        .when(!active, |el| {
            el.hover(|s| s.bg(colors::accent_blue_bg_hover().opacity(0.25)))
                .active(|s| s.bg(colors::accent_blue_bg_pressed().opacity(0.45)))
        })
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
                .bg(icon_box_bg)
                .when(active, |el| {
                    el.border_1()
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

pub fn sidebar_logo() -> impl IntoElement {
    div()
        .id("sidebar-logo")
        .w_full()
        .py_5()
        .flex()
        .flex_col()
        .items_center()
        .cursor_pointer()
        .on_click(|_, _, _| {
            let _ = open::that("https://github.com/sopaco/CLV3000-Plus/releases");
        })
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
        .child(crate::ui::loading_spinner(
            48.,
            colors::accent_cyan().opacity(0.85),
        ))
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

pub fn empty_state(
    icon: IconName,
    title: impl Into<SharedString>,
    hint: impl Into<SharedString>,
) -> Div {
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

fn inline_progress_bar(
    title: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    pct: f32,
    accent: Hsla,
    cancel_id: &'static str,
    cancel_label: impl Into<SharedString>,
    on_cancel: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> Div {
    glass_card()
        .mb_4()
        .p_4()
        .border_color(accent.opacity(0.35))
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
                                .child(crate::ui::loading_spinner(18., accent))
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(colors::text_primary())
                                        .child(title.into()),
                                ),
                        )
                        .child(
                            crate::ui::ghost_pill(cancel_id, cancel_label, false, cx)
                                .on_click(on_cancel),
                        ),
                )
                .child(
                    Progress::new()
                        .value(pct)
                        .bg(accent)
                        .h(px(8.))
                        .rounded(corner()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_secondary())
                        .child(detail.into()),
                ),
        )
}

/// Inline scan progress — shown while background scan is running.
pub fn scan_progress_bar(
    i18n: &I18n,
    phase: &str,
    items_found: usize,
    bytes_found: u64,
    current_path: Option<&str>,
    on_cancel: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> Div {
    let pct = (items_found.min(80) as f32 / 80.0 * 100.0).min(99.0);
    inline_progress_bar(
        i18n.fast_scanning(),
        i18n.scan_bar_detail(phase, items_found, bytes_found, current_path),
        pct,
        colors::accent_cyan(),
        "scan-cancel",
        i18n.cancel(),
        on_cancel,
        cx,
    )
}

/// Inline cleanup progress — shown while background cleanup is running.
pub fn cleanup_progress_bar(
    i18n: &I18n,
    completed: usize,
    total: usize,
    freed_bytes: u64,
    current_path: Option<&str>,
    on_cancel: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> Div {
    let pct = if total == 0 {
        0.0
    } else {
        (completed as f32 / total as f32 * 100.0).min(99.0)
    };
    inline_progress_bar(
        i18n.cleanup_progress_title(),
        i18n.cleanup_progress_detail(completed, total, freed_bytes, current_path),
        pct,
        colors::accent_blue(),
        "cleanup-cancel",
        i18n.cancel(),
        on_cancel,
        cx,
    )
}
