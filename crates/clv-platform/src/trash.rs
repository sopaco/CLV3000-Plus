use anyhow::{Context, Result};

/// Returns an approximate byte size of the system recycle bin / Trash, if available.
pub fn system_trash_bytes() -> Option<u64> {
    #[cfg(target_os = "windows")]
    {
        return windows_trash_bytes();
    }
    #[cfg(target_os = "macos")]
    {
        return macos_trash_bytes();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

/// Empties the OS recycle bin / Finder Trash. Returns freed bytes when known.
pub fn empty_system_trash() -> Result<u64> {
    let before = system_trash_bytes().unwrap_or(0);
    #[cfg(target_os = "windows")]
    {
        windows_empty_trash()?;
    }
    #[cfg(target_os = "macos")]
    {
        macos_empty_trash()?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        anyhow::bail!("system trash is not supported on this platform");
    }
    let after = system_trash_bytes().unwrap_or(0);
    Ok(before.saturating_sub(after))
}

#[cfg(target_os = "windows")]
fn windows_trash_bytes() -> Option<u64> {
    let mut total = 0u64;
    for letter in b'A'..=b'Z' {
        let drive = format!("{}:\\$Recycle.Bin", letter as char);
        let path = std::path::Path::new(&drive);
        if path.exists() {
            total = total.saturating_add(dir_size(path));
        }
    }
    Some(total)
}

#[cfg(target_os = "windows")]
fn windows_empty_trash() -> Result<()> {
    use windows_sys::Win32::UI::Shell::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOSOUND};

    let hwnd = std::ptr::null_mut();
    let flags = SHERB_NOCONFIRMATION | SHERB_NOSOUND;
    let hr = unsafe { SHEmptyRecycleBinW(hwnd, std::ptr::null(), flags) };
    if hr != 0 {
        anyhow::bail!("SHEmptyRecycleBin failed: {hr}");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_trash_bytes() -> Option<u64> {
    let home = directories::UserDirs::new()?.home_dir().to_path_buf();
    let trash = home.join(".Trash");
    if trash.exists() {
        Some(dir_size(&trash))
    } else {
        Some(0)
    }
}

#[cfg(target_os = "macos")]
fn macos_empty_trash() -> Result<()> {
    std::process::Command::new("osascript")
        .arg("-e")
        .arg("tell application \"Finder\" to empty trash")
        .status()
        .context("failed to run osascript to empty Trash")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("Finder refused to empty Trash"))
}

fn dir_size(path: &std::path::Path) -> u64 {
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0u64;
    let Ok(read_dir) = std::fs::read_dir(path) else {
        return 0;
    };
    for entry in read_dir.filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_dir() {
            total = total.saturating_add(dir_size(&p));
        } else {
            total = total.saturating_add(entry.metadata().map(|m| m.len()).unwrap_or(0));
        }
    }
    total
}
