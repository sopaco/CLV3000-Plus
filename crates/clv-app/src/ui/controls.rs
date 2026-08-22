//! Shared control sizing — Windows-like click targets.

use crate::prelude::*;
use gpui_component::button::Button;

/// Standard corner radius — slightly rounded (matches `theme::corner_control`).
pub const BTN_RADIUS: f32 = 6.;
/// Standard button height (≈ Windows default control).
pub const BTN_H: f32 = 38.;
/// Primary / CTA button height.
pub const BTN_H_LG: f32 = 42.;

/// Apply standard button dimensions + pointer cursor.
pub fn std_button(btn: Button) -> Button {
    btn.h(px(BTN_H))
        .min_h(px(BTN_H))
        .min_w(px(96.))
        .px(px(18.))
        .py(px(8.))
        .rounded(px(BTN_RADIUS))
        .cursor_pointer()
}

/// Apply large primary button dimensions + pointer cursor.
pub fn lg_button(btn: Button) -> Button {
    btn.h(px(BTN_H_LG))
        .min_h(px(BTN_H_LG))
        .min_w(px(128.))
        .px(px(24.))
        .py(px(10.))
        .rounded(px(BTN_RADIUS))
        .cursor_pointer()
}
