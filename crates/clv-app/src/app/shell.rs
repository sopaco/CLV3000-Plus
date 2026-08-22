//! Window shell — custom title bar aligned with CLV visual style.

use super::{state::AppPage, ClvApp};
use crate::prelude::*;
use crate::theme::colors;
use gpui::{linear_color_stop, linear_gradient};
use gpui_component::{Root, TitleBar};

pub struct AppShell {
    app: Entity<ClvApp>,
}

impl AppShell {
    pub fn new(app: Entity<ClvApp>, _window: &Window, _cx: &mut Context<Self>) -> Self {
        Self { app }
    }
}

impl Render for AppShell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.app.read(cx).current_page(cx);
        let i18n = self.app.read(cx).i18n(cx);
        let page_title = page.title(&i18n);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::bg_app())
            .child(
                TitleBar::new().child(
                    h_flex()
                        .w_full()
                        .h_full()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::accent_blue())
                                .child("CLV3000 Plus"),
                        )
                        .when(page != AppPage::Onboarding, |this| {
                            this.child(
                                div()
                                    .w(px(1.))
                                    .h(px(14.))
                                    .bg(colors::panel_divider()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors::text_secondary())
                                    .child(page_title),
                            )
                        }),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .bg(titlebar_content_gradient())
                    .child(self.app.clone()),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}

fn titlebar_content_gradient() -> gpui::Background {
    linear_gradient(
        180.,
        linear_color_stop(colors::bg_app(), 0.0),
        linear_color_stop(colors::gradient_titlebar_end(), 1.0),
    )
}
