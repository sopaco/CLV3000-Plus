use crate::app::state::{AppPage, AppStore};
use crate::prelude::*;
use crate::theme::colors;
use clv_core::RiskLevel;

pub struct AgentView {
    store: Entity<AppStore>,
}

impl AgentView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self { store }
    }
}

impl Render for AgentView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store_ref = self.store.read(cx);
        let projects = store_ref
            .last_report
            .as_ref()
            .map(|r| r.agent_projects.clone())
            .unwrap_or_default();
        let store = self.store.clone();
        let scanning = store_ref.scanning;

        div()
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .child(ui::scroll_y(
                div()
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(ui::page_banner(
                        "Agent 试验项目",
                        "识别 Codex / Claude / Cursor 等 Agent 可能创建的项目",
                    ))
                    .child(
                        h_flex()
                            .justify_end()
                            .child(
                                ui::action_button(
                                    "agent-scan",
                                    if scanning { "扫描中…" } else { "扫描 Agent 项目" },
                                    Some(ui::ACTION_SCAN),
                                    true,
                                    cx,
                                )
                                .disabled(scanning)
                                .on_click({
                                    let store = store.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| s.start_scan(cx));
                                    }
                                }),
                            ),
                    )
                    .when(projects.is_empty(), |this| {
                        this.child(ui::empty_state(
                            ui::EMPTY_AGENT,
                            "暂无 Agent 项目数据",
                            "先运行一次「立即体检」",
                        ))
                    })
                    .children(projects.iter().map(|project| {
                        let stacks: String = project
                            .stacks
                            .iter()
                            .map(|s| s.label())
                            .collect::<Vec<_>>()
                            .join(" · ");
                        let inactive = project
                            .days_inactive
                            .map(|d| format!("{d} 天未使用"))
                            .unwrap_or_else(|| "未知".into());
                        let risk = if project.days_inactive.unwrap_or(0) > 14 {
                            RiskLevel::Safe
                        } else {
                            RiskLevel::Caution
                        };
                        let path = project.path.clone();
                        let store_clean = store.clone();
                        let clean_id = eid(format!("agent-clean-{}", project.name));
                        let open_id = eid(format!("agent-open-{}", project.name));

                        ui::card()
                            .p_5()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_start()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::text_primary())
                                            .child(project.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(colors::accent_blue())
                                            .child(project.size_human()),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(ui::risk_badge(risk))
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(colors::text_muted())
                                            .child(stacks),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(colors::text_muted())
                                            .child(inactive),
                                    ),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_secondary())
                                    .child(project.reason.clone()),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_muted())
                                    .child(format!("包含 {} 个可清理子项", project.items.len())),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(ui::open_path_button(open_id, &path))
                                    .child(
                                        ui::action_button(
                                            clean_id,
                                            "清理此项目缓存",
                                            Some(ui::ACTION_CLEAN),
                                            true,
                                            cx,
                                        )
                                        .on_click({
                                            let store = store_clean.clone();
                                            let project_path = path.clone();
                                            move |_, _, cx| {
                                                store.update(cx, |s, cx| {
                                                    if let Some(report) = &mut s.last_report {
                                                        for item in &mut report.items {
                                                            if item.project_root.as_ref() == Some(&project_path) {
                                                                item.selected = true;
                                                            }
                                                        }
                                                    }
                                                    s.page = AppPage::Cleanup;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                    ),
                            )
                    })),
            ))
    }
}
