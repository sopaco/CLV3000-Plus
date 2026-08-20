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
        use objc::runtime::Object;
        use objc::{class, msg_send, sel, sel_impl};
        use std::ptr;

        let nil = ptr::null_mut::<Object>();
        let data: *mut Object = msg_send![
            class!(NSData),
            dataWithBytes: bytes.data.as_ptr() as *const std::ffi::c_void
            length: bytes.data.len() as u64
        ];
        if data.is_null() {
            return;
        }

        let image: *mut Object = msg_send![class!(NSImage), alloc];
        let image: *mut Object = msg_send![image, initWithData: data];
        if image.is_null() {
            return;
        }

        // Use raw msg_send — cocoa::NSApplication::sharedApplication expects a
        // `platform` ivar that only exists on GPUI's GPUIApplication subclass.
        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, setApplicationIconImage: image];
        let _ = nil;
    }
}

#[cfg(not(target_os = "macos"))]
pub fn apply_app_icon() {}
