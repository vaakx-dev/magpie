use std::rc::Rc;

use url::Url;
use webview2_com::{
    DocumentTitleChangedEventHandler, FaviconChangedEventHandler, GetFaviconCompletedHandler,
    HistoryChangedEventHandler, Microsoft::Web::WebView2::Win32::*,
    NavigationCompletedEventHandler, NavigationStartingEventHandler,
    NewWindowRequestedEventHandler, SourceChangedEventHandler,
};
use windows::{
    Win32::System::Com::IStream,
    core::{BOOL, HSTRING, Interface, Result},
};

use super::{
    Disposition,
    native::Native,
    permission,
    runtime::Runtime,
    signal::{CurrentPage, History, Requested, Signal, Signals, address, text},
};
use crate::{
    address::upgraded,
    config::Privacy,
    internal::access::{self, Initiator},
    storage::permissions::Origin,
};
use magpie_engine::tracking::stripped;

const STREAM_CHUNK_BYTES: usize = 16 * 1024;

fn redirected(args: &ICoreWebView2NavigationStartingEventArgs) -> Result<bool> {
    let mut redirected = BOOL::default();
    unsafe { args.IsRedirected(&mut redirected) }?;
    Ok(redirected.as_bool())
}

fn traversal(args: &ICoreWebView2NavigationStartingEventArgs) -> Result<bool> {
    let args = args.cast::<ICoreWebView2NavigationStartingEventArgs3>()?;
    let mut kind = COREWEBVIEW2_NAVIGATION_KIND::default();
    unsafe { args.NavigationKind(&mut kind) }?;
    Ok(kind == COREWEBVIEW2_NAVIGATION_KIND_BACK_OR_FORWARD)
}

fn upgrade(privacy: Privacy, url: &Url, redirected: bool) -> Option<Url> {
    let wanted = privacy.https_only.on() && !redirected;
    wanted.then(|| upgraded(url)).flatten()
}

fn strip(privacy: Privacy, url: &Url, redirected_from: Option<&Url>) -> Option<Url> {
    let clean = stripped(url).filter(|_| privacy.strip_tracking.on())?;
    let bounced_back = redirected_from == Some(&clean);
    (!bounced_back).then_some(clean)
}

fn rewrite(
    runtime: &Runtime,
    args: &ICoreWebView2NavigationStartingEventArgs,
    url: &Url,
    started: &CurrentPage,
) -> Result<Option<Url>> {
    let privacy = runtime.privacy();
    let redirected = redirected(args)?;
    let watch = runtime.focus().redirect(url);
    let url = watch.as_ref().unwrap_or(url);
    let secure = upgrade(privacy, url, redirected);
    let target = secure.as_ref().unwrap_or(url);
    let started = started.borrow();
    let redirected_from = started.as_ref().filter(|_| redirected);
    Ok(strip(privacy, target, redirected_from).or(secure).or(watch))
}

fn leave_shorts(webview: &ICoreWebView2, runtime: &Runtime, url: &Url) -> Result<()> {
    let Some(watch) = runtime.focus().redirect(url) else {
        return Ok(());
    };
    let target = serde_json::to_string(watch.as_str()).expect("a URL serializes");
    let script = HSTRING::from(format!("location.replace({target});"));
    unsafe { webview.ExecuteScript(&script, None) }
}

fn document(webview: &ICoreWebView2) -> Result<Initiator> {
    let source = address(|value| unsafe { webview.Source(value) })?;
    Ok(Initiator::page(source.as_ref()))
}

fn browser_requested(requested: &Requested, url: &Url) -> bool {
    let requested = requested.borrow_mut().take();
    requested.is_some_and(|requested| access::requested(&requested, url))
}

fn initiator(
    webview: &ICoreWebView2,
    args: &ICoreWebView2NavigationStartingEventArgs,
    started: &CurrentPage,
) -> Result<Initiator> {
    if traversal(args)? {
        return Ok(Initiator::Browser);
    }
    if redirected(args)? {
        return Ok(Initiator::page(started.borrow().as_ref()));
    }
    document(webview)
}

fn cancel_blocked(
    args: &ICoreWebView2NavigationStartingEventArgs,
    initiator: Initiator,
    url: &Url,
) -> Result<bool> {
    let blocked = !access::allowed(initiator, url);
    if blocked {
        unsafe { args.SetCancel(true) }?;
    }
    Ok(blocked)
}

fn on_starting(
    webview: &ICoreWebView2,
    runtime: Rc<Runtime>,
    page: CurrentPage,
    requested: Requested,
    signals: Signals,
) -> Result<()> {
    let handler = NavigationStartingEventHandler::create(Box::new(move |sender, args| {
        let (Some(sender), Some(args)) = (sender, args) else {
            return Ok(());
        };
        let Some(url) = address(|value| unsafe { args.Uri(value) })? else {
            return Ok(());
        };
        let initiator = if browser_requested(&requested, &url) {
            Initiator::Browser
        } else {
            initiator(&sender, &args, &page)?
        };
        if cancel_blocked(&args, initiator, &url)? {
            return Ok(());
        }
        if let Some(rewritten) = rewrite(&runtime, &args, &url, &page)? {
            unsafe { args.SetCancel(true) }?;
            signals.send(Signal::Rewritten(rewritten));
            return Ok(());
        }
        *page.borrow_mut() = Some(url);
        signals.send(Signal::Started);
        Ok(())
    }));
    unsafe { webview.add_NavigationStarting(&handler, &mut 0) }
}

fn on_frame_starting(webview: &ICoreWebView2) -> Result<()> {
    let handler = NavigationStartingEventHandler::create(Box::new(move |sender, args| {
        let (Some(sender), Some(args)) = (sender, args) else {
            return Ok(());
        };
        let Some(url) = address(|value| unsafe { args.Uri(value) })? else {
            return Ok(());
        };
        cancel_blocked(&args, document(&sender)?, &url)?;
        Ok(())
    }));
    unsafe { webview.add_FrameNavigationStarting(&handler, &mut 0) }
}

fn on_source(webview: &ICoreWebView2, runtime: Rc<Runtime>, signals: Signals) -> Result<()> {
    let handler = SourceChangedEventHandler::create(Box::new(move |sender, _| {
        let Some(sender) = sender else {
            return Ok(());
        };
        let Some(url) = address(|value| unsafe { sender.Source(value) })? else {
            return Ok(());
        };
        leave_shorts(&sender, &runtime, &url)?;
        signals.send(Signal::Source(url));
        Ok(())
    }));
    unsafe { webview.add_SourceChanged(&handler, &mut 0) }
}

fn on_title(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let handler = DocumentTitleChangedEventHandler::create(Box::new(move |sender, _| {
        let Some(sender) = sender else {
            return Ok(());
        };
        let title = text(|value| unsafe { sender.DocumentTitle(value) })?;
        signals.send(Signal::Title(title));
        Ok(())
    }));
    unsafe { webview.add_DocumentTitleChanged(&handler, &mut 0) }
}

fn history(webview: &ICoreWebView2) -> Result<History> {
    let mut back = BOOL::default();
    let mut forward = BOOL::default();
    unsafe { webview.CanGoBack(&mut back) }?;
    unsafe { webview.CanGoForward(&mut forward) }?;
    Ok(History {
        back: back.as_bool(),
        forward: forward.as_bool(),
    })
}

fn on_history(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let handler = HistoryChangedEventHandler::create(Box::new(move |sender, _| {
        if let Some(sender) = sender {
            signals.send(Signal::History(history(&sender)?));
        }
        Ok(())
    }));
    unsafe { webview.add_HistoryChanged(&handler, &mut 0) }
}

fn cancelled(args: &ICoreWebView2NavigationCompletedEventArgs) -> Result<bool> {
    let mut status = COREWEBVIEW2_WEB_ERROR_STATUS::default();
    unsafe { args.WebErrorStatus(&mut status) }?;
    Ok(status == COREWEBVIEW2_WEB_ERROR_STATUS_OPERATION_CANCELED)
}

fn on_completed(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let handler = NavigationCompletedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        if !cancelled(&args)? {
            signals.send(Signal::Finished);
        }
        Ok(())
    }));
    unsafe { webview.add_NavigationCompleted(&handler, &mut 0) }
}

fn bytes(stream: &IStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut chunk = vec![0u8; STREAM_CHUNK_BYTES];
    loop {
        let mut read = 0u32;
        let status = unsafe {
            stream.Read(
                chunk.as_mut_ptr().cast(),
                chunk.len() as u32,
                Some(&mut read),
            )
        };
        if status.is_err() || read == 0 {
            return bytes;
        }
        bytes.extend_from_slice(&chunk[..read as usize]);
    }
}

fn request_favicon(webview: ICoreWebView2, signals: Signals) -> Result<()> {
    let completed = GetFaviconCompletedHandler::create(Box::new(move |result, stream| {
        result?;
        let png = stream.as_ref().map(bytes).unwrap_or_default();
        if !png.is_empty() {
            signals.send(Signal::Favicon(png));
        }
        Ok(())
    }));
    let webview = webview.cast::<ICoreWebView2_15>()?;
    unsafe { webview.GetFavicon(COREWEBVIEW2_FAVICON_IMAGE_FORMAT_PNG, &completed) }
}

fn on_favicon(webview: &ICoreWebView2, signals: Signals) -> Result<()> {
    let handler = FaviconChangedEventHandler::create(Box::new(move |sender, _| match sender {
        Some(sender) => request_favicon(sender, signals.clone()),
        None => Ok(()),
    }));
    let webview = webview.cast::<ICoreWebView2_15>()?;
    unsafe { webview.add_FaviconChanged(&handler, &mut 0) }
}

fn user_initiated(args: &ICoreWebView2NewWindowRequestedEventArgs) -> Result<bool> {
    let mut initiated = BOOL::default();
    unsafe { args.IsUserInitiated(&mut initiated) }?;
    Ok(initiated.as_bool())
}

fn new_window(
    runtime: &Runtime,
    sender: &ICoreWebView2,
    args: &ICoreWebView2NewWindowRequestedEventArgs,
    signals: &Signals,
) -> Result<()> {
    unsafe { args.SetHandled(true) }?;
    let source = address(|value| unsafe { sender.Source(value) })?;
    let initiator = Initiator::page(source.as_ref());
    let url = address(|value| unsafe { args.Uri(value) })?;
    let Some(url) = url.filter(|url| access::allowed(initiator, url)) else {
        return Ok(());
    };
    let opener = source.as_ref().and_then(Origin::of);
    match (user_initiated(args)?, opener) {
        (false, Some(opener)) => permission::open_popup(runtime, opener, url, signals),
        _ => signals.send(Signal::Opened(url, Disposition::Foreground)),
    }
    Ok(())
}

fn on_new_window(webview: &ICoreWebView2, runtime: Rc<Runtime>, signals: Signals) -> Result<()> {
    let handler = NewWindowRequestedEventHandler::create(Box::new(move |sender, args| {
        let (Some(sender), Some(args)) = (sender, args) else {
            return Ok(());
        };
        new_window(&runtime, &sender, &args, &signals)
    }));
    unsafe { webview.add_NewWindowRequested(&handler, &mut 0) }
}

pub fn wire(
    native: &Native,
    runtime: Rc<Runtime>,
    page: CurrentPage,
    signals: &Signals,
) -> Result<()> {
    let webview = &native.webview;
    let requested = Rc::clone(&native.requested);
    on_starting(
        webview,
        Rc::clone(&runtime),
        page,
        requested,
        signals.clone(),
    )?;
    on_frame_starting(webview)?;
    on_source(webview, Rc::clone(&runtime), signals.clone())?;
    on_title(webview, signals.clone())?;
    on_history(webview, signals.clone())?;
    on_completed(webview, signals.clone())?;
    on_favicon(webview, signals.clone())?;
    on_new_window(webview, runtime, signals.clone())
}
