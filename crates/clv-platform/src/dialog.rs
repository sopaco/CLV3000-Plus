use std::path::PathBuf;

/// Opens a native multi-select folder picker. Blocks the calling thread until the user
/// dismisses the dialog — call from a background thread, not the GPUI main thread.
pub fn pick_folders(title: &str) -> Vec<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title(title);
    dialog.pick_folders().unwrap_or_default()
}
