//! Shared GPUI imports for all views.
pub use gpui_kit::{
    div, px, App, Context, Div, Entity, FontWeight, Render, SharedString, StatefulInteractiveElement,
    Window, prelude::*,
};
pub use gpui_kit::component::{
    button::*, checkbox::*, h_flex, switch::Switch, v_flex, *,
};
pub use gpui_kit::component::scroll::ScrollableElement as _;

pub use crate::ui;
/// Build a stable element id from a formatted string.
pub fn eid(s: impl Into<SharedString>) -> SharedString {
    s.into()
}
