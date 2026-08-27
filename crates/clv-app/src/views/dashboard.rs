use crate::app::state::{AppPage, AppStore};
use crate::i18n::I18n;
use crate::prelude::*;
use crate::theme::{colors, corner};
use clv_core::{format_bytes, CleanupBucket, CleanupHistory};
use clv_platform::DiskVolume;
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
        let safe_count = store
            .last_report
            .as_ref()
            .map(|r| r.safe_item_count())
            .unwrap_or(0);
        let safe_bytes = store
            .last_report
            .as_ref()
            .map(|r| format_bytes(r.safe_reclaimable()))
            .unwrap_or_else(|| "—".into());
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
        let bucket_summaries = store
            .last_report
            .as_ref()
            .map(|r| r.bucket_summaries())
            .unwrap_or_default();
        let large_file_count = store
            .last_report
            .as_ref()
            .map(|r| r.large_files.len())
            .unwrap_or(0);

        let disk_pct = store.disk_used_percent();
        let disk_used = format_bytes(store.disk_used);
        let disk_total = format_bytes(store.disk_total);
        let disk_free = format_bytes(store.disk_free());

        let scanning = store.scanning;
        let cleaning = store.cleaning;
        let has_report = store.last_report.is_some();
        let show_clean_safe = has_report && safe_count > 0 && !scanning;
        let reclaim_summary = if has_report && safe_count > 0 {
            Some(i18n.reclaim_safe_summary(&safe_bytes, safe_count))
        } else {
            None
        };

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
                        reclaim_summary,
                        show_clean_safe,
                        cleaning,
                        i18n.clean_safe_items(),
                        {
                            let store = store_entity.clone();
                            move |_, window, cx| {
                                let (count, bytes, i18n) = {
                                    let s = store.read(cx);
                                    let ids = s
                                        .last_report
                                        .as_ref()
                                        .map(|r| clv_core::default_selected_item_ids(&r.items))
                                        .unwrap_or_default();
                                    let bytes = s
                                        .last_report
                                        .as_ref()
                                        .map(|r| {
                                            r.items
                                                .iter()
                                                .filter(|i| ids.contains(&i.id))
                                                .map(|i| i.size_bytes)
                                                .sum::<u64>()
                                        })
                                        .unwrap_or(0);
                                    (ids.len(), bytes, s.i18n())
                                };
                                let store_ok = store.clone();
                                window.open_dialog(cx, move |dialog, _, _| {
                                    dialog
                                        .title(i18n.confirm_cleanup_title())
                                        .child(i18n.confirm_cleanup_body(count, bytes))
                                        .confirm()
                                        .on_ok({
                                            let store = store_ok.clone();
                                            move |_, _, cx| {
                                                store.update(cx, |s, cx| {
                                                    s.run_cleanup_safe(cx);
                                                });
                                                true
                                            }
                                        })
                                });
                            }
                        },
                        {
                            let store = store_entity.clone();
                            move |_, _, cx| {
                                store.update(cx, |s, cx| {
                                    s.set_page(AppPage::Cleanup, cx);
                                });
                            }
                        },
                        i18n.view_cleanup_details(),
                        &i18n,
                        cx,
                    ))
                    .when(!bucket_summaries.is_empty(), |col| {
                        col.child(category_summary_card(
                            &bucket_summaries,
                            &i18n,
                            store_entity.clone(),
                            cx,
                        ))
                    })
                    .when(store.disk_volumes.len() > 1, |col| {
                        col.child(disk_volumes_card(&store.disk_volumes, &i18n))
                    })
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
                                "tile-large-files",
                                i18n.large_files_tile(),
                                large_file_count.to_string(),
                                i18n.large_files_tile_hint(),
                                colors::warn_orange(),
                                {
                                    let store = store_entity.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| {
                                            s.set_page(AppPage::LargeFiles, cx);
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
                    .child(system_actions_card(
                        store.system_trash_bytes,
                        store_entity.clone(),
                        &i18n,
                        cx,
                    ))
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
                                    if store.disk_total > 0 {
                                        ((store
                                            .last_report
                                            .as_ref()
                                            .map(|r| r.safe_reclaimable())
                                            .unwrap_or(0) as f32
                                            / store.disk_total as f32)
                                            * 100.0)
                                            .min(100.)
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
                    .child(history_card(&store.cleanup_history, store_entity.clone(), &i18n, cx)),
            ))
    }
}

fn category_summary_card(
    summaries: &[(CleanupBucket, u64, usize, usize)],
    i18n: &I18n,
    store: Entity<AppStore>,
    cx: &App,
) -> Div {
    let mut card = ui::glass_card()
        .p_5()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::text_primary())
                .child(i18n.category_summary_title()),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors::text_secondary())
                .child(i18n.category_summary_hint()),
        );

    for (index, (bucket, bytes, total, safe)) in summaries.iter().enumerate() {
        let bucket = *bucket;
        let bytes_label = format_bytes(*bytes);
        let filter = bucket_to_filter(bucket);
        let store_btn = store.clone();
        let id = SharedString::from(format!("cat-clean-{index}"));
        card = card.child(
            h_flex()
                .justify_between()
                .items_center()
                .gap_3()
                .p_3()
                .rounded(corner())
                .bg(colors::glass_bg_soft())
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors::text_primary())
                                .child(i18n.cleanup_bucket_label(bucket)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors::text_muted())
                                .child(i18n.category_bucket_meta(*total, *safe)),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::safe_green())
                                .child(bytes_label),
                        )
                        .when(*safe > 0, |row| {
                            row.child(
                                ui::ghost_pill(id, i18n.clean_category(), false, cx).on_click({
                                    let store = store_btn.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| {
                                            s.cleanup_filter = filter;
                                            s.set_page(AppPage::Cleanup, cx);
                                        });
                                    }
                                }),
                            )
                        }),
                ),
        );
    }

    card
}

fn bucket_to_filter(bucket: CleanupBucket) -> crate::app::state::CleanupFilter {
    match bucket {
        CleanupBucket::ProjectBuildCache => crate::app::state::CleanupFilter::ProjectBuildCache,
        CleanupBucket::SharedToolCache => crate::app::state::CleanupFilter::SharedToolCache,
        CleanupBucket::DevEnvironment => crate::app::state::CleanupFilter::DevEnvironment,
        CleanupBucket::AiGenerated => crate::app::state::CleanupFilter::AiGenerated,
    }
}

fn disk_volumes_card(volumes: &[DiskVolume], i18n: &I18n) -> Div {
    let mut card = ui::glass_card()
        .p_5()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::text_primary())
                .child(i18n.disk_volumes_title()),
        );

    for volume in volumes {
        let label = volume.label.clone();
        let used = format_bytes(volume.used_bytes);
        let total = format_bytes(volume.total_bytes);
        let free = format_bytes(volume.free_bytes());
        card = card.child(
            v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .justify_between()
                        .child(
                            div()
                                .text_color(colors::text_primary())
                                .child(label),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors::text_secondary())
                                .child(format!("{used} / {total}")),
                        ),
                )
                .child(ui::metric_bar(
                    &i18n.free_space(&free),
                    volume.used_percent(),
                    colors::accent_cyan(),
                )),
        );
    }

    card
}

fn system_actions_card(
    trash_bytes: Option<u64>,
    store: Entity<AppStore>,
    i18n: &I18n,
    cx: &App,
) -> Div {
    let trash_label = trash_bytes
        .map(|b| i18n.system_trash_size(&format_bytes(b)))
        .unwrap_or_else(|| i18n.system_trash_unknown().to_string());

    ui::glass_card()
        .p_5()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::text_primary())
                .child(i18n.system_actions_title()),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors::text_secondary())
                .child(i18n.system_trash_desc()),
        )
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_base()
                        .text_color(colors::text_primary())
                        .child(trash_label),
                )
                .child(
                    ui::action_button("empty-trash", i18n.empty_system_trash(), None, false, cx)
                        .on_click({
                            let store = store.clone();
                            move |_, window, cx| {
                                let i18n = store.read(cx).i18n();
                                let store_ok = store.clone();
                                window.open_dialog(cx, move |dialog, _, _| {
                                    dialog
                                        .title(i18n.confirm_empty_trash_title())
                                        .child(i18n.confirm_empty_trash_body())
                                        .confirm()
                                        .on_ok({
                                            let store = store_ok.clone();
                                            move |_, _, cx| {
                                                store.update(cx, |s, cx| {
                                                    s.empty_system_trash_async(cx);
                                                });
                                                true
                                            }
                                        })
                                });
                            }
                        }),
                ),
        )
}

fn history_card(
    history: &CleanupHistory,
    store: Entity<AppStore>,
    i18n: &I18n,
    cx: &App,
) -> Div {
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

    let restorable = history.restorable_entries();
    if !restorable.is_empty() {
        card = card.child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors::text_primary())
                .child(i18n.recently_trashed_title()),
        );
        for (index, entry) in restorable.into_iter().take(8).enumerate() {
            let name = entry.name.clone();
            let store_btn = store.clone();
            let id = SharedString::from(format!("restore-{index}"));
            card = card.child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_sm()
                            .text_color(colors::text_secondary())
                            .truncate()
                            .child(name.clone()),
                    )
                    .child(
                        ui::ghost_pill(id, i18n.restore(), false, cx).on_click({
                            let store = store_btn;
                            move |_, _, cx| {
                                store.update(cx, |s, cx| {
                                    s.restore_trashed_entry(entry.clone(), cx);
                                });
                            }
                        }),
                    ),
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
