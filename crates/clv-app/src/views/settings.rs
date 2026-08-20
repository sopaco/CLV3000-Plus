use crate::app::state::AppStore;
use crate::prelude::*;
use crate::theme::colors;
use clv_core::save_settings;

pub struct SettingsView {
    store: Entity<AppStore>,
}

impl SettingsView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self { store }
    }
}

impl Render for SettingsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.store.read(cx).settings.clone();
        let paths = settings
            .scan_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");
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
                        "显示完整路径、更多可清理项与高级选项",
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
                            .gap_2()
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
                                    .child("默认扫描以下目录（可在后续版本自定义编辑）"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_family("monospace")
                                    .text_color(colors::text_muted())
                                    .child(paths),
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
                                    .child("Rust · Node.js/Web · Android · iOS · Flutter · KMP · Java · Python · .NET · C/C++"),
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
