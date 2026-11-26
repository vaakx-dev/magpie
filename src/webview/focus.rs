use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
use windows::core::{HSTRING, Result};

use super::script::Slot;
use magpie_engine::focus::Focus as Rules;

#[derive(Default)]
pub struct Focus {
    script: Slot,
    applied: Option<Rules>,
}

impl Focus {
    pub fn apply(&mut self, webview: &ICoreWebView2, rules: &Rules) -> Result<()> {
        if self.applied.as_ref() == Some(rules) {
            return Ok(());
        }
        self.applied = Some(rules.clone());
        unsafe { webview.ExecuteScript(&HSTRING::from(rules.update()), None) }?;
        self.script.replace(webview, &rules.script())
    }
}
