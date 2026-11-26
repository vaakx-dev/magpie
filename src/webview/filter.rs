use std::rc::Rc;

use url::Url;
use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebResourceRequestedEventHandler};
use windows::{
    Win32::System::Com::IStream,
    core::{BOOL, HSTRING, Interface, PCWSTR, Result, w},
};

use super::{
    runtime::Runtime,
    signal::{CurrentPage, address, text},
};
use crate::{config::Privacy, locale::Locale};
use magpie_engine::{blocker::Resource, tracking::trimmed_referrer};

const PRIVACY_HEADERS: [(PCWSTR, PCWSTR); 2] = [(w!("DNT"), w!("1")), (w!("Sec-GPC"), w!("1"))];
const ACCEPT_LANGUAGE: PCWSTR = w!("Accept-Language");
const REFERER: PCWSTR = w!("Referer");
const PURPOSE_HEADERS: [PCWSTR; 2] = [w!("Sec-Purpose"), w!("Purpose")];
const PREFETCH: &str = "prefetch";

#[derive(Clone, Copy)]
enum Block {
    Resource,
    Speculation,
}

struct Response {
    status: i32,
    reason: PCWSTR,
    headers: PCWSTR,
}

pub struct Filter {
    pub environment: ICoreWebView2Environment,
    pub runtime: Rc<Runtime>,
    pub page: CurrentPage,
}

impl Block {
    fn response(self) -> Response {
        match self {
            Self::Resource => Response {
                status: 204,
                reason: w!("No Content"),
                headers: w!(""),
            },
            Self::Speculation => Response {
                status: 503,
                reason: w!("Service Unavailable"),
                headers: w!("Cache-Control: no-store"),
            },
        }
    }
}

fn resource(context: COREWEBVIEW2_WEB_RESOURCE_CONTEXT, url: &Url, page: &Url) -> Resource {
    match context {
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT if url == page => Resource::Document,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT => Resource::Subdocument,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET => Resource::Stylesheet,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE => Resource::Image,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA => Resource::Media,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT => Resource::Font,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT => Resource::Script,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST
        | COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH => Resource::Fetch,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_WEBSOCKET => Resource::WebSocket,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_PING => Resource::Ping,
        _ => Resource::Other,
    }
}

fn header(headers: &ICoreWebView2HttpRequestHeaders, name: PCWSTR) -> Result<Option<String>> {
    let mut present = BOOL::default();
    unsafe { headers.Contains(name, &mut present) }?;
    if !present.as_bool() {
        return Ok(None);
    }
    text(|value| unsafe { headers.GetHeader(name, value) }).map(Some)
}

fn prefetching(purpose: &str) -> bool {
    purpose
        .split(';')
        .next()
        .is_some_and(|kind| kind.trim().eq_ignore_ascii_case(PREFETCH))
}

fn speculative(headers: &ICoreWebView2HttpRequestHeaders) -> Result<bool> {
    let purposes = PURPOSE_HEADERS
        .iter()
        .map(|name| header(headers, *name))
        .collect::<Result<Vec<_>>>()?;
    Ok(purposes
        .iter()
        .flatten()
        .any(|purpose| prefetching(purpose)))
}

fn send_privacy_signals(headers: &ICoreWebView2HttpRequestHeaders) -> Result<()> {
    PRIVACY_HEADERS
        .iter()
        .try_for_each(|(name, value)| unsafe { headers.SetHeader(*name, *value) })
}

fn send_language(headers: &ICoreWebView2HttpRequestHeaders, locale: Locale) -> Result<()> {
    let languages = HSTRING::from(locale.accept_language());
    unsafe { headers.SetHeader(ACCEPT_LANGUAGE, &languages) }
}

fn trim_referrer(headers: &ICoreWebView2HttpRequestHeaders, destination: &Url) -> Result<()> {
    let referrer = header(headers, REFERER)?.and_then(|value| Url::parse(&value).ok());
    let Some(trimmed) = referrer.and_then(|referrer| trimmed_referrer(&referrer, destination))
    else {
        return Ok(());
    };
    unsafe { headers.SetHeader(REFERER, &HSTRING::from(trimmed)) }
}

fn context(
    args: &ICoreWebView2WebResourceRequestedEventArgs,
) -> Result<COREWEBVIEW2_WEB_RESOURCE_CONTEXT> {
    let mut context = COREWEBVIEW2_WEB_RESOURCE_CONTEXT::default();
    unsafe { args.ResourceContext(&mut context) }?;
    Ok(context)
}

impl Filter {
    fn block(
        &self,
        args: &ICoreWebView2WebResourceRequestedEventArgs,
        headers: &ICoreWebView2HttpRequestHeaders,
        url: &Url,
        privacy: Privacy,
    ) -> Result<Option<Block>> {
        if privacy.block_preloading.on() && speculative(headers)? {
            return Ok(Some(Block::Speculation));
        }
        let context = context(args)?;
        if privacy.block_pings.on() && context == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_PING {
            return Ok(Some(Block::Resource));
        }
        let Some(page) = self.page.borrow().clone() else {
            return Ok(None);
        };
        let blocked = self
            .runtime
            .shield()
            .blocks(url, &page, resource(context, url, &page));
        Ok(blocked.then_some(Block::Resource))
    }

    fn refuse(
        &self,
        args: &ICoreWebView2WebResourceRequestedEventArgs,
        block: Block,
    ) -> Result<()> {
        let Response {
            status,
            reason,
            headers,
        } = block.response();
        let response = unsafe {
            self.environment
                .CreateWebResourceResponse(None::<&IStream>, status, reason, headers)
        }?;
        unsafe { args.SetResponse(&response) }
    }

    fn check(&self, args: &ICoreWebView2WebResourceRequestedEventArgs) -> Result<()> {
        let request = unsafe { args.Request() }?;
        let headers = unsafe { request.Headers() }?;
        let privacy = self.runtime.privacy();
        if privacy.do_not_track.on() {
            send_privacy_signals(&headers)?;
        }
        if let Some(locale) = self.runtime.language().chosen() {
            send_language(&headers, locale)?;
        }
        let Some(url) = address(|value| unsafe { request.Uri(value) })? else {
            return Ok(());
        };
        if privacy.trim_referrer.on() {
            trim_referrer(&headers, &url)?;
        }
        match self.block(args, &headers, &url, privacy)? {
            Some(block) => self.refuse(args, block),
            None => Ok(()),
        }
    }
}

pub fn wire(webview: &ICoreWebView2, filter: Filter) -> Result<()> {
    let every_source = webview.cast::<ICoreWebView2_22>()?;
    unsafe {
        every_source.AddWebResourceRequestedFilterWithRequestSourceKinds(
            w!("*"),
            COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
        )
    }?;
    let handler = WebResourceRequestedEventHandler::create(Box::new(move |_, args| match args {
        Some(args) => filter.check(&args),
        None => Ok(()),
    }));
    unsafe { webview.add_WebResourceRequested(&handler, &mut 0) }
}
