use serde::Deserialize;
use url::Url;
use uuid::Uuid;
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
    Background { url: String, token: String },
}

struct Token(String);

impl Token {
    fn new() -> Self {
        Self(Uuid::new_v4().simple().to_string())
    }

    fn script(&self) -> String {
        format!("{SCRIPT}(\"{}\");", self.0)
    }
}

fn message(args: &ICoreWebView2WebMessageReceivedEventArgs) -> Option<Message> {
    let json = text(|value| unsafe { args.TryGetWebMessageAsString(value) }).ok()?;
    serde_json::from_str(&json).ok()
}

fn background(message: Message, expected: &Token) -> Option<Url> {
    let Message::Background { url, token } = message;
    if token != expected.0 {
        return None;
    }
    Url::parse(&url).ok().filter(web)
}

pub fn web(url: &Url) -> bool {
    WEB_SCHEMES.contains(&url.scheme())
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let token = Token::new();
    script::install(webview, &token.script())?;
    let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
        let opened = args
            .and_then(|args| message(&args))
            .and_then(|message| background(message, &token));
        if let Some(url) = opened {
            signals.send(Signal::Opened(url, Disposition::Background));
        }
        Ok(())
    }));
    unsafe { webview.add_WebMessageReceived(&handler, &mut 0) }
}
