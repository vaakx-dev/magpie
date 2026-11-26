use serde::{Deserialize, Serialize};
use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebMessageReceivedEventHandler};
use windows::core::{HSTRING, Result};

use super::{
    native::report,
    script,
    signal::{Signal, Signals, text},
};

const SCRIPT: &str = include_str!("media.js");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Playback {
    Playing,
    Paused,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub state: Playback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Control {
    Toggle,
    Next,
    Previous,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Message {
    Media(Track),
}

#[derive(Serialize)]
struct Command {
    control: Control,
}

pub fn track(json: &str) -> Option<Track> {
    match serde_json::from_str(json) {
        Ok(Message::Media(track)) => Some(track),
        Err(_) => None,
    }
}

fn receive(args: &ICoreWebView2WebMessageReceivedEventArgs, signals: &Signals) -> Result<()> {
    let json = text(|value| unsafe { args.WebMessageAsJson(value) })?;
    if let Some(track) = track(&json) {
        signals.send(Signal::Track(track));
    }
    Ok(())
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    script::install(webview, SCRIPT)?;
    let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| match args {
        Some(args) => receive(&args, &signals),
        None => Ok(()),
    }));
    unsafe { webview.add_WebMessageReceived(&handler, &mut 0) }
}

pub fn control(webview: &ICoreWebView2, control: Control) {
    let json = serde_json::to_string(&Command { control })
        .expect("a media command always serializes to JSON");
    report(unsafe { webview.PostWebMessageAsJson(&HSTRING::from(json)) });
}
