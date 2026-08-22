use crate::app::state::{AppPage, AppStore};
use crate::prelude::*;
use crate::theme::colors;
use clv_core::{AgentProject, RiskLevel};
use gpui::{ScrollStrategy, Subscription, UniformListScrollHandle};
use gpui_component::input::{Input, InputEvent, InputState};

/// Virtualized row slot — card body + gap between rows.
const AGENT_ROW_H: f32 = 104.;
const AGENT_CARD_H: f32 = 92.;

pub struct AgentView {
    store: Entity<AppStore>,
    search_query: String,
    search_input: Option<Entity<InputState>>,
    _search_subscription: Option<Subscription>,
    scroll_handle: UniformListScrollHandle,
}

impl AgentView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self {
            store,
            search_query: String::new(),
            search_input: None,
            _search_subscription: None,
            scroll_handle: UniformListScrollHandle::new(),
        }
    }

    fn all_projects(&self, cx: &Context<Self>) -> Vec<AgentProject> {
        self.store
            .read(cx)
            .last_report
            .as_ref()
            .map(|r| r.agent_projects.clone())
            .unwrap_or_default()
    }

    fn filtered_projects(&self, cx: &Context<Self>) -> Vec<AgentProject> {
        let query = self.search_query.trim().to_lowercase();
        let projects = self.all_projects(cx);
        if query.is_empty() {
            return projects;
        }

        projects
            .into_iter()
            .filter(|project| {
                project.name.to_lowercase().contains(&query)
                    || project.reason.to_lowercase().contains(&query)
                    || project.path.to_string_lossy().to_lowercase().contains(&query)
                    || project
                        .stacks
                        .iter()
                        .any(|stack| stack.label().to_lowercase().contains(&query))
            })
            .collect()
    }

    fn ensure_search_input(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        if let Some(input) = &self.search_input {
            return input.clone();
        }

        let input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("搜索名称、路径、技术栈或原因…")
        });
        let subscription = cx.subscribe(&input, |view, input, event, cx| {
            if matches!(event, InputEvent::Change) {
                view.search_query = input.read(cx).value().to_string();
                view.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                cx.notify();
            }
        });
        self._search_subscription = Some(subscription);
        self.search_input = Some(input.clone());
        input
    }

    fn render_row(
        &self,
        project: &clv_core::AgentProject,
        store: Entity<AppStore>,
        scanning: bool,
        cx: &mut Context<Self>,
    ) -> Div {
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

        div()
            .w_full()
            .h(px(AGENT_ROW_H))
            .pb_3()
            .child(
                ui::soft_card()
                    .w_full()
                    .h(px(AGENT_CARD_H))
                    .px_4()
                    .child(
                        h_flex()
                            .size_full()
                            .justify_between()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(colors::text_primary())
                                            .truncate()
                                            .child(project.name.clone()),
                                    )
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .child(ui::risk_badge(risk))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(colors::text_muted())
                                                    .truncate()
                                                    .child(stacks),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(colors::text_muted())
                                                    .child(inactive),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors::text_secondary())
                                            .truncate()
                                            .child(project.reason.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .flex()
                                    .flex_col()
                                    .items_end()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::accent_blue())
                                            .child(project.size_human()),
                                    )
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .child(ui::open_path_button(open_id, &path))
                                            .child(
                                                ui::action_button(
                                                    clean_id,
                                                    "清理缓存",
                                                    Some(ui::ACTION_CLEAN),
                                                    true,
                                                    cx,
                                                )
                                                .disabled(scanning)
                                                .on_click({
                                                    let store = store_clean.clone();
                                                    let project_path = path.clone();
                                                    move |_, _, cx| {
                                                        store.update(cx, |s, cx| {
                                                            if let Some(report) = &mut s.last_report {
                                                                for item in &mut report.items {
                                                                    if item.project_root.as_ref()
                                                                        == Some(&project_path)
                                                                    {
                                                                        item.selected = true;
                                                                    }
                                                                }
                                                            }
                                                            s.set_page(AppPage::Cleanup, cx);
                                                        });
                                                    }
                                                }),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for AgentView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let search_input = self.ensure_search_input(window, cx);
        let store_ref = self.store.read(cx);
        let total = store_ref
            .last_report
            .as_ref()
            .map(|r| r.agent_projects.len())
            .unwrap_or(0);
        let projects = self.filtered_projects(cx);
        let store = self.store.clone();
        let scanning = store_ref.scanning;
        let count = projects.len();
        let searching = !self.search_query.trim().is_empty();
        let scroll_handle = self.scroll_handle.clone();

        div()
            .id("agent-view")
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .overflow_hidden()
            .child(
                div()
                    .flex_shrink_0()
                    .px_6()
                    .pt_6()
                    .pb_4()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(ui::page_banner(
                        "Agent 试验项目",
                        if searching {
                            format!("搜索到 {count} / {total} 个项目")
                        } else if total > 0 {
                            format!("共 {total} 个 Agent 项目")
                        } else {
                            "识别 Codex / Claude / Cursor 等 Agent 可能创建的项目".into()
                        },
                    ))
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .w(px(280.))
                                    .child(Input::new(&search_input)),
                            )
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
                    ),
            )
            .child(
                ui::list_body(
                    div()
                        .flex_1()
                        .min_h_0()
                        .w_full()
                        .flex()
                        .flex_col()
                        .px_6()
                        .pb_6()
                        .when(total == 0, |this| {
                            this.flex()
                                .items_center()
                                .justify_center()
                                .child(ui::empty_state(
                                    ui::EMPTY_AGENT,
                                    "暂无 Agent 项目",
                                    "点击上方「扫描 Agent 项目」，将识别含 .agents / .claude 等标记的目录",
                                ))
                        })
                        .when(total > 0 && count == 0, |this| {
                            this.flex().items_center().justify_center().child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_muted())
                                    .child("没有匹配的项目"),
                            )
                        })
                        .when(count > 0, |this| {
                            let projects = projects.clone();
                            this.child(ui::uniform_list_pane(
                                "agent-rows",
                                count,
                                scroll_handle,
                                cx,
                                move |this, visible_range, _window, cx| {
                                    visible_range
                                        .filter_map(|ix| {
                                            projects.get(ix).map(|project| {
                                                this.render_row(project, store.clone(), scanning, cx)
                                            })
                                        })
                                        .collect()
                                },
                            ))
                        }),
                ),
            )
    }
}
