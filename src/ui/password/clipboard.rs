use std::time::Duration;

use gpui::{App, ClipboardItem};
use zeroize::Zeroizing;

const CLEAR_AFTER: Duration = Duration::from_secs(30);

fn still_holds(secret: &str, cx: &App) -> bool {
    cx.read_from_clipboard()
        .and_then(|item| item.text())
        .map(Zeroizing::new)
        .is_some_and(|text| text.as_str() == secret)
}

fn clear_if_unchanged(secret: &str, cx: &mut App) {
    if still_holds(secret, cx) {
        cx.write_to_clipboard(ClipboardItem::new_string(String::new()));
    }
}

pub fn copy_secret(secret: Zeroizing<String>, cx: &mut App) {
    cx.write_to_clipboard(ClipboardItem::new_string(secret.as_str().to_owned()));
    cx.spawn(async move |cx| {
        cx.background_executor().timer(CLEAR_AFTER).await;
        cx.update(|cx| clear_if_unchanged(&secret, cx)).ok();
    })
    .detach();
}
