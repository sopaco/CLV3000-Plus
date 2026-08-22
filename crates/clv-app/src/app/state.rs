use crate::i18n::I18n;
use crate::prelude::*;
use clv_core::{
    resolve_language, AppSettings, CleanupBucket, RiskLevel, ScanProgress, ScanReport, Scanner,
    cleanup::CleanupExecutor, detect_agent_projects, item_cleanup_bucket, save_settings, Language,
};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use sysinfo::Disks;

enum ScanEvent {
    Progress(ScanProgress),
    Done(ScanReport),
}

enum CleanupEvent {
    Done(clv_core::cleanup::CleanupReport, Vec<std::path::PathBuf>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Dashboard,
    Cleanup,
    Agent,
    Startup,
    Process,
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
    pub scanning: bool,
    pub cleaning: bool,
    pub scan_phase: String,
    pub scan_items_found: usize,
    pub scan_bytes_found: u64,
    pub scan_current_path: Option<String>,
    pub status_message: Option<String>,
    pub cleanup_filter: CleanupFilter,
    pub search_query: String,
    pub expanded_item: Option<String>,
    pub last_cleanup_freed: Option<u64>,
    pub disk_total: u64,
    pub disk_used: u64,
    pub startup_count: usize,
    pub process_refresh_trigger: u64,
}

impl AppStore {
    pub fn i18n(&self) -> I18n {
        I18n::from_settings(&self.settings)
    }

    pub fn language(&self) -> Language {
        resolve_language(self.settings.language)
    }

    pub fn new(settings: AppSettings, _cx: &mut Context<Self>) -> Self {
        Self {
            settings,
            page: AppPage::Dashboard,
            last_report: None,
            scanning: false,
            cleaning: false,
            scan_phase: String::new(),
            scan_items_found: 0,
            scan_bytes_found: 0,
            scan_current_path: None,
            status_message: None,
            cleanup_filter: CleanupFilter::All,
            search_query: String::new(),
            expanded_item: None,
            last_cleanup_freed: None,
            disk_total: 0,
            disk_used: 0,
            startup_count: 0,
            process_refresh_trigger: 0,
        }
    }

    pub fn kill_process_pid(&mut self, pid: u32, cx: &mut Context<Self>) {
        let i18n = self.i18n();
        self.status_message = Some(i18n.killing_process(pid));
        cx.notify();

        cx.spawn(async move |weak, cx| {
            let result = std::thread::spawn(move || clv_platform::kill_process(pid)).join();
            let result = match result {
                Ok(r) => r,
                Err(_) => Err(anyhow::anyhow!(i18n.kill_process_internal_error())),
            };
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
            let usage = std::thread::spawn(disk_usage)
                .join()
                .unwrap_or(default_disk_usage());
            weak.update(cx, |store, cx| {
                store.disk_total = usage.0;
                store.disk_used = usage.1;
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

    pub fn filtered_items(&self) -> Vec<clv_core::ScanItem> {
        let Some(report) = &self.last_report else {
            return Vec::new();
        };
        let expert = self.settings.expert_mode;
        let q = self.search_query.to_lowercase();

        report
            .items
            .iter()
            .filter(|item| {
                if !expert && item.risk == RiskLevel::Protected {
                    return false;
                }
                match self.cleanup_filter {
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
                }
            })
            .filter(|item| {
                if q.is_empty() {
                    true
                } else {
                    item.name.to_lowercase().contains(&q)
                        || item.path.to_string_lossy().to_lowercase().contains(&q)
                        || item.description.to_lowercase().contains(&q)
                }
            })
            .cloned()
            .collect()
    }

    pub fn selected_items(&self) -> Vec<clv_core::ScanItem> {
        self.filtered_items()
            .into_iter()
            .filter(|i| i.selected)
            .collect()
    }

    pub fn selected_bytes(&self) -> u64 {
        self.selected_items().iter().map(|i| i.size_bytes).sum()
    }

    pub fn toggle_item(&mut self, id: &str) {
        if let Some(report) = &mut self.last_report {
            if let Some(item) = report.items.iter_mut().find(|i| i.id == id) {
                item.selected = !item.selected;
            }
        }
    }

    pub fn select_all_filtered(&mut self, selected: bool) {
        let ids: Vec<String> = self.filtered_items().iter().map(|i| i.id.clone()).collect();
        if let Some(report) = &mut self.last_report {
            for item in &mut report.items {
                if ids.contains(&item.id) {
                    item.selected = selected;
                }
            }
        }
    }

    pub fn start_scan(&mut self, cx: &mut Context<Self>) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.scan_phase = self.i18n().scan_preparing().into();
        self.scan_items_found = 0;
        self.scan_bytes_found = 0;
        self.scan_current_path = None;
        self.status_message = Some(self.i18n().scan_start_message());
        cx.notify();

        let settings = self.settings.clone();
        let (tx, rx) = mpsc::sync_channel::<ScanEvent>(64);

        std::thread::spawn(move || {
            let scanner = Scanner::new(settings);
            let report = scanner.scan(|progress| {
                let _ = tx.try_send(ScanEvent::Progress(progress));
            });
            let _ = tx.send(ScanEvent::Done(report));
        });

        cx.spawn(async move |weak, cx| {
            let mut finished = false;
            while !finished {
                let mut latest_progress = None;
                let mut done_report = None;
                loop {
                    match rx.try_recv() {
                        Ok(ScanEvent::Progress(progress)) => latest_progress = Some(progress),
                        Ok(ScanEvent::Done(report)) => {
                            done_report = Some(report);
                            break;
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            weak.update(cx, |store, cx| {
                                store.scanning = false;
                                store.scan_phase.clear();
                                store.status_message = Some(store.i18n().scan_interrupted().into());
                                cx.notify();
                            })
                            .ok();
                            finished = true;
                            break;
                        }
                    }
                }

                if let Some(report) = done_report {
                    weak.update(cx, |store, cx| {
                        store.scanning = false;
                        store.scan_phase.clear();
                        store.scan_current_path = None;
                        store.last_report = Some(report);
                        store.status_message = Some(store.i18n().scan_complete().into());
                        store.refresh_disk_usage_sync();
                        store.startup_count = clv_platform::list_startup_items().len();
                        cx.notify();
                    })
                    .ok();
                    finished = true;
                } else if let Some(progress) = latest_progress {
                    weak.update(cx, |store, cx| {
                        store.scan_phase = progress.phase;
                        store.scan_items_found = progress.items_found;
                        store.scan_bytes_found = progress.bytes_found;
                        store.scan_current_path =
                            progress.current_path.map(|p| truncate_path_display(&p, 96));
                        cx.notify();
                    })
                    .ok();
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
        let Some(report) = self.last_report.clone() else {
            return false;
        };
        let selected: Vec<_> = report.items.into_iter().filter(|i| i.selected).collect();
        if selected.is_empty() {
            self.status_message = Some(self.i18n().select_items_first().into());
            cx.notify();
            return false;
        }
        if self.cleaning {
            return false;
        }

        self.cleaning = true;
        self.status_message = Some(self.i18n().cleanup_in_progress().into());
        cx.notify();

        let settings = self.settings.clone();
        let selected_paths: Vec<_> = selected.iter().map(|i| i.path.clone()).collect();
        let (tx, rx) = mpsc::channel::<CleanupEvent>();

        std::thread::spawn(move || {
            let executor = CleanupExecutor::new(settings);
            let result = executor.execute(&selected);
            let _ = tx.send(CleanupEvent::Done(result, selected_paths));
        });

        cx.spawn(async move |weak, cx| {
            let mut finished = false;
            while !finished {
                match rx.try_recv() {
                    Ok(CleanupEvent::Done(result, removed_paths)) => {
                        weak.update(cx, |store, cx| {
                            store.cleaning = false;
                            store.last_cleanup_freed = Some(result.freed_bytes);
                            store.status_message = Some(store.i18n().cleanup_summary(&result));

                            if let Some(current) = &mut store.last_report {
                                let removed: std::collections::HashSet<_> =
                                    removed_paths.into_iter().collect();
                                current.items.retain(|i| !removed.contains(&i.path));
                                current.agent_projects = detect_agent_projects(
                                    &current.items,
                                    &store.settings.scan_paths,
                                );
                            }

                            store.refresh_disk_usage_sync();
                            cx.notify();
                        })
                        .ok();
                        finished = true;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        cx.background_executor()
                            .timer(Duration::from_millis(80))
                            .await;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        weak.update(cx, |store, cx| {
                            store.cleaning = false;
                            store.status_message = Some(store.i18n().cleanup_interrupted().into());
                            cx.notify();
                        })
                        .ok();
                        finished = true;
                    }
                }
            }
        })
        .detach();

        true
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

impl AppStore {
    fn refresh_disk_usage_sync(&mut self) {
        let (total, used) = disk_usage();
        self.disk_total = total;
        self.disk_used = used;
    }
}

fn default_disk_usage() -> (u64, u64) {
    (512_u64 * 1024 * 1024 * 1024, 400_u64 * 1024 * 1024 * 1024)
}

fn disk_usage() -> (u64, u64) {
    let disks = Disks::new_with_refreshed_list();
    let mut total = 0u64;
    let mut available = 0u64;
    for disk in disks.list() {
        total += disk.total_space();
        available += disk.available_space();
    }
    if total > 0 {
        (total, total.saturating_sub(available))
    } else {
        default_disk_usage()
    }
}
