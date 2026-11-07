use clv_core::{
    AppSettings, CleanupCategory, ScanItem, TrashedEntry, TrashedEntryKind,
    cleanup::{CleanupExecutor, CleanupProgress, CleanupReport},
};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

pub enum CleanupEvent {
    Progress(CleanupProgress),
    Done(CleanupReport, Vec<PathBuf>),
}

pub struct CleanupSpawn {
    pub rx: mpsc::Receiver<CleanupEvent>,
}

pub enum CleanupPoll {
    Progress(CleanupProgress),
    Done(CleanupReport, Vec<PathBuf>),
    Disconnected,
    Idle,
}

pub fn spawn_cleanup(
    settings: AppSettings,
    items: Vec<ScanItem>,
    cancel: Arc<AtomicBool>,
) -> CleanupSpawn {
    let (tx, rx) = mpsc::sync_channel::<CleanupEvent>(64);

    std::thread::spawn(move || {
        // Registry keys cannot go through the recycle-area file pipeline:
        // they are exported with `reg export` first (M2 safety rule). Split
        // them out; everything else keeps the normal executor path.
        let (registry_items, file_items): (Vec<ScanItem>, Vec<ScanItem>) = items
            .into_iter()
            .partition(|item| item.category == CleanupCategory::RegistryLeftover);

        let executor = CleanupExecutor::new(settings);
        let mut result = executor.execute_cancellable(&file_items, &cancel, |progress| {
            let _ = tx.try_send(CleanupEvent::Progress(progress));
        });

        if !registry_items.is_empty() && !cancel.load(Ordering::Relaxed) {
            let registry = clv_platform::clean_registry_keys(&registry_items);
            for backup in registry.succeeded {
                result.success_count += 1;
                result
                    .successful_paths
                    .push(PathBuf::from(&backup.key_path));
                result.trashed.push(backup.backup_path.clone());
                result.trashed_entries.push(TrashedEntry {
                    original: PathBuf::from(&backup.key_path),
                    trash_path: backup.backup_path,
                    size_bytes: 0,
                    name: backup.name,
                    kind: TrashedEntryKind::RegistryKey,
                });
            }
            for (path, error) in registry.failed {
                result.failed.push((PathBuf::from(path), error));
            }
        }

        let cleaned_paths = result.successful_paths.clone();
        let _ = tx.send(CleanupEvent::Done(result, cleaned_paths));
    });

    CleanupSpawn { rx }
}

pub fn poll_cleanup(rx: &mpsc::Receiver<CleanupEvent>) -> CleanupPoll {
    let mut latest_progress = None;
    loop {
        match rx.try_recv() {
            Ok(CleanupEvent::Progress(progress)) => latest_progress = Some(progress),
            Ok(CleanupEvent::Done(report, paths)) => {
                return CleanupPoll::Done(report, paths);
            }
            Err(mpsc::TryRecvError::Empty) => {
                return latest_progress
                    .map(CleanupPoll::Progress)
                    .unwrap_or(CleanupPoll::Idle);
            }
            Err(mpsc::TryRecvError::Disconnected) => return CleanupPoll::Disconnected,
        }
    }
}
