use crate::app::state::{AppPage, AppStore};
use crate::i18n::I18n;
use crate::prelude::*;
use crate::theme::{colors, corner};
use clv_core::{format_bytes, CleanupHistory};
use clv_platform::{list_disk_volumes, DiskVolume};
use gpui::Hsla;

enum VolumesDialogState {
    Loading,
    Ready(Vec<DiskVolume>),
}

struct DiskVolumesDialog {
    i18n: I18n,
    state: VolumesDialogState,
}

impl DiskVolumesDialog {
    fn new(i18n: I18n, cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |weak, cx| {
            let volumes = cx
                .background_spawn(async { list_disk_volumes() })
                .await;
            weak.update(cx, |this, cx| {
                this.state = VolumesDialogState::Ready(volumes);
                cx.notify();
            })
            .ok();
        })
        .detach();
        Self {
            i18n,
            state: VolumesDialogState::Loading,
        }
    }
}

impl Render for DiskVolumesDialog {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let i18n = self.i18n;
        v_flex()
            .min_h(px(140.))
            .gap_4()
            .child(match &self.state {
                VolumesDialogState::Loading => v_flex()
                    .gap_3()
                    .items_center()
                    .justify_center()
                    .py_6()
                    .child(ui::loading_spinner(32., colors::accent_cyan()))
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::text_secondary())
                            .child(i18n.disk_volumes_loading()),
                    ),
                VolumesDialogState::Ready(volumes) if volumes.is_empty() => div()
                    .text_sm()
                    .text_color(colors::text_secondary())
                    .child(i18n.disk_volumes_empty()),
                VolumesDialogState::Ready(volumes) => disk_volumes_list(volumes, &i18n),
            })
            .child(
                div()
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(i18n.press_esc_to_close()),
            )
    }
}

pub struct DashboardView {
    store: Entity<AppStore>,
    recently_deleted_expanded: bool,
}

impl DashboardView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self {
            store,
            recently_deleted_expanded: false,
        }
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
        let large_file_count = store
            .last_report
            .as_ref()
            .map(|r| r.large_files.len())
            .unwrap_or(0);

        let disk_pct = store.disk_used_percent();
        let disk_ready = store.disk_total > 0;
        let disk_used = format_bytes(store.disk_used);
        let disk_total = format_bytes(store.disk_total);
        let disk_free = format_bytes(store.disk_free());
        let disk_tile_value = if disk_ready {
            format!("{disk_pct:.0}%")
        } else {
            "—".into()
        };
        let disk_tile_hint = if disk_ready {
            format!("{disk_used} / {disk_total}")
        } else {
            "—".into()
        };

        let scanning = store.scanning;
        let has_report = store.last_report.is_some();
        let show_view_details = has_report && !scanning;
        let reclaim_summary = if has_report && safe_count > 0 {
            Some(i18n.reclaim_safe_summary(&safe_bytes, safe_count))
        } else {
            None
        };

        let store_entity = self.store.clone();
        let dashboard = cx.entity();
        let (score, status, accent) = ui::compute_health(store);
        let scan_label = if scanning {
            i18n.scanning_health()
        } else {
            i18n.scan_now()
        };

        div()
            .size_full()
            .relative()
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
                        show_view_details,
                        i18n.view_cleanup_details(),
                        {
                            let store = store_entity.clone();
                            move |_, _, cx| {
                                store.update(cx, |s, cx| {
                                    s.set_page(AppPage::Cleanup, cx);
                                });
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
                                disk_tile_value,
                                disk_tile_hint,
                                colors::accent_cyan(),
                                {
                                    let store = store_entity.clone();
                                    move |_, window, cx| {
                                        let i18n = store.read(cx).i18n();
                                        let body = cx.new(|cx| DiskVolumesDialog::new(i18n, cx));
                                        window.open_dialog(cx, move |dialog, _, _| {
                                            dialog
                                                .title(i18n.disk_volumes_title())
                                                .child(body.clone())
                                        });
                                    }
                                },
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
                    .child(system_status_card(
                        disk_pct,
                        store.disk_total,
                        store
                            .last_report
                            .as_ref()
                            .map(|r| r.safe_reclaimable())
                            .unwrap_or(0),
                        &disk_free,
                        store.settings.expert_mode,
                        &i18n,
                    ))
                    .child(history_card(
                        &store.cleanup_history,
                        store_entity.clone(),
                        &i18n,
                        self.recently_deleted_expanded,
                        {
                            let dashboard = dashboard.clone();
                            move |_, _, cx| {
                                dashboard.update(cx, |this, cx| {
                                    this.recently_deleted_expanded = !this.recently_deleted_expanded;
                                    cx.notify();
                                });
                            }
                        },
                        cx,
                    ))
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
                    ),
            ))
    }
}

fn disk_volumes_list(volumes: &[DiskVolume], i18n: &I18n) -> Div {
    let mut list = v_flex().gap_4();
    for volume in volumes {
        let label = volume.label.clone();
        let used = format_bytes(volume.used_bytes);
        let total = format_bytes(volume.total_bytes);
        let free = format_bytes(volume.free_bytes());
        list = list.child(
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
    list
}

fn system_status_card(
    disk_pct: f32,
    disk_total: u64,
    safe_reclaimable: u64,
    disk_free: &str,
    expert_mode: bool,
    i18n: &I18n,
) -> Div {
    let optimizable_pct = if disk_total > 0 {
        ((safe_reclaimable as f32 / disk_total as f32) * 100.0).min(100.)
    } else {
        0.
    };

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
        .child(ui::metric_bar(
            i18n.disk_usage_metric(),
            disk_pct,
            colors::accent_cyan(),
        ))
        .child(ui::metric_bar(
            i18n.optimizable_space(),
            optimizable_pct,
            colors::safe_green(),
        ))
        .child(
            h_flex()
                .justify_between()
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_secondary())
                        .child(i18n.free_space(disk_free)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_muted())
                        .child(if expert_mode {
                            i18n.expert_mode_short()
                        } else {
                            i18n.simple_mode_short()
                        }),
                ),
        )
}

fn history_card(
    history: &CleanupHistory,
    store: Entity<AppStore>,
    i18n: &I18n,
    recently_deleted_expanded: bool,
    on_toggle: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
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
        let count = restorable.len();
        let chevron = if recently_deleted_expanded {
            IconName::ChevronDown
        } else {
            IconName::ChevronRight
        };
        card = card.child(
            h_flex()
                .id("recently-deleted-toggle")
                .items_center()
                .gap_2()
                .cursor_pointer()
                .on_click(on_toggle)
                .child(
                    Icon::new(chevron)
                        .with_size(px(16.))
                        .text_color(colors::text_muted()),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::text_primary())
                        .child(i18n.recently_trashed_summary(count)),
                ),
        );
        if recently_deleted_expanded {
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
