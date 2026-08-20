use crate::app::state::{AppPage, AppStore};
use crate::prelude::*;
use crate::theme::{colors, corner_sm};
use clv_platform::{list_startup_items, set_startup_enabled, StartupImpact};
use gpui::{Subscription, UniformListScrollHandle};

const STARTUP_ROW_H: f32 = 92.;

pub struct StartupView {
    store: Entity<AppStore>,
    #[allow(dead_code)]
    _store_subscription: Subscription,
    items: Vec<clv_platform::StartupItem>,
    scroll_handle: UniformListScrollHandle,
    loaded: bool,
    last_error: Option<String>,
}

impl StartupView {
    pub fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        let store_subscription = cx.observe(&store, |view, store, cx| {
            if store.read(cx).page == AppPage::Startup {
                view.ensure_loaded(cx);
            }
        });

        Self {
            store,
            _store_subscription: store_subscription,
            items: Vec::new(),
            scroll_handle: UniformListScrollHandle::new(),
            loaded: false,
            last_error: None,
        }
    }

    fn ensure_loaded(&mut self, cx: &mut Context<Self>) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        self.refresh(cx);
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.last_error = None;
        self.reload_items(cx);
    }

    /// 重新枚举启动项，保持开关与系统真实状态一致
    fn reload_items(&mut self, cx: &mut Context<Self>) {
        self.items = list_startup_items();
        let count = self.items.len();
        self.store.update(cx, |store, cx| {
            store.startup_count = count;
            cx.notify();
        });
        cx.notify();
    }

    fn toggle_item(&mut self, id: &str, enabled: bool, cx: &mut Context<Self>) {
        match set_startup_enabled(id, enabled) {
            Ok(()) => self.last_error = None,
            Err(e) => {
                tracing::warn!("startup toggle: {e}");
                self.last_error = Some(format!("启动项操作失败：{e}"));
            }
        }
        self.reload_items(cx);
    }

    fn render_row(&self, ix: usize, cx: &mut Context<Self>) -> Div {
        let Some(item) = self.items.get(ix) else {
            return div().h(px(STARTUP_ROW_H));
        };

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

        ui::soft_card()
            .h(px(STARTUP_ROW_H))
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
                                    .truncate()
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
                                    this.toggle_item(&id, *checked, cx);
                                }
                            })),
                    ),
            )
    }
}

impl Render for StartupView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.store.read(cx).page == AppPage::Startup && !self.loaded {
            self.ensure_loaded(cx);
        }

        let high_impact = self
            .items
            .iter()
            .filter(|i| i.enabled && i.impact == StartupImpact::High)
            .count();
        let count = self.items.len();
        let scroll_handle = self.scroll_handle.clone();

        div()
            .id("startup-view")
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
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(ui::page_header(
                                "启动项管理",
                                format!("共 {count} 项 · {high_impact} 项高影响启动项"),
                            ))
                            .child(
                                ui::action_button("startup-refresh", "刷新", None, false, cx)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.refresh(cx);
                                    })),
                            ),
                    ),
            )
            .when_some(self.last_error.clone(), |this, error| {
                this.child(
                    div()
                        .flex_shrink_0()
                        .mx_6()
                        .mb_2()
                        .px_4()
                        .py_2()
                        .rounded(corner_sm())
                        .bg(colors::red().opacity(0.12))
                        .text_color(colors::red())
                        .text_sm()
                        .child(error),
                )
            })
            .child(
                ui::list_body(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .px_6()
                        .pb_6()
                        .when(!self.loaded, |this| {
                            this.flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .text_base()
                                        .text_color(colors::text_muted())
                                        .child("正在加载启动项…"),
                                )
                        })
                        .when(self.loaded && count == 0, |this| {
                            this.flex()
                                .items_center()
                                .justify_center()
                                .child(ui::empty_state(
                                    ui::EMPTY_STARTUP,
                                    "未检测到启动项",
                                    "当前平台暂不支持，或列表为空",
                                ))
                        })
                        .when(self.loaded && count > 0, |this| {
                            this.child(ui::uniform_list_pane(
                                "startup-rows",
                                count,
                                scroll_handle,
                                cx,
                                move |this, visible_range, _window, cx| {
                                    visible_range
                                        .map(|ix| this.render_row(ix, cx))
                                        .collect()
                                },
                            ))
                        }),
                ),
            )
    }
}
