use webview2_com::{
    CoreWebView2CustomSchemeRegistration, Microsoft::Web::WebView2::Win32::*,
    WebMessageReceivedEventHandler, WebResourceRequestedEventHandler,
};
use windows::{
    Win32::{System::Com::IStream, UI::Shell::SHCreateMemStream},
    core::{HSTRING, Result, w},
};

use super::signal::{Signal, Signals, address, text};
use crate::internal::{self, Page, Resource};

const OK: i32 = 200;
const NOT_FOUND: i32 = 404;
const POLICY: &str = "default-src 'self'; img-src 'self' data:; frame-ancestors 'none'";

pub fn registration() -> ICoreWebView2CustomSchemeRegistration {
    let registration = CoreWebView2CustomSchemeRegistration::new(internal::SCHEME.to_owned());
    unsafe {
        registration.set_treat_as_secure(true);
        registration.set_has_authority_component(true);
    }
    registration.into()
}

fn headers(resource: &Resource) -> HSTRING {
    let mime = resource.mime;
    HSTRING::from(format!(
        "Content-Type: {mime}\r\nContent-Security-Policy: {POLICY}\r\nX-Frame-Options: DENY\r\nCache-Control: no-cache"
    ))
}

fn response(
    environment: &ICoreWebView2Environment,
    resource: Option<Resource>,
) -> Result<ICoreWebView2WebResourceResponse> {
    match resource {
        Some(resource) => unsafe {
            let body = SHCreateMemStream(Some(resource.bytes));
            environment.CreateWebResourceResponse(body.as_ref(), OK, w!("OK"), &headers(&resource))
        },
        None => unsafe {
            environment.CreateWebResourceResponse(
                None::<&IStream>,
                NOT_FOUND,
                w!("Not Found"),
                w!(""),
            )
        },
    }
}

fn serve(
    environment: &ICoreWebView2Environment,
    args: &ICoreWebView2WebResourceRequestedEventArgs,
) -> Result<()> {
    let request = unsafe { args.Request() }?;
    let Some(url) = address(|value| unsafe { request.Uri(value) })? else {
        return Ok(());
    };
    if url.scheme() != internal::SCHEME {
        return Ok(());
    }
    let response = response(environment, internal::resource(&url))?;
    unsafe { args.SetResponse(&response) }
}

fn message(args: &ICoreWebView2WebMessageReceivedEventArgs) -> Result<Option<internal::Message>> {
    let source = address(|value| unsafe { args.Source(value) })?;
    let Some(page) = source.as_ref().and_then(Page::of) else {
        return Ok(None);
    };
    let json = text(|value| unsafe { args.WebMessageAsJson(value) })?;
    Ok(page.message(&json))
}

pub fn wire(
    webview: &ICoreWebView2,
    environment: ICoreWebView2Environment,
    signals: Signals,
) -> Result<()> {
    let filter = HSTRING::from(format!("{}://*", internal::SCHEME));
    unsafe {
        webview.AddWebResourceRequestedFilter(&filter, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL)
    }?;
    let requested = WebResourceRequestedEventHandler::create(Box::new(move |_, args| match args {
        Some(args) => serve(&environment, &args),
        None => Ok(()),
    }));
    unsafe { webview.add_WebResourceRequested(&requested, &mut 0) }?;
    let received = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
        let Some(args) = args else {
            return Ok(());
        };
        if let Some(message) = message(&args)? {
            signals.send(Signal::Internal(message));
        }
        Ok(())
    }));
    unsafe { webview.add_WebMessageReceived(&received, &mut 0) }
}
