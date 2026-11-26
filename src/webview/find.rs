use webview2_com::{
    FindActiveMatchIndexChangedEventHandler, FindMatchCountChangedEventHandler,
    FindStartCompletedHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows::core::{HSTRING, IUnknown, Interface, Result};

use super::{
    native::{Native, report},
    signal::{Signal, Signals},
};
use crate::find::Matches;

fn session(webview: &ICoreWebView2) -> Result<ICoreWebView2Find> {
    unsafe { webview.cast::<ICoreWebView2_28>()?.Find() }
}

fn matches(find: &ICoreWebView2Find) -> Result<Matches> {
    let mut active_position = 0;
    let mut count = 0;
    unsafe { find.ActiveMatchIndex(&mut active_position) }?;
    unsafe { find.MatchCount(&mut count) }?;
    Ok(Matches::new(active_position, count))
}

fn options(webview: &ICoreWebView2, term: &str) -> Result<ICoreWebView2FindOptions> {
    let environment = unsafe { webview.cast::<ICoreWebView2_2>()?.Environment() }?;
    let options = unsafe {
        environment
            .cast::<ICoreWebView2Environment15>()?
            .CreateFindOptions()
    }?;
    unsafe { options.SetFindTerm(&HSTRING::from(term)) }?;
    unsafe { options.SetSuppressDefaultFindDialog(true) }?;
    unsafe { options.SetShouldHighlightAllMatches(true) }?;
    Ok(options)
}

fn start(webview: &ICoreWebView2, term: &str) -> Result<()> {
    let options = options(webview, term)?;
    let completed = FindStartCompletedHandler::create(Box::new(|result| result));
    unsafe { session(webview)?.Start(&options, &completed) }
}

fn send_matches(
    signals: Signals,
) -> impl FnMut(Option<ICoreWebView2Find>, Option<IUnknown>) -> Result<()> {
    move |sender, _| {
        if let Some(sender) = sender {
            signals.send(Signal::Matches(matches(&sender)?));
        }
        Ok(())
    }
}

pub fn wire(webview: &ICoreWebView2, signals: &Signals) -> Result<()> {
    let find = session(webview)?;
    let counted =
        FindMatchCountChangedEventHandler::create(Box::new(send_matches(signals.clone())));
    let moved =
        FindActiveMatchIndexChangedEventHandler::create(Box::new(send_matches(signals.clone())));
    unsafe { find.add_MatchCountChanged(&counted, &mut 0) }?;
    unsafe { find.add_ActiveMatchIndexChanged(&moved, &mut 0) }
}

impl Native {
    pub fn find(&self, term: &str) {
        report(start(&self.webview, term));
    }

    pub fn find_next(&self) {
        report(session(&self.webview).and_then(|find| unsafe { find.FindNext() }));
    }

    pub fn find_previous(&self) {
        report(session(&self.webview).and_then(|find| unsafe { find.FindPrevious() }));
    }

    pub fn stop_find(&self) {
        report(session(&self.webview).and_then(|find| unsafe { find.Stop() }));
    }
}
