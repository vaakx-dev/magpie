use std::future::Future;

use gpui::{App, Context, Window, WindowHandle};

use super::browser::Browser;

fn regular(browser: &Browser) -> bool {
    !browser.store.role().private()
}

fn browsers(cx: &App) -> impl Iterator<Item = WindowHandle<Browser>> + use<> {
    cx.windows()
        .into_iter()
        .filter_map(|handle| handle.downcast::<Browser>())
}

fn other_regular_window(window: &Window, cx: &App) -> bool {
    let closing = window.window_handle().window_id();
    browsers(cx)
        .filter(|handle| handle.window_id() != closing)
        .any(|handle| handle.read(cx).is_ok_and(regular))
}

async fn forget(forgotten: impl Future<Output = windows::core::Result<()>>) {
    if let Err(error) = forgotten.await {
        eprintln!("magpie: could not clear browsing data on exit: {error}");
    }
}

pub(super) fn quit(cx: &mut App) {
    let clearing = browsers(cx).find(|handle| handle.read(cx).is_ok_and(Browser::wants_clearing));
    let forgotten = clearing.and_then(|handle| {
        handle
            .update(cx, |browser, _, cx| browser.forget_browsing(cx))
            .ok()
    });
    let Some(forgotten) = forgotten else {
        cx.quit();
        return;
    };
    cx.spawn(async move |cx| {
        forget(forgotten).await;
        cx.update(|cx| cx.quit()).ok();
    })
    .detach();
}

impl Browser {
    fn wants_clearing(&self) -> bool {
        self.settings().privacy.clear_on_exit.on() && regular(self)
    }

    fn clears_on_exit(&self, window: &Window, cx: &App) -> bool {
        self.wants_clearing() && !other_regular_window(window, cx)
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
            forget(forgotten).await;
            cx.update(|window, _| window.remove_window()).ok();
        }));
        false
    }
}
