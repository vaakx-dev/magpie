use webview2_com::{Microsoft::Web::WebView2::Win32::*, ProcessFailedEventHandler};
use windows::core::Result;

use super::signal::{Signal, Signals};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Page,
    Browser,
}

fn failure(kind: COREWEBVIEW2_PROCESS_FAILED_KIND) -> Option<Failure> {
    match kind {
        COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED
        | COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_UNRESPONSIVE => Some(Failure::Page),
        COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED => Some(Failure::Browser),
        _ => None,
    }
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let handler = ProcessFailedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
        unsafe { args.ProcessFailedKind(&mut kind) }?;
        if let Some(failure) = failure(kind) {
            signals.send(Signal::Crashed(failure));
        }
        Ok(())
    }));
    unsafe { webview.add_ProcessFailed(&handler, &mut 0) }
}
