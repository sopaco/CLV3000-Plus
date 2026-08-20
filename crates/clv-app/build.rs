use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    sync_icons();
    #[cfg(target_os = "windows")]
    embed_windows_icon();
}

fn sync_icons() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_icons = manifest_dir.join("../../assets/icons");
    let local_icons = manifest_dir.join("assets/icons");

    for name in ["icon_app.png", "icon_app.ico", "icon_app.icns"] {
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

#[cfg(target_os = "windows")]
fn embed_windows_icon() {
    use std::io::Write;

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ico = manifest_dir.join("assets/icons/icon_app.ico");
    if !ico.exists() {
        return;
    }

    println!("cargo:rerun-if-changed={}", ico.display());

    let rc_path = manifest_dir.join("app-icon.rc");
    let mut rc = fs::File::create(&rc_path).expect("create app-icon.rc");
    writeln!(rc, r#"1 ICON "{}""#, ico.display()).expect("write app-icon.rc");

    let _ = embed_resource::compile(rc_path, embed_resource::NONE);
}
