use crate::app::state::AppStore;
use crate::i18n::I18n;
use crate::prelude::*;
use crate::theme::{colors, corner};
use clv_core::{
    format_bytes, CleanupCategory, LargeFileEntry, RiskLevel, RuleDescription, ScanItem, TechStack,
    LARGE_FILE_THRESHOLD_BYTES,
};

pub struct LargeFilesView {
    store: Entity<AppStore>,
}

impl LargeFilesView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self { store }
    }
}

impl Render for LargeFilesView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let i18n = store.i18n();
        let files: Vec<LargeFileEntry> = store
            .last_report
            .as_ref()
            .map(|r| r.large_files.clone())
            .unwrap_or_default();
        let threshold = format_bytes(LARGE_FILE_THRESHOLD_BYTES);
        let store_entity = self.store.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .child(ui::scroll_y(
                div()
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(ui::page_header(
                        i18n.large_files_title(),
                        &i18n.large_files_subtitle(&threshold),
                    ))
                    .child(if files.is_empty() {
                        ui::empty_state(
                            IconName::Inbox,
                            i18n.large_files_empty_title(),
                            i18n.large_files_empty_hint(),
                        )
                        .into_any_element()
                    } else {
                        ui::glass_card()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .children(files.iter().map(|file| {
                                file_row(file, &i18n, store_entity.clone(), cx)
                            }))
                            .into_any_element()
                    }),
            ))
    }
}

fn file_row(file: &LargeFileEntry, i18n: &I18n, store: Entity<AppStore>, _cx: &App) -> Div {
    let path = file.path.clone();
    let entry = file.clone();
    h_flex()
        .w_full()
        .p_3()
        .rounded(corner())
        .border_1()
        .border_color(colors::glass_border())
        .bg(colors::glass_bg_soft())
        .justify_between()
        .items_center()
        .gap_4()
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_1()
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::text_primary())
                        .child(file.name.clone()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::text_muted())
                        .overflow_hidden()
                        .text_ellipsis()
                        .child(path.display().to_string()),
                ),
        )
        .child(
            div()
                .text_base()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::warn_orange())
                .child(file.size_human()),
        )
        .child(ui::open_path_button(
            SharedString::from(format!("open-{}", file.path.display())),
            &path,
            i18n,
        ))
        .child(
            ui::std_button(
                Button::new(SharedString::from(format!("del-{}", file.path.display())))
                    .danger()
                    .label(i18n.delete_file()),
            )
            .on_click({
                let store = store.clone();
                let i18n_title = i18n.confirm_delete_file_title();
                let body = i18n.confirm_delete_file_body(&entry.name, &entry.size_human());
                move |_, window, cx| {
                    let store_ok = store.clone();
                    let item = large_file_as_item(&entry);
                    let body = body.clone();
                    window.open_dialog(cx, move |dialog, _, _| {
                        dialog
                            .title(i18n_title)
                            .child(body.clone())
                            .confirm()
                            .on_ok({
                                let store = store_ok.clone();
                                let item = item.clone();
                                move |_, _, cx| {
                                    store.update(cx, |s, cx| {
                                        s.cleanup_paths(vec![item.clone()], cx);
                                    });
                                    true
                                }
                            })
                    });
                }
            }),
        )
}

fn large_file_as_item(file: &LargeFileEntry) -> ScanItem {
    ScanItem {
        id: format!("large-file-{}", file.path.display()),
        path: file.path.clone(),
        name: file.name.clone(),
        size_bytes: file.size_bytes,
        stack: TechStack::Other,
        risk: RiskLevel::Caution,
        category: CleanupCategory::TempFiles,
        description: RuleDescription::R016,
        project_root: file.path.parent().map(std::path::Path::to_path_buf),
        last_modified: file.last_modified,
    }
}
