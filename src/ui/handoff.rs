use gpui::{App, WindowHandle};

use super::browser::Browser;
use crate::instance::Instance;

fn shared_window(handle: &WindowHandle<Browser>, cx: &App) -> bool {
    handle
        .read(cx)
        .is_ok_and(|browser| !browser.store.role().private())
}

fn target(cx: &App) -> Option<WindowHandle<Browser>> {
    let browsers: Vec<WindowHandle<Browser>> = cx
        .windows()
        .into_iter()
        .filter_map(|handle| handle.downcast())
        .collect();
    let active = cx
        .active_window()
        .and_then(|handle| handle.downcast::<Browser>())
        .filter(|handle| shared_window(handle, cx));
    active
        .or_else(|| {
            browsers
                .iter()
                .find(|handle| shared_window(handle, cx))
                .copied()
        })
        .or_else(|| browsers.first().copied())
}

fn deliver(addresses: &[String], cx: &mut App) {
    let Some(handle) = target(cx) else {
        return;
    };
    handle
        .update(cx, |browser, window, cx| {
            browser.open_addresses(addresses, cx);
            window.activate_window();
        })
        .ok();
}

pub(super) fn receive(instance: Instance, cx: &mut App) {
    let mut listener = match instance.listen() {
        Ok(listener) => listener,
        Err(error) => {
            return eprintln!("magpie: could not listen for links from new launches: {error}");
        }
    };
    cx.spawn(async move |cx| {
        while let Some(addresses) = listener.next().await {
            cx.update(|cx| deliver(&addresses, cx)).ok();
        }
    })
    .detach();
}
