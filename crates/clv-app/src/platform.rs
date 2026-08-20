//! Platform-specific application icon (dock / taskbar).

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct AppIcons;

#[cfg(target_os = "macos")]
pub fn apply_app_icon() {
    let Some(bytes) = AppIcons::get("icons/icon_app.icns").or_else(|| AppIcons::get("icons/icon_app.png"))
    else {
        return;
    };

    unsafe {
        use cocoa::appkit::NSApplication;
        use cocoa::base::{id, nil};
        use cocoa::foundation::NSData;
        use objc::{class, msg_send, sel, sel_impl};

        let data = NSData::dataWithBytes_length_(
            nil,
            bytes.data.as_ref().as_ptr() as *const std::ffi::c_void,
            bytes.data.len() as u64,
        );
        if data == nil {
            return;
        }

        let image: id = msg_send![class!(NSImage), alloc];
        let image: id = msg_send![image, initWithData: data];
        if image == nil {
            return;
        }

        let app = NSApplication::sharedApplication(nil);
        let _: () = msg_send![app, setApplicationIconImage: image];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply_app_icon() {}
