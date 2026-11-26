use serde::Serialize;
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
use windows::core::{HSTRING, Result};

use super::script::Slot;
use crate::config::Switch;

const SCRIPT: &str = include_str!("scrollbar.js");

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Message {
    Scrollbars { slim: bool },
}

#[derive(Default)]
pub struct Scrollbars {
    script: Slot,
    applied: Option<Switch>,
}

fn slim(classic: Switch) -> bool {
    !classic.on()
}

fn restyle_open_page(webview: &ICoreWebView2, classic: Switch) -> Result<()> {
    let message = Message::Scrollbars {
        slim: slim(classic),
    };
    let json = serde_json::to_string(&message).expect("the scrollbar message serializes");
    unsafe { webview.PostWebMessageAsJson(&HSTRING::from(json)) }
}

impl Scrollbars {
    pub fn apply(&mut self, webview: &ICoreWebView2, classic: Switch) -> Result<()> {
        if self.applied.replace(classic) == Some(classic) {
            return Ok(());
        }
        restyle_open_page(webview, classic)?;
        let source = format!("({SCRIPT})({});", slim(classic));
        self.script.replace(webview, &source)
    }
}
