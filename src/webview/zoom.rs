use serde::Deserialize;
use webview2_com::{
    AddScriptToExecuteOnDocumentCreatedCompletedHandler, Microsoft::Web::WebView2::Win32::*,
    WebMessageReceivedEventHandler, ZoomFactorChangedEventHandler,
};
use windows::core::{HSTRING, Result};

use super::{
    native::{Native, report},
    signal::{Signal, Signals, text},
};
use crate::zoom::Step;

const SCRIPT: &str = include_str!("zoom.js");

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Message {
    Zoom { step: Step },
}

pub(super) fn factor(controller: &ICoreWebView2Controller) -> Result<f64> {
    let mut factor = 0.0;
    unsafe { controller.ZoomFactor(&mut factor) }?;
    Ok(factor)
}

fn step(args: &ICoreWebView2WebMessageReceivedEventArgs) -> Result<Option<Step>> {
    let json = text(|value| unsafe { args.WebMessageAsJson(value) })?;
    let message = serde_json::from_str::<Message>(&json).ok();
    Ok(message.map(|Message::Zoom { step }| step))
}

fn disable_native_zoom(webview: &ICoreWebView2) -> Result<()> {
    unsafe { webview.Settings()?.SetIsZoomControlEnabled(false) }
}

fn on_wheel(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let installed =
        AddScriptToExecuteOnDocumentCreatedCompletedHandler::create(Box::new(|result, _| result));
    unsafe { webview.AddScriptToExecuteOnDocumentCreated(&HSTRING::from(SCRIPT), &installed) }?;
    let handler = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
        if let Some(step) = args.as_ref().map(step).transpose()?.flatten() {
            signals.send(Signal::Zoom(step));
        }
        Ok(())
    }));
    unsafe { webview.add_WebMessageReceived(&handler, &mut 0) }
}

fn on_drift(controller: &ICoreWebView2Controller, signals: Signals) -> Result<()> {
    let handler = ZoomFactorChangedEventHandler::create(Box::new(move |_, _| {
        signals.send(Signal::ZoomDrifted);
        Ok(())
    }));
    unsafe { controller.add_ZoomFactorChanged(&handler, &mut 0) }
}

pub fn wire(native: &Native, signals: &Signals) -> Result<()> {
    disable_native_zoom(&native.webview)?;
    on_wheel(&native.webview, signals.clone())?;
    on_drift(&native.controller, signals.clone())
}

impl Native {
    pub fn zoom_to(&self, wanted: f64) {
        match factor(&self.controller) {
            Ok(current) if current == wanted => {}
            Ok(_) => report(unsafe { self.controller.SetZoomFactor(wanted) }),
            Err(error) => report(Err(error)),
        }
    }
}
