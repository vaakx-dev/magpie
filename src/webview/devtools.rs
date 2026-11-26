use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
use windows::core::Result;

use super::native::{Native, report};

pub fn wire(webview: &ICoreWebView2) -> Result<()> {
    unsafe { webview.Settings()?.SetAreDevToolsEnabled(true) }
}

impl Native {
    pub fn open_dev_tools(&self) {
        report(unsafe { self.webview.OpenDevToolsWindow() });
    }
}
