use std::{collections::HashSet, sync::Arc};

use serde::Deserialize;
use url::Url;
use webview2_com::{Microsoft::Web::WebView2::Win32::*, WebMessageReceivedEventHandler};
use windows::core::{HSTRING, Result};

use super::{
    script,
    signal::{address, text},
};
use magpie_engine::blocker::{Shield, stylesheet};

const SCRIPT: &str = include_str!("cosmetic.js");

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Message {
    Page,
    Names {
        classes: HashSet<String>,
        ids: HashSet<String>,
    },
}

fn selectors(shield: &Shield, page: &Url, message: Message) -> Vec<String> {
    match message {
        Message::Page => shield.page_selectors(page),
        Message::Names { classes, ids } => shield.class_id_selectors(page, &classes, &ids),
    }
}

fn answer(
    shield: &Shield,
    webview: &ICoreWebView2,
    args: &ICoreWebView2WebMessageReceivedEventArgs,
) -> Result<()> {
    let Some(page) = address(|value| unsafe { args.Source(value) })? else {
        return Ok(());
    };
    let json = text(|value| unsafe { args.WebMessageAsJson(value) })?;
    let Ok(message) = serde_json::from_str::<Message>(&json) else {
        return Ok(());
    };
    let hidden = selectors(shield, &page, message);
    if hidden.is_empty() {
        return Ok(());
    }
    unsafe { webview.PostWebMessageAsString(&HSTRING::from(stylesheet(&hidden))) }
}

pub fn wire(webview: &ICoreWebView2, shield: Arc<Shield>) -> Result<()> {
    script::install(webview, SCRIPT)?;
    let handler = WebMessageReceivedEventHandler::create(Box::new(move |sender, args| {
        match (sender, args) {
            (Some(sender), Some(args)) => answer(&shield, &sender, &args),
            _ => Ok(()),
        }
    }));
    unsafe { webview.add_WebMessageReceived(&handler, &mut 0) }
}
