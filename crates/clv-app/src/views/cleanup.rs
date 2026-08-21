use crate::app::state::{AppStore, CleanupFilter};
use crate::prelude::*;
use crate::theme::{colors, corner_md};
use clv_core::format_bytes;
use gpui_component::Icon;
use std::path::{Path, PathBuf};

/// Path row height — keeps checkbox, path, and action button vertically centered together.
const CLEANUP_PATH_ROW_H: f32 = 36.;

pub struct CleanupView {
    store: Entity<AppStore>,
}

impl CleanupView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self { store }
    }
}

impl Render for CleanupView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store_ref = self.store.read(cx);
        let items = store_ref.filtered_items();
        let selected_bytes = store_ref.selected_bytes();
        let selected_count = store_ref.selected_items().len();
        let has_report = store_ref.last_report.is_some();
        let scanning = store_ref.scanning;
        let cleaning = store_ref.cleaning;
        let expert = store_ref.settings.expert_mode;
        let store = self.store.clone();

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
                            .child("筛选"),
                    )
                    .child(filter_btn("filter-all", "全部", CleanupFilter::All, &store, cx))
                    .child(filter_btn(
                        "filter-safe",
                        "仅安全项",
                        CleanupFilter::SafeOnly,
                        &store,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-project",
                        "项目构建缓存",
                        CleanupFilter::ProjectBuildCache,
                        &store,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-shared",
                        "工具下载缓存",
                        CleanupFilter::SharedToolCache,
                        &store,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-dev-env",
                        "环境与依赖",
                        CleanupFilter::DevEnvironment,
                        &store,
                        cx,
                    ))
                    .child(filter_btn(
                        "filter-ai",
                        "AI 工具数据",
                        CleanupFilter::AiGenerated,
                        &store,
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
                    .child(
                        h_flex()
                            .p_4()
                            .gap_3()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        ui::ghost_pill("select-all", "全选", false, cx).on_click({
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
                                        ui::ghost_pill("deselect-all", "取消全选", false, cx).on_click({
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
                                                format!("{} 项 · 已选 {selected_count} 项", items.len())
                                            } else {
                                                "尚未扫描".into()
                                            }),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        ui::action_button(
                                            "rescan",
                                            if has_report { "重新扫描" } else { "开始扫描" },
                                            Some(ui::ACTION_SCAN),
                                            false,
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
                                            "清理选中项",
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
                                                let store_confirm = store.clone();
                                                window.open_dialog(cx, move |dialog, _window, _cx| {
                                                    dialog
                                                        .title("确认清理")
                                                        .child(format!(
                                                            "即将清理 {count} 项，预计释放 {}",
                                                            format_bytes(bytes)
                                                        ))
                                                        .confirm()
                                                        .on_ok({
                                                            let store = store_confirm.clone();
                                                            move |_, window, cx| {
                                                                store.update(cx, |s, cx| {
                                                                    s.run_cleanup(cx);
                                                                });
                                                                window.close_dialog(cx);
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
                    .child(ui::scroll_y(
                        div()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .when(!has_report && !scanning, |this| {
                                this.child(cleanup_scan_prompt(&store, cx))
                            })
                            .when(has_report && items.is_empty() && !scanning, |this| {
                                this.child(ui::empty_state(
                                    ui::EMPTY_SCAN,
                                    "当前筛选下暂无项目",
                                    "尝试切换左侧筛选条件，或重新扫描",
                                ))
                            })
                            .when(scanning, |this| {
                                let phase = store_ref.scan_phase.clone();
                                let found = store_ref.scan_items_found;
                                let bytes = store_ref.scan_bytes_found;
                                let path = store_ref.scan_current_path.clone();
                                this.child(ui::empty_state_loading(
                                    "正在扫描",
                                    if let Some(p) = path {
                                        format!(
                                            "{phase} · 已发现 {found} 项（{bytes}）\n{p}",
                                            bytes = format_bytes(bytes)
                                        )
                                    } else {
                                        format!(
                                            "{phase} · 已发现 {found} 项（{bytes}）",
                                            bytes = format_bytes(bytes)
                                        )
                                    },
                                ))
                            })
                            .children(items.iter().map(|item| {
                                let id = item.id.clone();
                                let expanded = store_ref.expanded_item.as_deref() == Some(&id);
                                let path_display = if expert {
                                    item.path.display().to_string()
                                } else {
                                    item.project_root
                                        .as_ref()
                                        .map(|p| p.display().to_string())
                                        .unwrap_or_else(|| item.name.clone())
                                };
                                let store_toggle = store.clone();
                                let store_expand = store.clone();
                                let item_path = item.path.clone();
                                let item_name = item.name.clone();
                                let item_description = item.description.clone();
                                let cb_id = eid(format!("cb-{id}"));
                                let exp_id = eid(format!("exp-{id}"));
                                let path_id = eid(format!("path-{id}"));

                                ui::card()
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
                                                            .on_click({
                                                                let id = id.clone();
                                                                let store = store_toggle.clone();
                                                                move |checked, _, cx| {
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
                                                            }),
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
                                                            .child(ui::risk_badge(item.risk))
                                                            .child(
                                                                div()
                                                                    .text_base()
                                                                    .text_color(colors::text_muted())
                                                                    .child(item.stack.label()),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_base()
                                                                    .text_color(colors::text_muted())
                                                                    .child(item.category.clone()),
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
                                                        ui::std_button(
                                                            Button::new(exp_id)
                                                                .ghost()
                                                                .label(if expanded { "收起" } else { "详情" }),
                                                        )
                                                        .on_click({
                                                            let id = id.clone();
                                                            let store = store_expand.clone();
                                                            move |_, _, cx| {
                                                                store.update(cx, |s, cx| {
                                                                    if s.expanded_item.as_deref() == Some(&id) {
                                                                        s.expanded_item = None;
                                                                    } else {
                                                                        s.expanded_item = Some(id.clone());
                                                                    }
                                                                    cx.notify();
                                                                });
                                                            }
                                                        }),
                                                    ),
                                            ),
                                    )
                                    .when(expanded, |this| {
                                        this.child(
                                            div()
                                                .ml(px(36.))
                                                .pt_3()
                                                .mt_1()
                                                .border_t_1()
                                                .border_color(colors::panel_divider())
                                                .flex()
                                                .flex_col()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(colors::text_primary())
                                                        .child(item_name),
                                                )
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .text_color(colors::text_secondary())
                                                        .child(item_description),
                                                ),
                                        )
                                    })
                            })),
                    ))
                    .child(ui::panel_divider())
                    .child(
                        h_flex()
                            .p_4()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors::text_secondary())
                                    .child(format!(
                                        "已选 {selected_count} 项 · 预计释放 {}",
                                        format_bytes(selected_bytes)
                                    )),
                            ),
                    ),
            )
    }
}

fn cleanup_scan_prompt(store: &Entity<AppStore>, cx: &App) -> Div {
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
                        .child("还没有扫描结果"),
                )
                .child(
                    div()
                        .text_base()
                        .text_color(colors::text_secondary())
                        .child("扫描后将列出可安全清理的缓存与构建产物"),
                ),
        )
        .child(
            ui::action_button("cleanup-first-scan", "立即扫描", Some(ui::ACTION_SCAN), true, cx)
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
        .hover(|s| s.bg(colors::accent_blue_bg().opacity(0.45)))
        .rounded(corner_md())
        .px_2()
        .py_1()
        .child(
            Icon::new(ui::ACTION_OPEN_FOLDER)
                .with_size(px(18.))
                .text_color(colors::accent_blue()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_base()
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors::text_primary())
                .overflow_hidden()
                .child(label),
        )
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
    cx: &App,
) -> Button {
    let active = store.read(cx).cleanup_filter == filter;
    let store = store.clone();
    let id: SharedString = id.into();
    let label: SharedString = label.into();
    ui::ghost_pill(id, label, active, cx).on_click(move |_, _, cx| {
        store.update(cx, |s, cx| {
            s.cleanup_filter = filter;
            cx.notify();
        });
    })
}
