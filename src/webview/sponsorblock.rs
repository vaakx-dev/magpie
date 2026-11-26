use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebMessageReceivedEventHandler};
use windows::core::Result;

use super::{
    script,
    signal::{Signal, Signals, text},
};
use magpie_engine::sponsorblock::{Bridge, Request};

fn request(args: &ICoreWebView2WebMessageReceivedEventArgs) -> Result<Option<Request>> {
    let json = text(|value| unsafe { args.WebMessageAsJson(value) })?;
    Ok(Request::parse(&json))
}

pub fn wire(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    script::install(webview, Bridge::WebView2.script())?;
    let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        if let Some(request) = request(&args)? {
            signals.send(Signal::SponsorBlock(request));
        }
        Ok(())
    }));
    unsafe { webview.add_WebMessageReceived(&handler, &mut 0) }
}
