use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
use windows::core::Result;

use super::script::Slot;
use crate::config::Switch;

const SCRIPT: &str = include_str!("preload.js");

#[derive(Default)]
pub struct Preloading {
    script: Slot,
    blocked: Option<Switch>,
}

impl Preloading {
    pub fn apply(&mut self, webview: &ICoreWebView2, blocked: Switch) -> Result<()> {
        if self.blocked.replace(blocked) == Some(blocked) {
            return Ok(());
        }
        match blocked {
            Switch::On => self.script.replace(webview, SCRIPT),
            Switch::Off => self.script.clear(webview),
        }
    }
}
