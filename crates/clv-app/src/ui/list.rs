//! Virtualized list helpers (`uniform_list` + flex-safe scroll layout).

use crate::prelude::*;
use gpui::{uniform_list, ElementId, ListSizingBehavior, Subscription, UniformListScrollHandle};
use gpui_component::input::{InputEvent, InputState};
use std::ops::Range;

/// Lazily create a search `InputState` and subscribe to change events.
pub fn ensure_search_input<V>(
    slot: &mut Option<Entity<InputState>>,
    subscription_slot: &mut Option<Subscription>,
    placeholder: impl Into<SharedString>,
    window: &mut Window,
    cx: &mut Context<V>,
    on_change: impl Fn(&mut V, String, &mut Context<V>) + 'static,
) -> Entity<InputState>
where
    V: Render + 'static,
{
    if let Some(input) = slot {
        return input.clone();
    }
    let input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder));
    let subscription = cx.subscribe(&input, move |view, input, event, cx| {
        if matches!(event, InputEvent::Change) {
            let value = input.read(cx).value().to_string();
            on_change(view, value, cx);
            cx.notify();
        }
    });
    *subscription_slot = Some(subscription);
    *slot = Some(input.clone());
    input
}

/// Scrollable virtualized list pane for **fixed-height** rows.
///
/// Parent must be a flex column with `flex_1 min_h_0` (see list views).
pub fn uniform_list_pane<V, F, R>(
    list_id: impl Into<ElementId>,
    item_count: usize,
    scroll_handle: UniformListScrollHandle,
    cx: &mut Context<V>,
    render_rows: F,
) -> Div
where
    V: Render,
    F: Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R> + 'static,
    R: IntoElement,
{
    let list = uniform_list(list_id, item_count, cx.processor(render_rows))
        .size_full()
        .track_scroll(scroll_handle.clone())
        .with_sizing_behavior(ListSizingBehavior::Auto);

    div()
        .size_full()
        .relative()
        .overflow_hidden()
        .vertical_scrollbar(&scroll_handle)
        .child(list)
}

/// Body region below a fixed page header — hosts a virtualized list or empty state.
pub fn list_body(content: impl IntoElement) -> Div {
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(content)
}
