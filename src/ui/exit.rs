use gpui::{App, Context, Window};

use super::browser::Browser;

fn regular(browser: &Browser) -> bool {
    !browser.store.role().private()
}

fn other_regular_window(window: &Window, cx: &App) -> bool {
    let closing = window.window_handle().window_id();
    cx.windows()
        .into_iter()
        .filter(|handle| handle.window_id() != closing)
        .filter_map(|handle| handle.downcast::<Browser>())
        .any(|handle| handle.read(cx).is_ok_and(regular))
}

impl Browser {
    fn clears_on_exit(&self, window: &Window, cx: &App) -> bool {
        self.settings().privacy.clear_on_exit.on()
            && regular(self)
            && !other_regular_window(window, cx)
    }

    pub(super) fn may_close(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.leaving.is_some() {
            return false;
        }
        if !self.clears_on_exit(window, cx) {
            return true;
        }
        let forgotten = self.forget_browsing(cx);
        self.leaving = Some(cx.spawn_in(window, async move |_, cx| {
            if let Err(error) = forgotten.await {
                eprintln!("magpie: could not clear browsing data on exit: {error}");
            }
            cx.update(|window, _| window.remove_window()).ok();
        }));
        false
    }
}
