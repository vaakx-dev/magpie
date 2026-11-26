use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
use windows::core::{HSTRING, Result};

use super::script::Slot;
use crate::darkmode::Config;

#[derive(Default)]
pub struct DarkMode {
    script: Slot,
    applied: Option<Config>,
}

impl DarkMode {
    pub fn apply(&mut self, webview: &ICoreWebView2, config: &Config) -> Result<()> {
        if self.applied.as_ref() == Some(config) {
            return Ok(());
        }
        self.applied = Some(config.clone());
        unsafe { webview.PostWebMessageAsJson(&HSTRING::from(config.message())) }?;
        self.script.replace(webview, &config.script())
    }
}
