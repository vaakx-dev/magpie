use webview2_com::{Microsoft::Web::WebView2::Win32::*, ProcessFailedEventHandler};
use windows::core::Result;

use super::signal::{Signal, Signals};

fn breaks_the_tab(kind: COREWEBVIEW2_PROCESS_FAILED_KIND) -> bool {
    matches!(
        kind,
        COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED
            | COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_UNRESPONSIVE
    )
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let handler = ProcessFailedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
        unsafe { args.ProcessFailedKind(&mut kind) }?;
        if breaks_the_tab(kind) {
            signals.send(Signal::Crashed);
        }
        Ok(())
    }));
    unsafe { webview.add_ProcessFailed(&handler, &mut 0) }
}
