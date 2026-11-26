use std::rc::Rc;

use webview2_com::{
    AcceleratorKeyPressedEventHandler, FocusChangedEventHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows::core::Result;

use super::{
    key,
    runtime::Runtime,
    signal::{Signal, Signals},
};

fn shortcut(
    runtime: &Runtime,
    args: &ICoreWebView2AcceleratorKeyPressedEventArgs,
) -> Result<Option<gpui::Keystroke>> {
    let mut kind = COREWEBVIEW2_KEY_EVENT_KIND::default();
    unsafe { args.KeyEventKind(&mut kind) }?;
    let pressed = matches!(
        kind,
        COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN | COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN
    );
    let mut virtual_key = 0;
    unsafe { args.VirtualKey(&mut virtual_key) }?;
    Ok(key::keystroke(virtual_key).filter(|keystroke| pressed && runtime.handles(keystroke)))
}

fn on_shortcut(
    controller: &ICoreWebView2Controller,
    runtime: Rc<Runtime>,
    signals: Signals,
) -> Result<()> {
    let handler = AcceleratorKeyPressedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        let Some(keystroke) = shortcut(&runtime, &args)? else {
            return Ok(());
        };
        unsafe { args.SetHandled(true) }?;
        signals.send(Signal::Shortcut(keystroke));
        Ok(())
    }));
    unsafe { controller.add_AcceleratorKeyPressed(&handler, &mut 0) }
}

fn on_focus(controller: &ICoreWebView2Controller, signals: Signals) -> Result<()> {
    let handler = FocusChangedEventHandler::create(Box::new(move |_, _| {
        signals.send(Signal::Focused);
        Ok(())
    }));
    unsafe { controller.add_GotFocus(&handler, &mut 0) }
}

pub fn wire(
    controller: &ICoreWebView2Controller,
    runtime: Rc<Runtime>,
    signals: &Signals,
) -> Result<()> {
    on_shortcut(controller, runtime, signals.clone())?;
    on_focus(controller, signals.clone())
}
