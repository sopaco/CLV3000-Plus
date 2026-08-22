use crate::app::state::AppStore;
use crate::i18n::I18n;
use crate::prelude::*;
use crate::theme::colors;
use clv_core::{format_scan_paths, parse_scan_paths, save_settings, LanguagePreference};
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
        placeholder: &'static str,
    ) -> Entity<InputState> {
        if let Some(input) = &self.scan_paths_input {
            return input.clone();
        }

        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .default_value(initial)
                .placeholder(placeholder)
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
        let i18n = I18n::from_settings(&settings);
        let paths_text = format_scan_paths(&settings.scan_paths);
        let scan_paths_input = self.ensure_scan_paths_input(
            window,
            cx,
            paths_text,
            i18n.scan_paths_placeholder(),
        );
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
                    .child(ui::page_header(i18n.settings_title(), i18n.settings_subtitle()))
                    .child(language_selector(&i18n, settings.language, store.clone(), cx))
                    .child(ui::setting_row(
                        i18n.expert_mode_label(),
                        i18n.expert_mode_desc(),
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
                        i18n.soft_delete_label(),
                        i18n.soft_delete_desc(),
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
                        i18n.agent_heuristics_label(),
                        i18n.agent_heuristics_desc(),
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
                            .min_w_0()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::text_primary())
                                    .child(i18n.scan_paths_title()),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_secondary())
                                    .child(i18n.scan_paths_desc()),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .min_w_0()
                                    .h(px(160.))
                                    .overflow_hidden()
                                    .child(Input::new(&scan_paths_input).h_full().w_full()),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        ui::std_button(
                                            Button::new("save-scan-paths")
                                                .label(i18n.save_scan_paths())
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
                                            .child(i18n.scan_paths_hint()),
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
                                    .child(i18n.supported_stacks_title()),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_secondary())
                                    .child(i18n.supported_stacks_list()),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::text_muted())
                            .child(i18n.app_version()),
                    ),
            ))
    }
}

fn language_selector(
    i18n: &I18n,
    current: LanguagePreference,
    store: Entity<AppStore>,
    cx: &App,
) -> Div {
    ui::card()
        .p_5()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::text_primary())
                .child(i18n.language_section_title()),
        )
        .child(
            div()
                .text_base()
                .text_color(colors::text_secondary())
                .child(i18n.language_section_desc()),
        )
        .child(
            h_flex()
                .gap_2()
                .flex_wrap()
                .child(language_pill(
                    "lang-system",
                    i18n.language_system(),
                    current == LanguagePreference::System,
                    LanguagePreference::System,
                    store.clone(),
                    cx,
                ))
                .child(language_pill(
                    "lang-zh",
                    i18n.language_zh(),
                    current == LanguagePreference::Zh,
                    LanguagePreference::Zh,
                    store.clone(),
                    cx,
                ))
                .child(language_pill(
                    "lang-en",
                    i18n.language_en(),
                    current == LanguagePreference::En,
                    LanguagePreference::En,
                    store.clone(),
                    cx,
                ))
                .child(language_pill(
                    "lang-ja",
                    i18n.language_ja(),
                    current == LanguagePreference::Ja,
                    LanguagePreference::Ja,
                    store,
                    cx,
                )),
        )
}

fn language_pill(
    id: &'static str,
    label: &'static str,
    active: bool,
    preference: LanguagePreference,
    store: Entity<AppStore>,
    cx: &App,
) -> Button {
    ui::ghost_pill(id, label, active, cx).on_click(move |_, _, cx| {
        store.update(cx, |s, cx| {
            s.settings.language = preference;
            let _ = save_settings(&s.settings);
            cx.notify();
        });
    })
}
