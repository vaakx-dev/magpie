use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebMessageReceivedEventHandler};
use windows::core::{HSTRING, Result};

use super::{
    native::report,
    signal::{Signal, Signals, text},
};
use crate::{locale::Locale, reader::Config};

fn style(args: &ICoreWebView2WebMessageReceivedEventArgs) -> Result<Option<Config>> {
    let json = text(|value| unsafe { args.WebMessageAsJson(value) })?;
    Ok(Config::parse(&json))
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        if let Some(config) = style(&args)? {
            signals.send(Signal::Reader(config));
        }
        Ok(())
    }));
    unsafe { webview.add_WebMessageReceived(&handler, &mut 0) }
}

pub fn toggle(webview: &ICoreWebView2, config: &Config, locale: Locale) {
    let script = HSTRING::from(config.script(locale));
    report(unsafe { webview.ExecuteScript(&script, None) });
}
