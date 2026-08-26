use crate::app::state::{AppPage, AppStore};
use crate::i18n::I18n;
use crate::prelude::*;
use crate::theme::{colors, corner};
use clv_core::{format_bytes, CleanupHistory};
use gpui::Hsla;

pub struct DashboardView {
    store: Entity<AppStore>,
}

impl DashboardView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self { store }
    }
}

impl Render for DashboardView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let i18n = store.i18n();
        let reclaimable = store
            .last_report
            .as_ref()
            .map(|r| r.total_reclaimable_human())
            .unwrap_or_else(|| "—".into());
        let item_count = store
            .last_report
            .as_ref()
            .map(|r| r.items.len())
            .unwrap_or(0);
        let agent_count = store
            .last_report
            .as_ref()
            .map(|r| r.agent_projects.len())
            .unwrap_or(0);
        let agent_bytes = store
            .last_report
            .as_ref()
            .map(|r| {
                format_bytes(
                    r.agent_projects
                        .iter()
                        .map(|p| p.total_bytes)
                        .sum::<u64>(),
                )
            })
            .unwrap_or_else(|| "—".into());

        let disk_pct = store.disk_used_percent();
        let disk_used = format_bytes(store.disk_used);
        let disk_total = format_bytes(store.disk_total);
        let disk_free = format_bytes(store.disk_free());

        let scanning = store.scanning;
        let store_entity = self.store.clone();
        let (score, status, accent) = ui::compute_health(store);
        let scan_label = if scanning {
            i18n.scanning_health()
        } else {
            i18n.scan_now()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .child(ui::scroll_y(
                div()
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(ui::hero_banner(
                        score,
                        status,
                        accent,
                        scanning,
                        scan_label,
                        {
                            let store = store_entity.clone();
                            move |_, _, cx| {
                                store.update(cx, |s, cx| s.start_scan(cx));
                            }
                        },
                        &i18n,
                        cx,
                    ))
                    .child(
                        h_flex()
                            .gap_3()
                            .flex_wrap()
                            .child(ui::quick_tile(
                                "tile-disk",
                                i18n.disk_usage(),
                                format!("{disk_pct:.0}%"),
                                format!("{disk_used} / {disk_total}"),
                                colors::accent_cyan(),
                                |_, _, _| {},
                            ))
                            .child(ui::quick_tile(
                                "tile-clean",
                                i18n.reclaimable_space(),
                                reclaimable.clone(),
                                i18n.cleanable_items_count(item_count),
                                colors::safe_green(),
                                {
                                    let store = store_entity.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| {
                                            s.set_page(AppPage::Cleanup, cx);
                                        });
                                    }
                                },
                            ))
                            .child(ui::quick_tile(
                                "tile-agent",
                                i18n.agent_projects_tile(),
                                agent_count.to_string(),
                                i18n.approx_size(&agent_bytes),
                                colors::accent_secondary(),
                                {
                                    let store = store_entity.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| {
                                            s.set_page(AppPage::Agent, cx);
                                        });
                                    }
                                },
                            ))
                            .child(ui::quick_tile(
                                "tile-startup",
                                i18n.startup_items(),
                                store.startup_count.to_string(),
                                i18n.manage_startup(),
                                colors::warn_orange(),
                                {
                                    let store = store_entity.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| {
                                            s.set_page(AppPage::Startup, cx);
                                        });
                                    }
                                },
                            )),
                    )
                    .child(
                        ui::glass_card()
                            .p_5()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::text_primary())
                                    .child(i18n.system_status()),
                            )
                            .child(ui::metric_bar(i18n.disk_usage_metric(), disk_pct, colors::accent_cyan()))
                            .child(
                                ui::metric_bar(
                                    i18n.optimizable_space(),
                                    if item_count > 0 {
                                        ((item_count.min(50) as f32 / 50.0) * 100.0).min(100.)
                                    } else {
                                        0.
                                    },
                                    colors::safe_green(),
                                ),
                            )
                            .child(
                                h_flex()
                                    .justify_between()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors::text_secondary())
                                            .child(i18n.free_space(&disk_free)),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors::text_muted())
                                            .child(if store.settings.expert_mode {
                                                i18n.expert_mode_short()
                                            } else {
                                                i18n.simple_mode_short()
                                            }),
                                    ),
                            ),
                    )
                    .child(
                        ui::glass_card()
                            .p_5()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::text_primary())
                                    .child(i18n.protection_features_title()),
                            )
                            .child(feature_line(i18n.feature_agent_detect()))
                            .child(feature_line(i18n.feature_safe_cleanup()))
                            .child(feature_line(i18n.feature_startup()))
                            .child(feature_line(i18n.feature_process())),
                    )
                    .child(history_card(&store.cleanup_history, &i18n)),
            ))
    }
}

fn history_card(history: &CleanupHistory, i18n: &I18n) -> Div {
    let freed_7d = history.freed_in_days(7);
    let freed_30d = history.freed_in_days(30);
    let cleanups_7d = history.cleanup_count_in_days(7);
    let cleanups_30d = history.cleanup_count_in_days(30);
    let success_7d = history.success_count_in_days(7);
    let failed_7d = history.failed_count_in_days(7);

    let has_data = !history.records.is_empty();

    let subtitle = if has_data {
        i18n.history_summary(&format_bytes(freed_7d), cleanups_7d)
    } else {
        i18n.no_cleanup_history().to_string()
    };

    let mut card = ui::glass_card()
        .p_5()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::text_primary())
                .child(i18n.cleanup_history_title()),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors::text_secondary())
                .child(subtitle),
        );

    if has_data {
        card = card.child(
            h_flex()
                .gap_4()
                .child(history_stat_block(
                    i18n.history_7d_freed(),
                    &format_bytes(freed_7d),
                    i18n.history_cleanups_count(cleanups_7d),
                    colors::safe_green(),
                ))
                .child(history_stat_block(
                    i18n.history_30d_freed(),
                    &format_bytes(freed_30d),
                    i18n.history_cleanups_count(cleanups_30d),
                    colors::accent_cyan(),
                )),
        );

        if success_7d > 0 || failed_7d > 0 {
            card = card.child(
                div()
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(i18n.history_7d_detail(success_7d, failed_7d)),
            );
        }
    }

    card
}

fn history_stat_block(title: &str, value: &str, sub: String, color: Hsla) -> Div {
    v_flex()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(colors::text_muted())
                .child(title.to_string()),
        )
        .child(
            div()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .text_color(color)
                .child(value.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors::text_secondary())
                .child(sub),
        )
}

fn feature_line(text: &str) -> Div {
    h_flex()
        .gap_3()
        .items_center()
        .child(
            div()
                .w(px(3.))
                .h(px(16.))
                .rounded(corner())
                .bg(colors::accent_cyan()),
        )
        .child(
            div()
                .text_base()
                .text_color(colors::text_secondary())
                .child(text.to_string()),
        )
}
