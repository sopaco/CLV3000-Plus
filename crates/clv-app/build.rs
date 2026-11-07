use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    sync_icons();
    // Build scripts are compiled for the *host*; `cfg(target_os)` would test the
    // host too, silently skipping Windows resources when cross-compiling.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_windows_resources();
    }
}

fn sync_icons() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_icons = manifest_dir.join("../../assets/icons");
    let local_icons = manifest_dir.join("assets/icons");

    for name in ["icon_app.png", "icon_app.ico", "icon_app.icns", "tray.png"] {
        let src = workspace_icons.join(name);
        if !src.exists() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", src.display());
        fs::create_dir_all(&local_icons).ok();
        let dst = local_icons.join(name);
        if fs::read(&src).ok() != fs::read(&dst).ok() {
            fs::copy(&src, &dst).ok();
        }
    }
}

fn embed_windows_resources() {
    use std::io::Write;

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ico = manifest_dir.join("assets/icons/icon_app.ico");
    if !ico.exists() {
        println!(
            "cargo:warning=Windows icon missing at {}; resources skipped",
            ico.display()
        );
        return;
    }

    println!("cargo:rerun-if-changed={}", ico.display());

    // Paths are relative to the .rc file — embed-resource resolves from the RC
    // location. Forward slashes work on Windows and avoid escaping issues.
    let rc_path = manifest_dir.join("app-icon.rc");
    let mut rc = fs::File::create(&rc_path).expect("create app-icon.rc");
    writeln!(rc, r#"1 ICON "assets/icons/icon_app.ico""#).expect("write icon resource");

    // Deliberately no RT_MANIFEST here: gpui-pre's default `windows-manifest`
    // feature already embeds an id-1 manifest (PerMonitorV2 DPI, Common-Controls
    // v6, SegmentHeap) into gpui.lib, and that feature cannot be disabled
    // through gpui-kit's dependency graph. Embedding a second id-1 manifest
    // makes MSVC fail with CVT1100 (duplicate resource MANIFEST) / LNK1123.

    // `manifest_optional` tolerates a missing resource compiler (cross builds),
    // which keeps `cargo check --target …-windows-msvc` working on macOS/Linux.
    match embed_resource::compile(&rc_path, embed_resource::NONE).manifest_optional() {
        Ok(()) => {}
        Err(e) => println!("cargo:warning=Windows resources not embedded: {e}"),
    }
}
