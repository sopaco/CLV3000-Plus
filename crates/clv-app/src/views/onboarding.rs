use crate::app::state::AppStore;
use crate::prelude::*;
use crate::theme::{colors, corner_sm};

pub struct OnboardingView {
    store: Entity<AppStore>,
    step: u8,
    expert: bool,
}

impl OnboardingView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self {
            store,
            step: 0,
            expert: false,
        }
    }
}

impl Render for OnboardingView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.clone();
        let step = self.step;
        let expert = self.expert;

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .relative()
            .overflow_hidden()
            .child(ui::glow_orb_public(200., colors::accent_cyan(), 40., 60.))
            .child(
                ui::glass_card()
                    .w(px(580.))
                    .p_8()
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        h_flex()
                            .gap_4()
                            .items_center()
                            .child(ui::brand_logo(56.))
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
                                            .child("欢迎使用 CLV3000 Plus"),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(colors::text_secondary())
                                            .child("您的电脑安全管家"),
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .children((0..3).map(|i| {
                                div()
                                    .flex_1()
                                    .h(px(4.))
                                    .rounded(corner_sm())
                                    .bg(if i <= step {
                                        colors::accent_cyan()
                                    } else {
                                        colors::border()
                                    })
                            })),
                    )
                    .child(if step == 0 {
                        v_flex()
                            .gap_3()
                            .child(feature_line("智能清理 Agent 与开发项目的缓存和依赖"))
                            .child(feature_line("管理登录启动项，减轻开机负担"))
                            .child(feature_line("查看并结束高占用进程"))
                            .into_any_element()
                    } else if step == 1 {
                        v_flex()
                            .gap_4()
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_secondary())
                                    .child("选择使用模式："),
                            )
                            .child(mode_option(
                                "mode-simple",
                                "简单模式（推荐）",
                                "用人话解释每一项，默认只清理安全内容",
                                !expert,
                                cx,
                                |this, _, _, cx| {
                                    this.expert = false;
                                    cx.notify();
                                },
                            ))
                            .child(mode_option(
                                "mode-expert",
                                "专家模式",
                                "显示完整路径，可清理更多项目",
                                expert,
                                cx,
                                |this, _, _, cx| {
                                    this.expert = true;
                                    cx.notify();
                                },
                            ))
                            .into_any_element()
                    } else {
                        v_flex()
                            .gap_3()
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_secondary())
                                    .child("将扫描以下常见目录："),
                            )
                            .child(
                                ui::glass_card()
                                    .p_4()
                                    .text_base()
                                    .text_color(colors::text_primary())
                                    .child("~/Projects · ~/Documents · ~/Desktop · ~/Developer 等"),
                            )
                            .into_any_element()
                    })
                    .child(
                        h_flex()
                            .justify_between()
                            .child(
                                ui::action_button("onboard-back", "上一步", None, false, cx)
                                    .disabled(step == 0)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.step > 0 {
                                            this.step -= 1;
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(if step < 2 {
                                ui::scan_cta_button("onboard-next", "下一步", false, cx)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.step += 1;
                                        cx.notify();
                                    }))
                                    .into_any_element()
                            } else {
                                ui::scan_cta_button("onboard-finish", "开始体检", false, cx)
                                    .on_click({
                                        let store = store.clone();
                                        let expert = expert;
                                        move |_, _, cx| {
                                            store.update(cx, |s, cx| {
                                                s.finish_onboarding(expert, vec![], cx);
                                                s.start_scan(cx);
                                            });
                                        }
                                    })
                                    .into_any_element()
                            }),
                    ),
            )
    }
}

fn feature_line(text: &str) -> Div {
    h_flex()
        .gap_3()
        .items_center()
        .child(
            div()
                .size(px(8.))
                .rounded(corner_sm())
                .bg(colors::accent_cyan()),
        )
        .child(
            div()
                .text_base()
                .text_color(colors::text_secondary())
                .child(text.to_string()),
        )
}

fn mode_option(
    id: &'static str,
    title: &'static str,
    desc: &'static str,
    selected: bool,
    cx: &mut Context<OnboardingView>,
    on_click: impl Fn(&mut OnboardingView, &gpui::ClickEvent, &mut Window, &mut Context<OnboardingView>) + 'static,
) -> impl IntoElement {
    ui::glass_card()
        .p_4()
        .border_color(if selected {
            colors::accent_cyan()
        } else {
            colors::from_hex(0xffffff).opacity(0.08)
        })
        .when(selected, |el| el.bg(colors::accent_blue_bg().opacity(0.5)))
        .cursor_pointer()
        .id(id)
        .on_click(cx.listener(on_click))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if selected {
                            colors::text_primary()
                        } else {
                            colors::text_secondary()
                        })
                        .child(title),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_muted())
                        .child(desc),
                ),
        )
}
