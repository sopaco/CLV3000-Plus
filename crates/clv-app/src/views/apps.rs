//! App uninstaller view: app list (left) + related-residue detail (right).
//!
//! Interaction model (PRD §5.1): select an app → review its grouped residue →
//! confirm → the bundle plus checked items move to the recycle area through
//! the shared cleanup pipeline.

use crate::app::state::AppStore;
use crate::i18n::{self, I18n};
use crate::prelude::*;
use crate::theme::{colors, corner};
use clv_core::{
    AppEntry, AppScanReport, AppSource, CleanupCategory, MatchQuality, RelatedItem, RiskLevel,
    ScanItem, format_bytes,
};
use gpui_kit::component::dialog::DialogButtonProps;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{AnyElement, ScrollStrategy, Stateful, Subscription, UniformListScrollHandle};
use std::path::Path;

const APP_ROW_H: f32 = 76.;

/// Left-column row selector. Broken CLI links share the app list as a
/// trailing pseudo-entry (PRD §5.1 source badge "失效链接").
#[derive(Debug, Clone, PartialEq, Eq)]
enum RowKey {
    App(String),
    StrayLinks,
}

pub struct AppsView {
    store: Entity<AppStore>,
    search_query: String,
    search_input: Option<Entity<InputState>>,
    _search_subscription: Option<Subscription>,
    scroll_handle: UniformListScrollHandle,
}

impl AppsView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self {
            store,
            search_query: String::new(),
            search_input: None,
            _search_subscription: None,
            scroll_handle: UniformListScrollHandle::new(),
        }
    }

    fn ensure_search_input(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        let placeholder = self.store.read(cx).i18n().apps_search_placeholder();
        ui::ensure_search_input(
            &mut self.search_input,
            &mut self._search_subscription,
            placeholder,
            window,
            cx,
            |view, value, _cx| {
                view.search_query = value;
                view.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
            },
        )
    }

    /// Rows shown in the left list, respecting the search query.
    fn visible_rows(
        &self,
        report: Option<&AppScanReport>,
        lang: clv_core::Language,
    ) -> Vec<RowKey> {
        let Some(report) = report else {
            return Vec::new();
        };
        let q = self.search_query.trim().to_lowercase();
        let mut rows: Vec<RowKey> = report
            .apps
            .iter()
            .filter(|app| {
                q.is_empty()
                    || app.name.to_lowercase().contains(&q)
                    || app
                        .bundle_id
                        .as_deref()
                        .is_some_and(|id| id.to_lowercase().contains(&q))
            })
            .map(|app| RowKey::App(app.id.clone()))
            .collect();
        let links_label = i18n::scan_category_label(lang, CleanupCategory::StrayExecutable);
        if !report.stray_executables.is_empty()
            && (q.is_empty() || links_label.to_lowercase().contains(&q))
        {
            rows.push(RowKey::StrayLinks);
        }
        rows
    }
}

impl Render for AppsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let search_input = self.ensure_search_input(window, cx);
        let store_ref = self.store.read(cx);
        let i18n = store_ref.i18n();
        let lang = store_ref.language();
        let scanning = store_ref.app_scanning;
        let report = store_ref.app_report.clone();
        let expert = store_ref.settings.expert_mode;
        let app_scan_apps_found = store_ref.app_scan_apps_found;
        let show_stray_links = store_ref.show_stray_links;
        let selected_app_id = store_ref.selected_app_id.clone();

        let rows = self.visible_rows(report.as_ref(), lang);
        let has_content = report
            .as_ref()
            .is_some_and(|r| !r.apps.is_empty() || !r.stray_executables.is_empty());
        let selected_key = if show_stray_links
            && report
                .as_ref()
                .is_some_and(|r| !r.stray_executables.is_empty())
        {
            RowKey::StrayLinks
        } else {
            report
                .as_ref()
                .and_then(|r| {
                    r.apps
                        .iter()
                        .find(|a| Some(a.id.as_str()) == selected_app_id.as_deref())
                        .or_else(|| r.apps.first())
                })
                .map(|a| RowKey::App(a.id.clone()))
                .unwrap_or(RowKey::StrayLinks)
        };
        let store = self.store.clone();
        let scroll_handle = self.scroll_handle.clone();

        let summary = match &report {
            Some(r) if has_content => {
                let bytes = r.related_residue_bytes()
                    + r.apps
                        .iter()
                        .filter(|a| a.risk != RiskLevel::Protected)
                        .map(|a| a.binary_size_bytes)
                        .sum::<u64>();
                format!(
                    "{} · {}",
                    i18n.apps_count(r.apps.len()),
                    i18n.apps_reclaimable(&format_bytes(bytes)),
                )
            }
            _ => i18n.apps_subtitle().to_string(),
        };
        let last_scanned = report
            .as_ref()
            .and_then(|r| r.scanned_at)
            .map(|at| {
                let local: chrono::DateTime<chrono::Local> = at.into();
                i18n.apps_last_scanned(&local.format("%Y-%m-%d %H:%M").to_string())
            })
            .unwrap_or_default();

        div()
            .id("apps-view")
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .overflow_hidden()
            .child(
                // Top bar: banner + search + rescan.
                div()
                    .flex_shrink_0()
                    .px_6()
                    .pt_6()
                    .pb_4()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(ui::page_banner(i18n.apps_title(), summary))
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .gap_4()
                            .child(
                                h_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .w(px(280.))
                                            .flex_shrink_0()
                                            .child(ui::std_input(Input::new(&search_input))),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(colors::text_muted())
                                            .truncate()
                                            .child(last_scanned),
                                    ),
                            )
                            .child(
                                ui::action_button(
                                    "apps-scan",
                                    if scanning {
                                        i18n.app_scanning_status(app_scan_apps_found)
                                    } else {
                                        i18n.apps_rescan().to_string()
                                    },
                                    Some(ui::ACTION_SCAN),
                                    true,
                                    cx,
                                )
                                .disabled(scanning)
                                .on_click({
                                    let store = store.clone();
                                    move |_, _, cx| {
                                        store.update(cx, |s, cx| s.start_app_scan(cx));
                                    }
                                }),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .flex()
                    .gap_4()
                    .px_6()
                    .pb_6()
                    .overflow_hidden()
                    .when(!has_content, |this| {
                        this.items_center().justify_center().child(ui::empty_state(
                            ui::EMPTY_AGENT,
                            if scanning {
                                i18n.app_scan_start_message()
                            } else {
                                i18n.apps_empty_title()
                            },
                            i18n.apps_empty_hint(),
                        ))
                    })
                    .when(has_content, |this| {
                        let detail = match (&report, &selected_key) {
                            (Some(r), key) => self.render_detail(r, key, &i18n, lang, expert, cx),
                            _ => div().into_any_element(),
                        };
                        let rows_for_list = rows.clone();
                        let i18n_list = i18n;
                        let lang_list = lang;
                        let store_list = store.clone();
                        let selected_for_list = selected_key.clone();
                        this.child(
                            // Left: app list.
                            div()
                                .w(px(340.))
                                .flex_shrink_0()
                                .h_full()
                                .min_h_0()
                                .flex()
                                .flex_col()
                                .child(ui::uniform_list_pane(
                                    "apps-rows",
                                    rows_for_list.len(),
                                    scroll_handle,
                                    cx,
                                    move |this, visible_range, _window, cx| {
                                        let store_snapshot = this.store.read(cx);
                                        let report = store_snapshot.app_report.clone();
                                        let expert = store_snapshot.settings.expert_mode;
                                        visible_range
                                            .filter_map(|ix| {
                                                let key = rows_for_list.get(ix)?;
                                                let report = report.as_ref()?;
                                                Some(render_list_row(
                                                    report,
                                                    key,
                                                    *key == selected_for_list,
                                                    &i18n_list,
                                                    lang_list,
                                                    expert,
                                                    store_list.clone(),
                                                ))
                                            })
                                            .collect()
                                    },
                                )),
                        )
                        .child(
                            // Right: detail pane.
                            div()
                                .flex_1()
                                .min_w_0()
                                .h_full()
                                .min_h_0()
                                .flex()
                                .flex_col()
                                .child(detail),
                        )
                    }),
            )
    }
}

/// Reads `show_stray_links` without keeping a store borrow across the
/// render tree (helpers re-read state at click time).
impl AppsView {
    fn render_detail(
        &self,
        report: &AppScanReport,
        key: &RowKey,
        i18n: &I18n,
        lang: clv_core::Language,
        expert: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let app = match key {
            RowKey::App(id) => report.apps.iter().find(|a| &a.id == id),
            RowKey::StrayLinks => None,
        };
        match app {
            Some(app) => self.render_app_detail(app, i18n, lang, expert, cx),
            None => self.render_stray_links_detail(report, i18n, lang, cx),
        }
    }

    fn render_app_detail(
        &self,
        app: &AppEntry,
        i18n: &I18n,
        lang: clv_core::Language,
        expert: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let store = self.store.clone();
        let selected_items = self.store.read(cx).selected_app_items(app);
        let selected_bytes: u64 = selected_items.iter().map(|i| i.size_bytes).sum();
        let binary_included =
            app.app_path.is_some() && !app.is_running && app.risk != RiskLevel::Protected;

        // Group residue by category (fuzzy rows are expert-only, protected
        // rows hidden outside expert mode — same rule as the cleanup view).
        let mut groups: Vec<(CleanupCategory, Vec<&RelatedItem>)> = Vec::new();
        for related in &app.related_items {
            if related.quality == MatchQuality::Fuzzy && !expert {
                continue;
            }
            if related.item.risk == RiskLevel::Protected && !expert {
                continue;
            }
            match groups.iter_mut().find(|(c, _)| *c == related.item.category) {
                Some((_, items)) => items.push(related),
                None => groups.push((related.item.category, vec![related])),
            }
        }

        let cta_enabled = !selected_items.is_empty() || binary_included;

        div()
            .id("app-detail")
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .child(ui::scroll_y(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .pb_4()
                    // Info card.
                    .child(self.render_info_card(app, i18n, lang, expert))
                    .when(app.is_running, |this| {
                        this.child(running_banner(&app.name, i18n))
                    })
                    // Residue groups.
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::text_secondary())
                            .child(i18n.app_related_header()),
                    )
                    .when(groups.is_empty(), |this| {
                        this.child(
                            div()
                                .text_base()
                                .text_color(colors::text_muted())
                                .child(i18n.app_related_empty()),
                        )
                    })
                    .children(groups.iter().map(|(category, items)| {
                        render_group(app, *category, items, i18n, lang, &store, cx)
                    })),
            ))
            // Footer: summary + CTA.
            .child(
                h_flex()
                    .flex_shrink_0()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .pt_3()
                    .border_t_1()
                    .border_color(colors::panel_divider())
                    .child(div().text_sm().text_color(colors::text_secondary()).child(
                        i18n.app_selected_summary(
                            selected_items.len(),
                            &format_bytes(selected_bytes),
                        ),
                    ))
                    .child(
                        h_flex()
                            .items_center()
                            .gap_3()
                            // Windows: launch the official uninstaller first.
                            .when(app.uninstall_string.is_some(), |this| {
                                this.child(
                                    ui::action_button(
                                        eid(format!("app-run-uninstaller-{}", app.id)),
                                        i18n.app_run_uninstaller_cta(),
                                        None,
                                        false,
                                        cx,
                                    )
                                    .on_click({
                                        let store = store.clone();
                                        let app = app.clone();
                                        move |_, _, cx| {
                                            let app = app.clone();
                                            store.update(cx, |s, cx| {
                                                s.run_app_uninstaller(app, cx);
                                            });
                                        }
                                    }),
                                )
                            })
                            .child(
                                ui::action_button(
                                    eid(format!("app-cta-{}", app.id)),
                                    if binary_included {
                                        i18n.app_uninstall_cta()
                                    } else {
                                        i18n.app_clean_residue_cta()
                                    },
                                    Some(ui::ACTION_CLEAN),
                                    true,
                                    cx,
                                )
                                .disabled(!cta_enabled)
                                .on_click({
                                    let store = store.clone();
                                    let app = app.clone();
                                    let i18n = *i18n;
                                    move |_, window, cx| {
                                        confirm_app_cleanup(&store, &app, &i18n, window, cx);
                                    }
                                }),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_info_card(
        &self,
        app: &AppEntry,
        i18n: &I18n,
        lang: clv_core::Language,
        expert: bool,
    ) -> Div {
        let mut rows = v_flex().flex_col().gap_1();
        if let Some(version) = &app.version {
            rows = rows.child(info_line(i18n.app_version_label(), version.clone()));
        }
        if let Some(publisher) = &app.publisher {
            rows = rows.child(info_line(i18n.app_publisher_label(), publisher.clone()));
        }
        // Bundle id and the full path are expert-only fields (PRD §5.4).
        if expert {
            if let Some(bundle_id) = &app.bundle_id {
                rows = rows.child(info_line("Bundle ID", bundle_id.clone()));
            }
            if let Some(path) = &app.app_path {
                rows = rows.child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .min_w_0()
                        .child(
                            div()
                                .flex_shrink_0()
                                .w(px(72.))
                                .text_sm()
                                .text_color(colors::text_muted())
                                .child(i18n.app_path_label()),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_sm()
                                .text_color(colors::text_secondary())
                                .child(path.display().to_string()),
                        )
                        .child(ui::open_path_button(
                            eid(format!("app-open-{}", app.id)),
                            path,
                            i18n,
                        )),
                );
            }
        }
        rows = rows
            .child(info_line(i18n.app_total_size_label(), app.size_human()))
            // Orphan entities have no bundle; hide the zero-size row.
            .when(app.binary_size_bytes > 0, |this| {
                this.child(info_line(
                    i18n.app_bundle_size_label(),
                    format_bytes(app.binary_size_bytes),
                ))
            })
            .child(info_line(
                i18n.app_residue_size_label(),
                format_bytes(app.residue_bytes()),
            ));
        let _ = lang;

        ui::glass_card()
            .p_4()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .justify_between()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::text_primary())
                            .truncate()
                            .child(app.name.clone()),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(source_badge(app.source, app.app_path.as_deref(), i18n))
                            .child(ui::risk_badge(app.risk, lang))
                            .when(app.is_running, |this| {
                                this.child(badge(
                                    i18n.app_running_badge(),
                                    colors::risk_safe_bg(),
                                    colors::risk_safe_border(),
                                    colors::green(),
                                ))
                            }),
                    ),
            )
            .child(rows)
    }

    fn render_stray_links_detail(
        &self,
        report: &AppScanReport,
        i18n: &I18n,
        lang: clv_core::Language,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let store = self.store.clone();
        let links = &report.stray_executables;
        let selected = self.store.read(cx).selected_stray_executables();
        let cta_enabled = !selected.is_empty();

        div()
            .id("stray-links-detail")
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .child(ui::scroll_y(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .pb_4()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::text_secondary())
                            .child(i18n::scan_category_label(
                                lang,
                                CleanupCategory::StrayExecutable,
                            )),
                    )
                    .children(
                        links
                            .iter()
                            .map(|item| stray_link_row(item, i18n, lang, &store, cx)),
                    ),
            ))
            .child(
                h_flex()
                    .flex_shrink_0()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .pt_3()
                    .border_t_1()
                    .border_color(colors::panel_divider())
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::text_secondary())
                            .child(i18n.app_selected_summary(selected.len(), &format_bytes(0))),
                    )
                    .child(
                        ui::action_button(
                            "stray-links-cta",
                            i18n.app_clean_links_cta(),
                            Some(ui::ACTION_CLEAN),
                            true,
                            cx,
                        )
                        .disabled(!cta_enabled)
                        .on_click({
                            let store = store.clone();
                            let i18n = *i18n;
                            move |_, window, cx| {
                                let items = store.read(cx).selected_stray_executables();
                                confirm_items_cleanup(&items, &i18n, window, cx, &store);
                            }
                        }),
                    ),
            )
            .into_any_element()
    }
}

fn info_line(label: &str, value: String) -> Div {
    h_flex()
        .items_center()
        .gap_2()
        .min_w_0()
        .child(
            div()
                .flex_shrink_0()
                .w(px(72.))
                .text_sm()
                .text_color(colors::text_muted())
                .child(label.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_sm()
                .text_color(colors::text_secondary())
                .child(value),
        )
}

fn badge(text: &str, bg: gpui_kit::Hsla, border: gpui_kit::Hsla, fg: gpui_kit::Hsla) -> Div {
    h_flex()
        .items_center()
        .flex_shrink_0()
        .px(px(8.))
        .py(px(2.))
        .rounded(corner())
        .bg(bg)
        .border_1()
        .border_color(border)
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(fg)
                .child(text.to_string()),
        )
}

fn source_badge(source: AppSource, app_path: Option<&Path>, i18n: &I18n) -> Div {
    let (label, fg) = match source {
        AppSource::Applications | AppSource::UserApplications => {
            (i18n.app_source_installed(), colors::accent_blue())
        }
        AppSource::StrayBundle | AppSource::Orphan => {
            (i18n.app_source_stray(), colors::warn_orange())
        }
        AppSource::Registry => (i18n.app_source_installed(), colors::accent_blue()),
    };
    let _ = app_path;
    badge(label, colors::accent_blue_bg(), colors::glass_border(), fg)
}

fn running_banner(name: &str, i18n: &I18n) -> Div {
    ui::glass_card()
        .p_3()
        .flex()
        .items_center()
        .gap_2()
        .bg(colors::risk_caution_bg())
        .border_1()
        .border_color(colors::risk_caution_border())
        .child(
            div()
                .text_sm()
                .text_color(colors::risk_caution_fg())
                .child(format!("{name} · {}", i18n.app_running_hint())),
        )
}

// ── Left list rows ────────────────────────────────────────────────────────────

fn render_list_row(
    report: &AppScanReport,
    key: &RowKey,
    selected: bool,
    i18n: &I18n,
    lang: clv_core::Language,
    expert: bool,
    store: Entity<AppStore>,
) -> Div {
    let select_style = |this: Stateful<Div>| {
        if selected {
            this.border_1()
                .border_color(colors::accent_blue())
                .bg(colors::accent_blue_bg_hover().opacity(0.35))
        } else {
            this
        }
    };

    match key {
        RowKey::App(id) => {
            let Some(app) = report.apps.iter().find(|a| &a.id == id) else {
                return div().h(px(APP_ROW_H));
            };
            let row_id = eid(format!("app-row-{}", app.id));
            let card = select_style(
                ui::soft_card()
                    .id(row_id)
                    .w_full()
                    .h(px(APP_ROW_H - 8.))
                    .px_3()
                    .py_2()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap_1(),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::text_primary())
                            .child(app.name.clone()),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_sm()
                            .text_color(colors::accent_blue())
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(app.size_human()),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .child(source_badge(app.source, app.app_path.as_deref(), i18n))
                    .when(app.is_running, |this| {
                        this.child(
                            div()
                                .flex_shrink_0()
                                .text_sm()
                                .text_color(colors::green())
                                .child(i18n.app_running_badge()),
                        )
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_sm()
                            .text_color(colors::text_muted())
                            .child(app.version.clone().unwrap_or_else(|| {
                                // Bundle id is an expert-only field (PRD §5.4).
                                if expert {
                                    app.bundle_id.clone().unwrap_or_default()
                                } else {
                                    String::new()
                                }
                            })),
                    ),
            );
            selectable_row(card, key.clone(), store)
        }
        RowKey::StrayLinks => {
            let count = report.stray_executables.len();
            let row_id = eid("app-row-stray-links");
            let card = select_style(
                ui::soft_card()
                    .id(row_id)
                    .w_full()
                    .h(px(APP_ROW_H - 8.))
                    .px_3()
                    .py_2()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap_1(),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .min_w_0()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::text_primary())
                            .child(i18n::scan_category_label(
                                lang,
                                CleanupCategory::StrayExecutable,
                            )),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_sm()
                            .text_color(colors::text_muted())
                            .child(format!("×{count}")),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(colors::text_muted())
                    .truncate()
                    .child(
                        report
                            .stray_executables
                            .first()
                            .and_then(|i| i.path.parent())
                            .map(|p| p.display().to_string())
                            .unwrap_or_default(),
                    ),
            );
            selectable_row(card, key.clone(), store)
        }
    }
}

fn selectable_row(card: Stateful<Div>, key: RowKey, store: Entity<AppStore>) -> Div {
    div()
        .w_full()
        .h(px(APP_ROW_H))
        .pb_2()
        .child(card.on_click(move |_, _, cx| {
            store.update(cx, |s, cx| match &key {
                RowKey::App(id) => {
                    s.show_stray_links = false;
                    if s.selected_app_id.as_deref() != Some(id.as_str()) {
                        s.selected_app_id = Some(id.clone());
                        cx.notify();
                    }
                }
                RowKey::StrayLinks => {
                    if !s.show_stray_links {
                        s.show_stray_links = true;
                        cx.notify();
                    }
                }
            });
        }))
}

// ── Residue groups ───────────────────────────────────────────────────────────

fn render_group(
    app: &AppEntry,
    category: CleanupCategory,
    items: &[&RelatedItem],
    i18n: &I18n,
    lang: clv_core::Language,
    store: &Entity<AppStore>,
    cx: &App,
) -> Div {
    let group_bytes: u64 = items.iter().map(|r| r.item.size_bytes).sum();
    let all_selected = items
        .iter()
        .all(|r| store.read(cx).is_app_item_selected(&r.item.id));

    let header_id = eid(format!("group-cb-{}-{:?}", app.id, category));

    ui::glass_card()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            h_flex()
                .items_center()
                .gap_3()
                .child(
                    Checkbox::new(header_id)
                        .checked(all_selected)
                        .cursor_pointer()
                        .on_click({
                            let store = store.clone();
                            let app = app.clone();
                            move |checked, _, cx| {
                                store.update(cx, |s, cx| {
                                    s.set_app_group_selected(&app, category, *checked);
                                    cx.notify();
                                });
                            }
                        }),
                )
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::text_primary())
                        .child(i18n::scan_category_label(lang, category)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_muted())
                        .child(format!("{} · {}", items.len(), format_bytes(group_bytes))),
                ),
        )
        .when(category == CleanupCategory::AppContainer, |this| {
            this.child(
                div()
                    .text_sm()
                    .text_color(colors::warn_orange())
                    .child(i18n.app_container_protected_hint()),
            )
        })
        .children(
            items
                .iter()
                .map(|related| related_item_row(related, i18n, lang, store, cx)),
        )
}

fn related_item_row(
    related: &RelatedItem,
    i18n: &I18n,
    lang: clv_core::Language,
    store: &Entity<AppStore>,
    cx: &App,
) -> Div {
    let item = &related.item;
    let selected = store.read(cx).is_app_item_selected(&item.id);
    let cb_id = eid(format!("app-item-cb-{}", item.id));
    let item_id = item.id.clone();
    let path = item.path.clone();
    let name = item.name.clone();
    let size = format_bytes(item.size_bytes);
    let fuzzy = related.quality == MatchQuality::Fuzzy;

    let row = h_flex()
        .id(eid(format!("app-item-row-{}", item.id)))
        .w_full()
        .p_2()
        .rounded(corner())
        .border_1()
        .border_color(colors::glass_border())
        .bg(colors::glass_bg_soft())
        .items_center()
        .gap_3()
        .child(
            div().flex_shrink_0().flex().items_center().child(
                Checkbox::new(cb_id)
                    .checked(selected)
                    .cursor_pointer()
                    .on_click({
                        let store = store.clone();
                        let item_id = item_id.clone();
                        move |checked, _, cx| {
                            store.update(cx, |s, cx| {
                                s.set_app_item_selected(&item_id, *checked);
                                cx.notify();
                            });
                        }
                    }),
            ),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_1()
                .child(
                    div()
                        .truncate()
                        .text_sm()
                        .text_color(colors::text_primary())
                        .child(name),
                )
                .child(
                    div()
                        .truncate()
                        .text_sm()
                        .text_color(colors::text_muted())
                        .child(path.display().to_string()),
                ),
        )
        .child(
            h_flex()
                .flex_shrink_0()
                .items_center()
                .gap_2()
                .when(fuzzy, |this| {
                    this.child(badge(
                        i18n.app_fuzzy_badge(),
                        colors::accent_blue_bg(),
                        colors::glass_border(),
                        colors::text_secondary(),
                    ))
                })
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_secondary())
                        .child(size),
                )
                .child(ui::risk_badge(item.risk, lang)),
        );

    // Wrap the context menu in an owned div: `ContextMenu<E>` composes as a
    // child element and the wrapping keeps the row's layout contract (`Div`).
    // Registry keys carry a synthetic path with no folder to reveal.
    if item.category == CleanupCategory::RegistryLeftover {
        div().w_full().child(row)
    } else {
        div().w_full().child(ui::ctx_menu(row, &path, i18n))
    }
}

fn stray_link_row(
    item: &ScanItem,
    i18n: &I18n,
    lang: clv_core::Language,
    store: &Entity<AppStore>,
    cx: &App,
) -> Div {
    related_item_row(
        &RelatedItem {
            item: item.clone(),
            quality: MatchQuality::BundleId,
        },
        i18n,
        lang,
        store,
        cx,
    )
}

// ── Confirm + execute ────────────────────────────────────────────────────────

fn confirm_app_cleanup(
    store: &Entity<AppStore>,
    app: &AppEntry,
    i18n: &I18n,
    window: &mut Window,
    cx: &mut App,
) {
    let selected = store.read(cx).selected_app_items(app);
    let include_binary =
        app.app_path.is_some() && !app.is_running && app.risk != RiskLevel::Protected;
    let mut items = selected;
    if include_binary {
        if let Some(binary) = app.binary_scan_item() {
            items.insert(0, binary);
        }
    }
    if items.is_empty() {
        return;
    }
    let title = if include_binary {
        i18n.app_confirm_uninstall_title()
    } else {
        i18n.app_confirm_cleanup_title()
    };
    let bytes: u64 = items.iter().map(|i| i.size_bytes).sum();
    let body = if include_binary {
        i18n.app_confirm_uninstall_body(
            &app.name,
            items.len().saturating_sub(1),
            &format_bytes(bytes),
        )
    } else {
        i18n.app_confirm_cleanup_body(items.len(), &format_bytes(bytes))
    };
    let ok_text = i18n.dialog_ok();
    let cancel_text = i18n.dialog_cancel();
    let store_confirm = store.clone();

    window.open_alert_dialog(cx, move |alert, _window, _cx| {
        // The builder is `Fn` (may run more than once); clone captured
        // values here so `.on_ok` can take ownership.
        let items = items.clone();
        let store_confirm = store_confirm.clone();
        alert
            .title(title)
            .child(
                div()
                    .text_base()
                    .text_color(colors::text_secondary())
                    .child(body.clone()),
            )
            .button_props(
                DialogButtonProps::default()
                    .ok_text(ok_text)
                    .cancel_text(cancel_text)
                    .show_cancel(true),
            )
            .on_ok(move |_, _, cx| {
                store_confirm.update(cx, |s, cx| {
                    s.cleanup_paths(items.clone(), cx);
                });
                true
            })
    });
}

fn confirm_items_cleanup(
    items: &[ScanItem],
    i18n: &I18n,
    window: &mut Window,
    cx: &mut App,
    store: &Entity<AppStore>,
) {
    if items.is_empty() {
        return;
    }
    let title = i18n.app_confirm_cleanup_title();
    let bytes: u64 = items.iter().map(|i| i.size_bytes).sum();
    let body = i18n.app_confirm_cleanup_body(items.len(), &format_bytes(bytes));
    let ok_text = i18n.dialog_ok();
    let cancel_text = i18n.dialog_cancel();
    let items = items.to_vec();
    let store_confirm = store.clone();

    window.open_alert_dialog(cx, move |alert, _window, _cx| {
        // The builder is `Fn` (may run more than once); clone captured
        // values here so `.on_ok` can take ownership.
        let items = items.clone();
        let store_confirm = store_confirm.clone();
        alert
            .title(title)
            .child(
                div()
                    .text_base()
                    .text_color(colors::text_secondary())
                    .child(body.clone()),
            )
            .button_props(
                DialogButtonProps::default()
                    .ok_text(ok_text)
                    .cancel_text(cancel_text)
                    .show_cancel(true),
            )
            .on_ok(move |_, _, cx| {
                store_confirm.update(cx, |s, cx| {
                    s.cleanup_paths(items.clone(), cx);
                });
                true
            })
    });
}
