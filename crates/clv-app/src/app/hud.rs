use super::state::AppStore;
use crate::prelude::*;

/// Isolated scan/cleanup progress chrome. Progress ticks notify this view
/// instead of `AppStore`, so the sidebar and page tree are not rebuilt at 5 Hz.
pub struct ProgressHud {
    store: Entity<AppStore>,
}

impl ProgressHud {
    pub fn new(store: Entity<AppStore>) -> Self {
        Self { store }
    }
}

impl Render for ProgressHud {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let store_ref = self.store.read(cx);
        let i18n = store_ref.i18n();
        let scanning = store_ref.scanning;
        let cleaning = store_ref.cleaning;
        let scan_phase = store_ref.scan_phase.clone();
        let scan_items_found = store_ref.scan_items_found;
        let scan_bytes_found = store_ref.scan_bytes_found;
        let scan_current_path = store_ref.scan_current_path.clone();
        let cleanup_completed = store_ref.cleanup_completed;
        let cleanup_total = store_ref.cleanup_total;
        let cleanup_freed_bytes = store_ref.cleanup_freed_bytes;
        let cleanup_current_path = store_ref.cleanup_current_path.clone();
        let store = self.store.clone();

        div()
            .flex_shrink_0()
            .w_full()
            .when(scanning, |this| {
                this.child(ui::scan_progress_bar(
                    &i18n,
                    &scan_phase,
                    scan_items_found,
                    scan_bytes_found,
                    scan_current_path.as_deref(),
                    {
                        let store = store.clone();
                        move |_, _, cx| {
                            store.update(cx, |s, cx| s.cancel_scan(cx));
                        }
                    },
                    cx,
                ))
            })
            .when(cleaning, |this| {
                this.child(ui::cleanup_progress_bar(
                    &i18n,
                    cleanup_completed,
                    cleanup_total,
                    cleanup_freed_bytes,
                    cleanup_current_path.as_deref(),
                    {
                        let store = store.clone();
                        move |_, _, cx| {
                            store.update(cx, |s, cx| s.cancel_cleanup(cx));
                        }
                    },
                    cx,
                ))
            })
    }
}
