use std::{cell::RefCell, rc::Rc};

use magpie_engine::blocker::YOUTUBE_SCRIPT;
use webview2_com::{Microsoft::Web::WebView2::Win32::*, NavigationStartingEventHandler};
use windows::core::Result;

use super::{runtime::Runtime, script, signal::address};

#[derive(Default)]
enum Script {
    #[default]
    Absent,
    Adding,
    Present(String),
}

#[derive(Default)]
struct Injection {
    wanted: bool,
    script: Script,
}

type SharedInjection = Rc<RefCell<Injection>>;

fn added(
    webview: &ICoreWebView2,
    injection: &SharedInjection,
    result: Result<String>,
) -> Result<()> {
    let wanted = injection.borrow().wanted;
    injection.borrow_mut().script = match result {
        Ok(id) => Script::Present(id),
        Err(_) => Script::Absent,
    };
    sync(webview, injection, wanted)
}

fn add(webview: &ICoreWebView2, injection: &SharedInjection) -> Result<()> {
    let owner = webview.clone();
    let adding = Rc::clone(injection);
    script::add(webview, YOUTUBE_SCRIPT, move |result| {
        added(&owner, &adding, result)
    })
}

fn sync(webview: &ICoreWebView2, injection: &SharedInjection, wanted: bool) -> Result<()> {
    let mut state = injection.borrow_mut();
    state.wanted = wanted;
    match (wanted, &state.script) {
        (true, Script::Absent) => {
            state.script = Script::Adding;
            drop(state);
            add(webview, injection)
        }
        (false, Script::Present(id)) => {
            script::remove(webview, id)?;
            state.script = Script::Absent;
            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn wire(webview: &ICoreWebView2, runtime: Rc<Runtime>) -> Result<()> {
    let injection = SharedInjection::default();
    sync(webview, &injection, runtime.shield().enabled())?;
    let handler = NavigationStartingEventHandler::create(Box::new(move |sender, args| {
        let (Some(sender), Some(args)) = (sender, args) else {
            return Ok(());
        };
        match address(|value| unsafe { args.Uri(value) })? {
            Some(page) => sync(&sender, &injection, runtime.shield().active_on(&page)),
            None => Ok(()),
        }
    }));
    unsafe { webview.add_NavigationStarting(&handler, &mut 0) }
}
