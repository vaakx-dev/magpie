use webview2_com::{
    ContainsFullScreenElementChangedEventHandler, Microsoft::Web::WebView2::Win32::ICoreWebView2,
};
use windows::core::{BOOL, HSTRING, Result};

use super::{
    native::report,
    signal::{Signal, Signals},
};

const EXIT_SCRIPT: &str = "document.fullscreenElement && document.exitFullscreen()";

fn contains_fullscreen_element(webview: &ICoreWebView2) -> Result<bool> {
    let mut contains = BOOL::default();
    unsafe { webview.ContainsFullScreenElement(&mut contains) }?;
    Ok(contains.as_bool())
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let handler =
        ContainsFullScreenElementChangedEventHandler::create(Box::new(move |sender, _| {
            let Some(sender) = sender else {
                return Ok(());
            };
            signals.send(Signal::Fullscreen(contains_fullscreen_element(&sender)?));
            Ok(())
        }));
    unsafe { webview.add_ContainsFullScreenElementChanged(&handler, &mut 0) }
}

pub fn exit(webview: &ICoreWebView2) {
    report(unsafe { webview.ExecuteScript(&HSTRING::from(EXIT_SCRIPT), None) });
}
