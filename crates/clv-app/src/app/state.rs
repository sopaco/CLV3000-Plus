use crate::i18n::I18n;
use crate::prelude::*;
use crate::services::{
    cleanup::{poll_cleanup, spawn_cleanup, CleanupPoll},
    scan::{poll_scan, spawn_scan, ScanPoll},
};
use chrono::Utc;
use clv_core::{
    default_selected_item_ids, detect_agent_projects, item_cleanup_bucket, load_last_scan,
    purge_old_trash, resolve_language, restore_trashed, rule_description_matches_query,
    save_last_scan, save_settings, scan_phase_preparing, AppSettings, CleanupBucket, CleanupHistory,
    CleanupHistoryRecord,
    Language, RiskLevel, ScanItem, ScanReport, TrashedEntry,
};
use clv_platform::{pick_folders, primary_disk_usage};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Dashboard,
    Cleanup,
    Agent,
    Startup,
    Process,
    LargeFiles,
    Settings,
    Onboarding,
}

impl AppPage {
    pub fn title(self, i18n: &I18n) -> &'static str {
        i18n.page_title(self)
    }

    /// Stable id for page-transition animations (changes when navigating).
    pub fn transition_key(self) -> &'static str {
        match self {
            Self::Dashboard => "page-dashboard",
            Self::Cleanup => "page-cleanup",
            Self::Agent => "page-agent",
            Self::Startup => "page-startup",
            Self::Process => "page-process",
            Self::LargeFiles => "page-large-files",
            Self::Settings => "page-settings",
            Self::Onboarding => "page-onboarding",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupFilter {
    All,
    SafeOnly,
    ProjectBuildCache,
    SharedToolCache,
    DevEnvironment,
    AiGenerated,
}

pub struct AppStore {
    pub settings: AppSettings,
    pub page: AppPage,
    pub last_report: Option<ScanReport>,
    pub selected_item_ids: HashSet<String>,
    pub scanning: bool,
    pub cleaning: bool,
    pub scan_phase: String,
    pub scan_items_found: usize,
    pub scan_bytes_found: u64,
    pub scan_current_path: Option<String>,
    pub cleanup_completed: usize,
    pub cleanup_total: usize,
    pub cleanup_freed_bytes: u64,
    pub cleanup_current_path: Option<String>,
    pub status_message: Option<String>,
    pub cleanup_filter: CleanupFilter,
    pub search_query: String,
    pub disk_total: u64,
    pub disk_used: u64,
    pub startup_count: usize,
    pub process_refresh_trigger: u64,
    pub cleanup_history: CleanupHistory,
    pub pending_cleanup_notification: Option<String>,
    scan_cancel: Option<Arc<AtomicBool>>,
    cleanup_cancel: Option<Arc<AtomicBool>>,
    scan_restart_pending: bool,
    progress_hud: Option<Entity<super::hud::ProgressHud>>,
}

impl AppStore {
    pub fn i18n(&self) -> I18n {
        I18n::from_settings(&self.settings)
    }

    pub fn language(&self) -> Language {
        resolve_language(self.settings.language)
    }

    pub fn new(settings: AppSettings, cx: &mut Context<Self>) -> Self {
        let last_report = load_last_scan();
        let selected_item_ids = last_report
            .as_ref()
            .map(|r| default_selected_item_ids(&r.items))
            .unwrap_or_default();
        let trash_days = settings.soft_delete_days;
        cx.spawn(async move |_, cx| {
            let _ = cx
                .background_spawn(async move { purge_old_trash(trash_days) })
                .await;
        })
        .detach();

        let mut store = Self {
            settings,
            page: AppPage::Dashboard,
            last_report,
            selected_item_ids,
            scanning: false,
            cleaning: false,
            scan_phase: String::new(),
            scan_items_found: 0,
            scan_bytes_found: 0,
            scan_current_path: None,
            cleanup_completed: 0,
            cleanup_total: 0,
            cleanup_freed_bytes: 0,
            cleanup_current_path: None,
            status_message: None,
            cleanup_filter: CleanupFilter::All,
            search_query: String::new(),
            disk_total: 0,
            disk_used: 0,
            startup_count: 0,
            process_refresh_trigger: 0,
            cleanup_history: CleanupHistory::load(),
            pending_cleanup_notification: None,
            scan_cancel: None,
            cleanup_cancel: None,
            scan_restart_pending: false,
            progress_hud: None,
        };
        store.refresh_disk_usage_async(cx);
        store
    }

    pub fn attach_progress_hud(&mut self, hud: Entity<super::hud::ProgressHud>) {
        self.progress_hud = Some(hud);
    }

    fn notify_progress_only(&self, cx: &mut Context<Self>) {
        if let Some(hud) = &self.progress_hud {
            hud.update(cx, |_, cx| cx.notify());
        }
    }

    pub fn is_item_selected(&self, id: &str) -> bool {
        self.selected_item_ids.contains(id)
    }

    pub fn kill_process_pid(&mut self, pid: u32, cx: &mut Context<Self>) {
        let i18n = self.i18n();
        self.status_message = Some(i18n.killing_process(pid));
        cx.notify();

        cx.spawn(async move |weak, cx| {
            let result = cx
                .background_spawn(async move { clv_platform::kill_process(pid) })
                .await;
            weak.update(cx, |store, cx| {
                let i18n = store.i18n();
                store.process_refresh_trigger = store.process_refresh_trigger.wrapping_add(1);
                store.status_message = Some(match result {
                    Ok(()) => i18n.process_killed(pid),
                    Err(e) => i18n.kill_process_failed(&e.to_string()),
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn refresh_disk_usage_async(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, cx| {
            let usage = cx.background_spawn(async { disk_usage() }).await;
            weak.update(cx, |store, cx| {
                store.disk_total = usage.0;
                store.disk_used = usage.1;
                let tip = store.i18n().tray_tooltip(store.disk_used_percent());
                crate::tray::TrayController::set_global_tooltip(&tip);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn pick_scan_folders(&mut self, cx: &mut Context<Self>) {
        let title = self.i18n().pick_folders_title().to_string();
        cx.spawn(async move |weak, cx| {
            let picked = cx
                .background_spawn(async move { pick_folders(&title) })
                .await;
            if picked.is_empty() {
                return;
            }
            weak.update(cx, |store, cx| {
                for path in picked {
                    if !store.settings.scan_paths.iter().any(|p| p == &path) {
                        store.settings.scan_paths.push(path);
                    }
                }
                let _ = save_settings(&store.settings);
                store.status_message = Some(store.i18n().folders_added().into());
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn set_page(&mut self, page: AppPage, cx: &mut Context<Self>) {
        if self.page == page {
            return;
        }
        self.page = page;
        cx.notify();
    }

    pub fn disk_free(&self) -> u64 {
        self.disk_total.saturating_sub(self.disk_used)
    }

    pub fn disk_used_percent(&self) -> f32 {
        if self.disk_total == 0 {
            0.0
        } else {
            (self.disk_used as f32 / self.disk_total as f32) * 100.0
        }
    }

    /// All / Safe / Project / Shared / DevEnv / Ai counts (ignores search).
    pub fn cleanup_filter_counts(&self) -> [usize; 6] {
        let Some(report) = &self.last_report else {
            return [0; 6];
        };
        let expert = self.settings.expert_mode;
        let mut counts = [0usize; 6];
        for item in &report.items {
            if !expert && item.risk == RiskLevel::Protected {
                continue;
            }
            counts[0] += 1;
            if item.risk == RiskLevel::Safe {
                counts[1] += 1;
            }
            match item_cleanup_bucket(item) {
                CleanupBucket::ProjectBuildCache => counts[2] += 1,
                CleanupBucket::SharedToolCache => counts[3] += 1,
                CleanupBucket::DevEnvironment => counts[4] += 1,
                CleanupBucket::AiGenerated => counts[5] += 1,
            }
        }
        counts
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        let Some(report) = &self.last_report else {
            return Vec::new();
        };
        let expert = self.settings.expert_mode;
        let q = self.search_query.to_lowercase();

        report
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if !expert && item.risk == RiskLevel::Protected {
                    return false;
                }
                let bucket_ok = match self.cleanup_filter {
                    CleanupFilter::All => true,
                    CleanupFilter::SafeOnly => item.risk == RiskLevel::Safe,
                    CleanupFilter::ProjectBuildCache => {
                        item_cleanup_bucket(item) == CleanupBucket::ProjectBuildCache
                    }
                    CleanupFilter::SharedToolCache => {
                        item_cleanup_bucket(item) == CleanupBucket::SharedToolCache
                    }
                    CleanupFilter::DevEnvironment => {
                        item_cleanup_bucket(item) == CleanupBucket::DevEnvironment
                    }
                    CleanupFilter::AiGenerated => {
                        item_cleanup_bucket(item) == CleanupBucket::AiGenerated
                    }
                };
                if !bucket_ok {
                    return false;
                }
                if q.is_empty() {
                    true
                } else {
                    item.name.to_lowercase().contains(&q)
                        || item.path.to_string_lossy().to_lowercase().contains(&q)
                        || rule_description_matches_query(item.description, &q)
                }
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// All checked items in the scan report, regardless of the active sidebar filter.
    pub fn selected_items(&self) -> Vec<clv_core::ScanItem> {
        let Some(report) = &self.last_report else {
            return Vec::new();
        };
        report
            .items
            .iter()
            .filter(|i| self.is_item_selected(&i.id))
            .cloned()
            .collect()
    }

    pub fn selected_count(&self) -> usize {
        let Some(report) = &self.last_report else {
            return 0;
        };
        report
            .items
            .iter()
            .filter(|i| self.is_item_selected(&i.id))
            .count()
    }

    pub fn selected_bytes(&self) -> u64 {
        let Some(report) = &self.last_report else {
            return 0;
        };
        report
            .items
            .iter()
            .filter(|i| self.is_item_selected(&i.id))
            .map(|i| i.size_bytes)
            .sum()
    }

    pub fn set_item_selected(&mut self, id: &str, selected: bool) {
        if selected {
            self.selected_item_ids.insert(id.to_string());
        } else {
            self.selected_item_ids.remove(id);
        }
    }

    pub fn toggle_item(&mut self, id: &str) {
        if self.selected_item_ids.contains(id) {
            self.selected_item_ids.remove(id);
        } else {
            self.selected_item_ids.insert(id.to_string());
        }
    }

    pub fn select_all_filtered(&mut self, selected: bool) {
        let ids: Vec<String> = {
            let Some(report) = &self.last_report else {
                return;
            };
            self.filtered_indices()
                .into_iter()
                .filter_map(|i| report.items.get(i).map(|item| item.id.clone()))
                .collect()
        };
        if selected {
            self.selected_item_ids.extend(ids);
        } else {
            for id in ids {
                self.selected_item_ids.remove(&id);
            }
        }
    }

    pub fn select_project_items(&mut self, project_path: &Path) {
        if let Some(report) = &self.last_report {
            for item in &report.items {
                if item.project_root.as_deref() == Some(project_path) {
                    self.selected_item_ids.insert(item.id.clone());
                }
            }
        }
    }

    pub fn project_has_caution_items(&self, project_path: &Path) -> bool {
        self.last_report.as_ref().is_some_and(|report| {
            report.items.iter().any(|item| {
                item.project_root.as_deref() == Some(project_path)
                    && item.risk == RiskLevel::Caution
            })
        })
    }

    pub fn cancel_scan(&mut self, cx: &mut Context<Self>) {
        if let Some(flag) = &self.scan_cancel {
            flag.store(true, Ordering::Relaxed);
        }
        self.status_message = Some(self.i18n().scan_cancelling().into());
        cx.notify();
    }

    pub fn cancel_cleanup(&mut self, cx: &mut Context<Self>) {
        if let Some(flag) = &self.cleanup_cancel {
            flag.store(true, Ordering::Relaxed);
        }
        self.status_message = Some(self.i18n().cleanup_cancelling().into());
        cx.notify();
    }

    pub fn start_scan(&mut self, cx: &mut Context<Self>) {
        if self.scanning {
            self.scan_restart_pending = true;
            self.cancel_scan(cx);
            return;
        }
        self.scanning = true;
        self.scan_restart_pending = false;
        self.scan_phase = scan_phase_preparing(self.language());
        self.scan_items_found = 0;
        self.scan_bytes_found = 0;
        self.scan_current_path = None;
        self.status_message = Some(self.i18n().scan_start_message());
        let cancel = Arc::new(AtomicBool::new(false));
        self.scan_cancel = Some(cancel.clone());
        cx.notify();
        self.notify_progress_only(cx);

        let settings = self.settings.clone();
        let job = spawn_scan(settings, cancel);

        cx.spawn(async move |weak, cx| {
            let mut finished = false;
            while !finished {
                match poll_scan(&job.rx) {
                    ScanPoll::Done(report) => {
                        let mut restart = false;
                        weak.update(cx, |store, cx| {
                            store.scanning = false;
                            store.scan_cancel = None;
                            store.scan_phase.clear();
                            store.scan_current_path = None;
                            store.selected_item_ids = default_selected_item_ids(&report.items);
                            let cancelled = report.cancelled;
                            store.last_report = Some(report);
                            if let Some(current) = &store.last_report {
                                let _ = save_last_scan(current);
                            }
                            store.status_message = Some(if cancelled {
                                store.i18n().scan_cancelled().into()
                            } else {
                                store.i18n().scan_complete().into()
                            });
                            restart = store.scan_restart_pending;
                            store.scan_restart_pending = false;
                            store.refresh_disk_usage_async(cx);
                            cx.notify();
                            if restart {
                                store.start_scan(cx);
                            }
                        })
                        .ok();
                        finished = true;
                    }
                    ScanPoll::Progress(progress) => {
                        weak.update(cx, |store, cx| {
                            store.scan_phase = progress.phase;
                            store.scan_items_found = progress.items_found;
                            store.scan_bytes_found = progress.bytes_found;
                            store.scan_current_path =
                                progress.current_path.map(|p| truncate_path_display(&p, 96));
                            store.notify_progress_only(cx);
                        })
                        .ok();
                    }
                    ScanPoll::Disconnected => {
                        weak.update(cx, |store, cx| {
                            store.scanning = false;
                            store.scan_cancel = None;
                            store.scan_phase.clear();
                            store.status_message = Some(store.i18n().scan_interrupted().into());
                            cx.notify();
                        })
                        .ok();
                        finished = true;
                    }
                    ScanPoll::Idle => {}
                }

                if finished {
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(200))
                    .await;
            }
        })
        .detach();
    }

    pub fn run_cleanup(&mut self, cx: &mut Context<Self>) -> bool {
        let selected = self.selected_items();
        self.start_cleanup(selected, cx)
    }

    fn start_cleanup(&mut self, selected: Vec<ScanItem>, cx: &mut Context<Self>) -> bool {
        if selected.is_empty() {
            self.status_message = Some(self.i18n().select_items_first().into());
            cx.notify();
            return false;
        }
        if self.cleaning {
            return false;
        }

        self.cleaning = true;
        self.cleanup_completed = 0;
        self.cleanup_total = selected.len();
        self.cleanup_freed_bytes = 0;
        self.cleanup_current_path = None;
        self.status_message = Some(self.i18n().cleanup_in_progress().into());
        let cancel = Arc::new(AtomicBool::new(false));
        self.cleanup_cancel = Some(cancel.clone());
        cx.notify();
        self.notify_progress_only(cx);

        let settings = self.settings.clone();
        let job = spawn_cleanup(settings, selected, cancel);

        cx.spawn(async move |weak, cx| {
            let mut finished = false;
            while !finished {
                match poll_cleanup(&job.rx) {
                    CleanupPoll::Progress(progress) => {
                        weak.update(cx, |store, cx| {
                            store.cleanup_completed = progress.completed;
                            store.cleanup_total = progress.total;
                            store.cleanup_freed_bytes = progress.freed_bytes;
                            store.cleanup_current_path =
                                Some(truncate_path_display(&progress.current_path, 96));
                            store.notify_progress_only(cx);
                        })
                        .ok();
                    }
                    CleanupPoll::Done(result, removed_paths) => {
                        weak.update(cx, |store, cx| {
                            store.cleaning = false;
                            store.cleanup_cancel = None;
                            store.cleanup_completed = 0;
                            store.cleanup_total = 0;
                            store.cleanup_freed_bytes = 0;
                            store.cleanup_current_path = None;
                            store.status_message = Some(store.i18n().cleanup_summary(&result));
                            store.pending_cleanup_notification =
                                Some(store.i18n().cleanup_complete_notification(&result));

                            let record = CleanupHistoryRecord {
                                timestamp: Utc::now(),
                                freed_bytes: result.freed_bytes,
                                success_count: result.success_count,
                                failed_count: result.failed.len(),
                                trashed: result.trashed_entries.clone(),
                            };
                            store.cleanup_history.append(record);
                            let _ = store.cleanup_history.save();

                            if let Some(current) = &mut store.last_report {
                                let removed: HashSet<_> = removed_paths.into_iter().collect();
                                current.items.retain(|i| !removed.contains(&i.path));
                                current.large_files.retain(|f| !removed.contains(&f.path));
                                let roots: Vec<PathBuf> = current
                                    .agent_projects
                                    .iter()
                                    .map(|p| p.path.clone())
                                    .collect();
                                current.agent_projects =
                                    detect_agent_projects(&current.items, &roots);
                                store
                                    .selected_item_ids
                                    .retain(|id| current.items.iter().any(|i| &i.id == id));
                                let _ = save_last_scan(current);
                            }

                            store.refresh_disk_usage_async(cx);
                            cx.notify();
                        })
                        .ok();

                        finished = true;
                    }
                    CleanupPoll::Disconnected => {
                        weak.update(cx, |store, cx| {
                            store.cleaning = false;
                            store.cleanup_cancel = None;
                            store.cleanup_completed = 0;
                            store.cleanup_total = 0;
                            store.cleanup_freed_bytes = 0;
                            store.cleanup_current_path = None;
                            store.status_message = Some(store.i18n().cleanup_interrupted().into());
                            cx.notify();
                        })
                        .ok();
                        finished = true;
                    }
                    CleanupPoll::Idle => {
                        cx.background_executor()
                            .timer(Duration::from_millis(80))
                            .await;
                    }
                }
            }
        })
        .detach();

        true
    }

    pub fn restore_trashed_entry(&mut self, entry: TrashedEntry, cx: &mut Context<Self>) {
        cx.spawn(async move |weak, cx| {
            let restore_result = cx
                .background_spawn({
                    let entry = entry.clone();
                    async move { restore_trashed(&entry) }
                })
                .await;
            weak.update(cx, |store, cx| {
                match restore_result {
                    Ok(()) => {
                        store.cleanup_history.remove_trashed(&entry.trash_path);
                        let _ = store.cleanup_history.save();
                        store.status_message = Some(store.i18n().restore_ok(&entry.name));
                    }
                    Err(e) => {
                        store.status_message =
                            Some(store.i18n().restore_failed(&e.to_string()));
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Delete only these items. Does not merge with Cleanup page selection.
    pub fn cleanup_paths(&mut self, items: Vec<ScanItem>, cx: &mut Context<Self>) -> bool {
        self.start_cleanup(items, cx)
    }

    pub fn finish_onboarding(
        &mut self,
        expert: bool,
        paths: Vec<std::path::PathBuf>,
        cx: &mut Context<Self>,
    ) {
        self.settings.expert_mode = expert;
        if !paths.is_empty() {
            self.settings.scan_paths = paths;
        }
        self.settings.onboarding_done = true;
        let _ = save_settings(&self.settings);
        self.set_page(AppPage::Dashboard, cx);
    }
}

fn truncate_path_display(path: &Path, max_chars: usize) -> String {
    ui::truncate_middle(&path.display().to_string(), max_chars)
}

fn default_disk_usage() -> (u64, u64) {
    (0, 0)
}

fn disk_usage() -> (u64, u64) {
    primary_disk_usage().unwrap_or_else(default_disk_usage)
}
