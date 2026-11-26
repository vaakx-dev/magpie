use std::{cell::RefCell, rc::Rc};

use webview2_com::{
    AddScriptToExecuteOnDocumentCreatedCompletedHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows::core::{HSTRING, Result};

#[derive(Default)]
pub struct Slot(Rc<RefCell<Option<String>>>);

pub fn add(
    webview: &ICoreWebView2,
    source: &str,
    added: impl FnOnce(Result<String>) -> Result<()> + 'static,
) -> Result<()> {
    let completed =
        AddScriptToExecuteOnDocumentCreatedCompletedHandler::create(Box::new(move |result, id| {
            added(result.map(|()| id))
        }));
    unsafe { webview.AddScriptToExecuteOnDocumentCreated(&HSTRING::from(source), &completed) }
}

pub fn install(webview: &ICoreWebView2, source: &str) -> Result<()> {
    add(webview, source, |result| result.map(drop))
}

pub fn remove(webview: &ICoreWebView2, id: &str) -> Result<()> {
    unsafe { webview.RemoveScriptToExecuteOnDocumentCreated(&HSTRING::from(id)) }
}

impl Slot {
    pub fn clear(&self, webview: &ICoreWebView2) -> Result<()> {
        match self.0.borrow_mut().take() {
            Some(id) => remove(webview, &id),
            None => Ok(()),
        }
    }

    pub fn replace(&self, webview: &ICoreWebView2, source: &str) -> Result<()> {
        self.clear(webview)?;
        let slot = Rc::clone(&self.0);
        add(webview, source, move |result| {
            *slot.borrow_mut() = Some(result?);
            Ok(())
        })
    }
}
