//! Shared control sizing — Windows-like click targets.

use crate::prelude::*;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::Input;

/// Standard corner radius — slightly rounded (matches `theme::corner_control`).
pub const BTN_RADIUS: f32 = 6.;
/// Standard button height (≈ Windows default control).
pub const BTN_H: f32 = 38.;
/// Primary / CTA button height.
pub const BTN_H_LG: f32 = 42.;
/// Standard single-line text field height — same box as [`BTN_H`].
pub const INPUT_H: f32 = BTN_H;

/// Force a single-line input to the standard control height ([`INPUT_H`]).
///
/// gpui-kit sizes a single-line `Input` from its `Size` (`sizing.rs::input_h`):
/// the default `Size::Medium` resolves to `h_8()` ≈ 32px, which is 6px shorter
/// than [`std_button`]'s 38px. Placed in an `items_center` row next to buttons
/// the field reads as misaligned / squat.
///
/// `Input::render` applies `refine_style(&self.style)` **after** its internal
/// `input_h(self.size)`, so this `Styled` override wins.
pub fn std_input(input: Input) -> Input {
    input.h(px(INPUT_H)).min_h(px(INPUT_H))
}

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
