use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
use windows::core::{HSTRING, Result};

use super::script::Slot;
use crate::config::Switch;

const SCRIPT: &str = include_str!("background.js");

#[derive(Default)]
pub struct BackgroundPlay {
    script: Slot,
    applied: Option<Switch>,
}

fn source(play: Switch) -> String {
    format!("({SCRIPT})({});", play.on())
}

impl BackgroundPlay {
    pub fn apply(&mut self, webview: &ICoreWebView2, play: Switch) -> Result<()> {
        if self.applied.replace(play) == Some(play) {
            return Ok(());
        }
        let source = source(play);
        unsafe { webview.ExecuteScript(&HSTRING::from(&source), None) }?;
        match play {
            Switch::On => self.script.replace(webview, &source),
            Switch::Off => self.script.clear(webview),
        }
    }
}
