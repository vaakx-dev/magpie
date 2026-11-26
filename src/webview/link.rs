use serde::Deserialize;
use url::Url;
use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebMessageReceivedEventHandler};
use windows::core::Result;

use super::{
    Disposition, script,
    signal::{Signal, Signals, text},
};

const SCRIPT: &str = include_str!("link.js");
const WEB_SCHEMES: [&str; 2] = ["http", "https"];

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Message {
    Background { url: String },
}

fn background(args: &ICoreWebView2WebMessageReceivedEventArgs) -> Result<Option<Url>> {
    let json = text(|value| unsafe { args.WebMessageAsJson(value) })?;
    let Ok(Message::Background { url }) = serde_json::from_str(&json) else {
        return Ok(None);
    };
    let url = Url::parse(&url).ok();
    Ok(url.filter(web))
}

pub fn web(url: &Url) -> bool {
    WEB_SCHEMES.contains(&url.scheme())
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    script::install(webview, SCRIPT)?;
    let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        if let Some(url) = background(&args)? {
            signals.send(Signal::Opened(url, Disposition::Background));
        }
        Ok(())
    }));
    unsafe { webview.add_WebMessageReceived(&handler, &mut 0) }
}
