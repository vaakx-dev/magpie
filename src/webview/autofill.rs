use serde::Serialize;
use url::Url;
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
use windows::core::HSTRING;
use zeroize::Zeroizing;

use super::{link, native::report};

const SCRIPT: &str = include_str!("autofill.js");

#[derive(Serialize)]
pub struct Fill<'a> {
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
}

#[derive(Serialize)]
struct Request<'a> {
    origin: String,
    #[serde(flatten)]
    fill: &'a Fill<'a>,
}

pub fn fillable(page: &Url) -> bool {
    link::web(page)
}

pub fn fill(webview: &ICoreWebView2, page: &Url, fill: &Fill) {
    if !fillable(page) {
        return;
    }
    let request = Request {
        origin: page.origin().ascii_serialization(),
        fill,
    };
    let json = Zeroizing::new(
        serde_json::to_string(&request).expect("an autofill request always serializes to JSON"),
    );
    let script = Zeroizing::new(format!("{SCRIPT}({})", json.as_str()));
    report(unsafe { webview.ExecuteScript(&HSTRING::from(script.as_str()), None) });
}
