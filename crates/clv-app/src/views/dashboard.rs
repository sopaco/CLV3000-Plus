use crate::app::state::{AppPage, AppStore};
use crate::prelude::*;
use crate::theme::{colors, corner};
use clv_core::format_bytes;

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
            "体检中…"
        } else {
            "立即体检"
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
                        cx,
                    ))
                    .child(
                        h_flex()
                            .gap_3()
                            .flex_wrap()
                            .child(ui::quick_tile(
                                "tile-disk",
                                "磁盘使用",
                                format!("{disk_pct:.0}%"),
                                format!("{disk_used} / {disk_total}"),
                                colors::accent_cyan(),
                                |_, _, _| {},
                            ))
                            .child(ui::quick_tile(
                                "tile-clean",
                                "可释放空间",
                                reclaimable.clone(),
                                format!("{item_count} 个可清理项"),
                                colors::safe_green(),
                                {
                                    let store = store_entity.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| {
                                            s.page = AppPage::Cleanup;
                                            cx.notify();
                                        });
                                    }
                                },
                            ))
                            .child(ui::quick_tile(
                                "tile-agent",
                                "Agent 项目",
                                agent_count.to_string(),
                                format!("约 {agent_bytes}"),
                                colors::from_hex(0xa78bfa),
                                {
                                    let store = store_entity.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| {
                                            s.page = AppPage::Agent;
                                            cx.notify();
                                        });
                                    }
                                },
                            ))
                            .child(ui::quick_tile(
                                "tile-startup",
                                "启动项",
                                store.startup_count.to_string(),
                                "管理开机启动",
                                colors::warn_orange(),
                                {
                                    let store = store_entity.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| {
                                            s.page = AppPage::Startup;
                                            cx.notify();
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
                                    .child("系统状态"),
                            )
                            .child(ui::metric_bar("磁盘占用", disk_pct, colors::accent_cyan()))
                            .child(
                                ui::metric_bar(
                                    "可优化空间",
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
                                            .child(format!("可用空间 {disk_free}")),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors::text_muted())
                                            .child(if store.settings.expert_mode {
                                                "专家模式"
                                            } else {
                                                "简单模式"
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
                                    .child("全方位守护"),
                            )
                            .child(feature_line(
                                "识别 Claude / Cursor / Codex 等 Agent 试验项目",
                            ))
                            .child(feature_line(
                                "安全清理多技术栈构建产物与依赖缓存",
                            ))
                            .child(feature_line(
                                "管理登录启动项，减轻开机负担",
                            ))
                            .child(feature_line(
                                "查看高占用进程，一键释放系统资源",
                            )),
                    )
                    .when_some(store.last_cleanup_freed, |this, freed| {
                        this.child(
                            ui::glass_card()
                                .p_4()
                                .border_color(colors::safe_green().opacity(0.3))
                                .child(
                                    h_flex()
                                        .gap_3()
                                        .items_center()
                                        .child(
                                            div()
                                                .w(px(3.))
                                                .h(px(16.))
                                                .rounded(corner())
                                                .bg(colors::safe_green()),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(colors::safe_green())
                                                .child(format!(
                                                    "上次清理已释放 {}",
                                                    format_bytes(freed)
                                                )),
                                        ),
                                ),
                        )
                    }),
            ))
    }
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
