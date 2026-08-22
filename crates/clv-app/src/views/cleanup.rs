use crate::app::state::{AppStore, CleanupFilter};
use crate::i18n::{self, I18n};
use crate::prelude::*;
use crate::theme::{colors, corner_md};
use clv_core::{ScanItem};
use gpui::{ScrollStrategy, UniformListScrollHandle};
use gpui_component::Icon;
use std::path::{Path, PathBuf};

/// Virtualized row slot — card body + gap between rows.
const CLEANUP_ROW_H: f32 = 108.;
const CLEANUP_CARD_H: f32 = 96.;
const CLEANUP_PATH_ROW_H: f32 = 36.;

pub struct CleanupView {
    store: Entity<AppStore>,
    scroll_handle: UniformListScrollHandle,
}

impl CleanupView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self {
            store,
            scroll_handle: UniformListScrollHandle::new(),
        }
    }

    fn render_row(
        &self,
        item: &ScanItem,
        expert: bool,
        i18n: &I18n,
        lang: clv_core::Language,
        store: Entity<AppStore>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Div {
        let id = item.id.clone();
        let path_display = if expert {
            item.path.display().to_string()
        } else {
            item.project_root
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| item.name.clone())
        };
        let item_path = item.path.clone();
        let item_name = item.name.clone();
        let item_description = item.description.clone();
        let cb_id = eid(format!("cb-{id}"));
        let exp_id = eid(format!("exp-{id}"));
        let path_id = eid(format!("path-{id}"));

        div()
            .w_full()
            .h(px(CLEANUP_ROW_H))
            .pb_3()
            .child(
                ui::card()
                    .w_full()
                    .h(px(CLEANUP_CARD_H))
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        h_flex()
                            .items_start()
                            .gap_3()
                            .child(
                                div()
                                    .flex()
                                    .flex_shrink_0()
                                    .items_center()
                                    .justify_center()
                                    .h(px(CLEANUP_PATH_ROW_H))
                                    .child(
                                        Checkbox::new(cb_id)
                                            .checked(item.selected)
                                            .cursor_pointer()
                                            .on_click(cx.listener({
                                                let id = id.clone();
                                                let store = store.clone();
                                                move |_, checked, _, cx| {
                                                    let selected = *checked;
                                                    store.update(cx, |s, cx| {
                                                        if let Some(report) = &mut s.last_report {
                                                            if let Some(item) =
                                                                report.items.iter_mut().find(|i| i.id == id)
                                                            {
                                                                item.selected = selected;
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                }
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        h_flex()
                                            .min_h(px(CLEANUP_PATH_ROW_H))
                                            .items_center()
                                            .gap_3()
                                            .child(clickable_folder_path(
                                                path_id,
                                                &item_path,
                                                path_display,
                                            ))
                                            .child(
                                                div()
                                                    .flex_shrink_0()
                                                    .text_base()
                                                    .text_color(colors::accent_blue())
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child(item.size_human()),
                                            ),
                                    )
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .child(ui::risk_badge(item.risk, lang))
                                            .child(
                                                div()
                                                    .text_base()
                                                    .text_color(colors::text_muted())
                                                    .child(i18n::tech_stack_label(lang, item.stack)),
                                            )
                                            .child(
                                                div()
                                                    .text_base()
                                                    .text_color(colors::text_muted())
                                                    .child(i18n::scan_category_label(lang, &item.category).to_string()),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_shrink_0()
                                    .items_center()
                                    .h(px(CLEANUP_PATH_ROW_H))
                                    .child(
                                        ui::std_button(Button::new(exp_id).ghost().label(i18n.details()))
                                            .on_click(cx.listener({
                                                move |_, _, window, cx| {
                                                    open_item_detail_dialog(
                                                        window,
                                                        cx,
                                                        &item_name,
                                                        &item_description,
                                                        &item_path,
                                                    );
                                                }
                                            })),
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for CleanupView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store_ref = self.store.read(cx);
        let i18n = store_ref.i18n();
        let lang = store_ref.language();
        let items = store_ref.filtered_items();
        let selected_bytes = store_ref.selected_bytes();
        let selected_count = store_ref.selected_items().len();
        let has_report = store_ref.last_report.is_some();
        let scanning = store_ref.scanning;
        let cleaning = store_ref.cleaning;
        let expert = store_ref.settings.expert_mode;
        let store = self.store.clone();
        let count = items.len();
        let scroll_handle = self.scroll_handle.clone();

        div()
            .size_full()
            .flex()
            .min_h_0()
            .child(
                div()
                    .w(px(200.))
                    .h_full()
                    .flex()
                    .flex_col()
                    .p_3()
                    .gap_1()
                    .bg(colors::bg_sidebar())
                    .child(
                        div()
                            .px_2()
                            .pb_2()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::text_muted())
                            .child(i18n.filter_label()),
                    )
                    .child(filter_btn(
                        "filter-all",
                        i18n.filter_all(),
                        CleanupFilter::All,
                        &store,
                        &self.scroll_handle,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-safe",
                        i18n.filter_safe(),
                        CleanupFilter::SafeOnly,
                        &store,
                        &self.scroll_handle,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-project",
                        i18n.filter_project(),
                        CleanupFilter::ProjectBuildCache,
                        &store,
                        &self.scroll_handle,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-shared",
                        i18n.filter_shared(),
                        CleanupFilter::SharedToolCache,
                        &store,
                        &self.scroll_handle,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-dev-env",
                        i18n.filter_dev_env(),
                        CleanupFilter::DevEnvironment,
                        &store,
                        &self.scroll_handle,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-ai",
                        i18n.filter_ai(),
                        CleanupFilter::AiGenerated,
                        &store,
                        &self.scroll_handle,
                        cx,
                    )),
            )
            .child(ui::panel_divider().h_full().w(px(1.)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        h_flex()
                            .flex_shrink_0()
                            .p_4()
                            .gap_3()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        ui::ghost_pill("select-all", i18n.select_all(), false, cx).on_click({
                                            let store = store.clone();
                                            move |_, _, cx| {
                                                store.update(cx, |s, cx| {
                                                    s.select_all_filtered(true);
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                    )
                                    .child(
                                        ui::ghost_pill("deselect-all", i18n.deselect_all(), false, cx).on_click({
                                            let store = store.clone();
                                            move |_, _, cx| {
                                                store.update(cx, |s, cx| {
                                                    s.select_all_filtered(false);
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                    )
                                    .child(
                                        div()
                                            .text_base()
                                            .text_color(colors::text_secondary())
                                            .child(if has_report {
                                                i18n.items_selected_summary(items.len(), selected_count)
                                            } else {
                                                i18n.not_scanned_yet().to_string()
                                            }),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        ui::action_button(
                                            "rescan",
                                            if has_report { i18n.rescan() } else { i18n.start_scan() },
                                            Some(ui::ACTION_SCAN),
                                            true,
                                            cx,
                                        )
                                        .disabled(scanning || cleaning)
                                        .on_click({
                                            let store = store.clone();
                                            move |_, _, cx| {
                                                store.update(cx, |s, cx| s.start_scan(cx));
                                            }
                                        }),
                                    )
                                    .child(
                                        ui::action_button(
                                            "cleanup-run",
                                            i18n.clean_selected(),
                                            Some(ui::ACTION_CLEAN),
                                            true,
                                            cx,
                                        )
                                        .disabled(selected_count == 0 || cleaning || !has_report)
                                        .on_click({
                                            let store = store.clone();
                                            move |_, window, cx| {
                                                let bytes = store.read(cx).selected_bytes();
                                                let count = store.read(cx).selected_items().len();
                                                let i18n = store.read(cx).i18n();
                                                let store_confirm = store.clone();
                                                window.open_dialog(cx, move |dialog, _window, _cx| {
                                                    dialog
                                                        .title(i18n.confirm_cleanup_title())
                                                        .child(i18n.confirm_cleanup_body(count, bytes))
                                                        .confirm()
                                                        .on_ok({
                                                            let store = store_confirm.clone();
                                                            move |_, _, cx| {
                                                                store.update(cx, |s, cx| {
                                                                    s.run_cleanup(cx);
                                                                });
                                                                true
                                                            }
                                                        })
                                                });
                                            }
                                        }),
                                    ),
                            ),
                    )
                    .child(ui::panel_divider())
                    .child(
                        ui::list_body(
                            div()
                                .flex_1()
                                .min_h_0()
                                .w_full()
                                .flex()
                                .flex_col()
                                .p_4()
                                .when(!has_report && !scanning, |this| {
                                    this.child(cleanup_scan_prompt(&store, &i18n, cx))
                                })
                                .when(has_report && count == 0 && !scanning, |this| {
                                    this.flex()
                                        .items_center()
                                        .justify_center()
                                        .child(ui::empty_state(
                                            ui::EMPTY_SCAN,
                                            i18n.empty_filter_title(),
                                            i18n.empty_filter_hint(),
                                        ))
                                })
                                .when(scanning, |this| {
                                    let phase = store_ref.scan_phase.clone();
                                    let found = store_ref.scan_items_found;
                                    let bytes = store_ref.scan_bytes_found;
                                    let path = store_ref.scan_current_path.clone();
                                    this.flex()
                                        .items_center()
                                        .justify_center()
                                        .child(ui::empty_state_loading(
                                            i18n.scanning_title(),
                                            i18n.scan_progress_detail(
                                                &phase,
                                                found,
                                                bytes,
                                                path.as_deref(),
                                            ),
                                        ))
                                })
                                .when(has_report && count > 0 && !scanning, |this| {
                                    let items = items.clone();
                                    let store = store.clone();
                                    let i18n = i18n;
                                    this.child(ui::uniform_list_pane(
                                        "cleanup-rows",
                                        count,
                                        scroll_handle,
                                        cx,
                                        move |this, visible_range, window, cx| {
                                            visible_range
                                                .filter_map(|ix| {
                                                    items.get(ix).map(|item| {
                                                        this.render_row(
                                                            item,
                                                            expert,
                                                            &i18n,
                                                            lang,
                                                            store.clone(),
                                                            window,
                                                            cx,
                                                        )
                                                    })
                                                })
                                                .collect()
                                        },
                                    ))
                                }),
                        ),
                    )
                    .child(ui::panel_divider())
                    .child(
                        h_flex()
                            .flex_shrink_0()
                            .p_4()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors::text_secondary())
                                    .child(i18n.selected_summary(selected_count, selected_bytes)),
                            ),
                    ),
            )
    }
}

fn open_item_detail_dialog(
    window: &mut Window,
    cx: &mut App,
    name: &str,
    description: &str,
    path: &Path,
) {
    let name: SharedString = name.to_string().into();
    let description: SharedString = description.to_string().into();
    let path: SharedString = path.display().to_string().into();
    window.open_dialog(cx, move |dialog, _window, _cx| {
        dialog
            .title(name.clone())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_base()
                            .text_color(colors::text_secondary())
                            .child(description.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::text_muted())
                            .child(path.clone()),
                    ),
            )
            .alert()
            .on_ok(|_, _, _| true)
    });
}

fn cleanup_scan_prompt(store: &Entity<AppStore>, i18n: &I18n, cx: &App) -> Div {
    let store = store.clone();
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_5()
        .py_12()
        .child(
            div()
                .size(px(72.))
                .rounded(corner_md())
                .bg(colors::accent_blue_bg())
                .flex()
                .items_center()
                .justify_center()
                .child(
                    gpui_component::Icon::new(ui::EMPTY_SCAN)
                        .with_size(px(36.))
                        .text_color(colors::accent_blue()),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::text_primary())
                        .child(i18n.no_scan_results_title()),
                )
                .child(
                    div()
                        .text_base()
                        .text_color(colors::text_secondary())
                        .child(i18n.no_scan_results_hint()),
                ),
        )
        .child(
            ui::action_button(
                "cleanup-first-scan",
                i18n.scan_now_short(),
                Some(ui::ACTION_SCAN),
                true,
                cx,
            )
            .on_click(move |_, _, cx| {
                store.update(cx, |s, cx| s.start_scan(cx));
            }),
        )
}

fn clickable_folder_path(
    id: impl Into<SharedString>,
    path: &Path,
    label: impl Into<SharedString>,
) -> impl IntoElement {
    let open_path = folder_open_target(path);
    let label: SharedString = label.into();
    let id: SharedString = id.into();

    h_flex()
        .id(id)
        .gap_2()
        .items_center()
        .flex_1()
        .min_w_0()
        .cursor_pointer()
        .on_click(move |_, _, _| {
            open::that(&open_path).ok();
        })
        .hover(|s| s.bg(colors::accent_blue_bg_hover().opacity(0.45)))
        .active(|s| s.bg(colors::accent_blue_bg_pressed().opacity(0.65)))
        .rounded(corner_md())
        .px_2()
        .py_1()
        .child(
            Icon::new(ui::ACTION_OPEN_FOLDER)
                .with_size(px(18.))
                .text_color(colors::accent_blue()),
        )
        .child(ui::middle_truncated_path(label))
}

fn folder_open_target(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf())
    }
}

fn filter_btn(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    filter: CleanupFilter,
    store: &Entity<AppStore>,
    scroll_handle: &UniformListScrollHandle,
    cx: &App,
) -> Button {
    let active = store.read(cx).cleanup_filter == filter;
    let store = store.clone();
    let scroll_handle = scroll_handle.clone();
    let id: SharedString = id.into();
    let label: SharedString = label.into();
    ui::ghost_pill(id, label, active, cx).on_click(move |_, _, cx| {
        store.update(cx, |s, cx| {
            s.cleanup_filter = filter;
            cx.notify();
        });
        scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
    })
}
