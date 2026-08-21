use crate::app::state::AppStore;
use crate::prelude::*;
use crate::theme::colors;
use clv_core::{format_scan_paths, parse_scan_paths, save_settings};
use gpui::{Subscription, Window};
use gpui_component::input::{Input, InputEvent, InputState};

pub struct SettingsView {
    store: Entity<AppStore>,
    scan_paths_input: Option<Entity<InputState>>,
    _scan_paths_subscription: Option<Subscription>,
}

impl SettingsView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self {
            store,
            scan_paths_input: None,
            _scan_paths_subscription: None,
        }
    }

    fn ensure_scan_paths_input(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        initial: String,
    ) -> Entity<InputState> {
        if let Some(input) = &self.scan_paths_input {
            return input.clone();
        }

        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .default_value(initial)
                .placeholder("每行一个目录路径，支持 ~/Projects")
        });
        let subscription = cx.subscribe(&input, |_view, _input, event, cx| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        });
        self._scan_paths_subscription = Some(subscription);
        self.scan_paths_input = Some(input.clone());
        input
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.store.read(cx).settings.clone();
        let paths_text = format_scan_paths(&settings.scan_paths);
        let scan_paths_input =
            self.ensure_scan_paths_input(window, cx, paths_text);
        let store = self.store.clone();

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
                    .child(ui::page_header("设置", "个性化扫描与清理行为"))
                    .child(ui::setting_row(
                        "专家模式",
                        "显示完整路径、受保护项与更多可清理项",
                        Switch::new("expert-mode")
                            .checked(settings.expert_mode)
                            .cursor_pointer()
                            .on_click({
                                let store = store.clone();
                                move |checked, _, cx| {
                                    store.update(cx, |s, cx| {
                                        s.settings.expert_mode = *checked;
                                        let _ = save_settings(&s.settings);
                                        cx.notify();
                                    });
                                }
                            }),
                    ))
                    .child(ui::setting_row(
                        "软删除（推荐）",
                        "清理的文件移入回收区，7 天后自动清除",
                        Switch::new("soft-delete")
                            .checked(settings.soft_delete)
                            .cursor_pointer()
                            .on_click({
                                let store = store.clone();
                                move |checked, _, cx| {
                                    store.update(cx, |s, cx| {
                                        s.settings.soft_delete = *checked;
                                        let _ = save_settings(&s.settings);
                                        cx.notify();
                                    });
                                }
                            }),
                    ))
                    .child(ui::setting_row(
                        "Agent 项目识别",
                        "根据目录名与 .agents/.claude/.cursor 等标记识别 Agent 试验项目",
                        Switch::new("agent-heuristics")
                            .checked(settings.include_agent_heuristics)
                            .cursor_pointer()
                            .on_click({
                                let store = store.clone();
                                move |checked, _, cx| {
                                    store.update(cx, |s, cx| {
                                        s.settings.include_agent_heuristics = *checked;
                                        let _ = save_settings(&s.settings);
                                        cx.notify();
                                    });
                                }
                            }),
                    ))
                    .child(
                        ui::card()
                            .p_5()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::text_primary())
                                    .child("扫描目录"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_secondary())
                                    .child("每行一个路径，保存后下次扫描生效"),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .min_h(px(120.))
                                    .child(Input::new(&scan_paths_input)),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        ui::std_button(
                                            Button::new("save-scan-paths")
                                                .label("保存扫描目录")
                                                .on_click({
                                                    let store = store.clone();
                                                    let scan_paths_input =
                                                        scan_paths_input.clone();
                                                    move |_, _, cx| {
                                                        let text = scan_paths_input
                                                            .read(cx)
                                                            .value()
                                                            .to_string();
                                                        let paths = parse_scan_paths(&text);
                                                        store.update(cx, |s, cx| {
                                                            if !paths.is_empty() {
                                                                s.settings.scan_paths = paths;
                                                            }
                                                            let _ = save_settings(&s.settings);
                                                            cx.notify();
                                                        });
                                                    }
                                                }),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors::text_muted())
                                            .child("留空行会被忽略；支持 ~/ 展开"),
                                    ),
                            ),
                    )
                    .child(
                        ui::card()
                            .p_5()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::text_primary())
                                    .child("支持清理的技术栈"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_secondary())
                                    .child(
                                        "Rust · Node.js/Web · Android · iOS · Flutter · KMP · Java · Python · .NET · C/C++ · Go · Ruby · PHP · Unity · Terraform",
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::text_muted())
                            .child("CLV3000 Plus v0.1.0 · Rust + GPUI"),
                    ),
            ))
    }
}
