use crate::app::state::AppStore;
use crate::prelude::*;
use crate::theme::{colors, corner_sm};
use clv_platform::{list_startup_items, set_startup_enabled, StartupImpact};

pub struct StartupView {
    store: Entity<AppStore>,
    items: Vec<clv_platform::StartupItem>,
}

impl StartupView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self {
            store,
            items: list_startup_items(),
        }
    }

    fn refresh(&mut self) {
        self.items = list_startup_items();
    }

    fn toggle_item(&mut self, id: &str, enabled: bool) {
        match set_startup_enabled(id, enabled) {
            Ok(()) => self.refresh(),
            Err(e) => tracing::warn!("startup toggle: {e}"),
        }
    }
}

impl Render for StartupView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = &self.store;
        let high_impact = self
            .items
            .iter()
            .filter(|i| i.enabled && i.impact == StartupImpact::High)
            .count();

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
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(ui::page_header(
                                "启动项管理",
                                format!(
                                    "共 {} 项 · {} 项高影响启动项",
                                    self.items.len(),
                                    high_impact
                                ),
                            ))
                            .child(
                                ui::action_button("startup-refresh", "刷新", None, false, cx)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.refresh();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .when(self.items.is_empty(), |this| {
                        this.child(ui::empty_state(
                            ui::EMPTY_STARTUP,
                            "未检测到启动项",
                            "当前平台暂不支持，或列表为空",
                        ))
                    })
                    .children(self.items.iter().map(|item| {
                        let id = item.id.clone();
                        let enabled = item.enabled;
                        let impact = item.impact.label();
                        let kind = item.kind.label();
                        let description = item.description.clone();
                        let name = item.name.clone();
                        let sw_id = eid(format!("sw-{id}"));
                        let impact_color = match item.impact {
                            StartupImpact::High => colors::red(),
                            StartupImpact::Medium => colors::from_hex(0xf59e0b),
                            StartupImpact::Low => colors::text_muted(),
                        };

                        ui::card()
                            .p_4()
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .gap_4()
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(colors::text_primary())
                                                    .child(name),
                                            )
                                            .child(
                                                h_flex()
                                                    .gap_2()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(colors::text_muted())
                                                            .child(kind),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .px(px(8.))
                                                            .py(px(2.))
                                                            .rounded(corner_sm())
                                                            .bg(impact_color.opacity(0.15))
                                                            .text_color(impact_color)
                                                            .child(format!("影响: {impact}")),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(colors::text_secondary())
                                                    .child(description),
                                            ),
                                    )
                                    .child(
                                        Switch::new(sw_id)
                                            .checked(enabled)
                                            .cursor_pointer()
                                            .on_click(cx.listener({
                                                let id = id.clone();
                                                move |this, checked, _, cx| {
                                                    this.toggle_item(&id, *checked);
                                                    cx.notify();
                                                }
                                            })),
                                    ),
                            )
                    })),
            ))
    }
}
