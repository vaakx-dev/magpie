use serde::Serialize;
use url::Url;

use crate::storage::session::Link;

#[derive(Serialize)]
pub(super) struct Shortcut<'a> {
    url: &'a str,
    title: &'a str,
    host: String,
}

pub(super) fn host(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_owned))
        .unwrap_or_else(|| url.to_owned())
}

pub(super) fn shortcuts(links: &[Link]) -> Vec<Shortcut<'_>> {
    links
        .iter()
        .map(|link| Shortcut {
            url: &link.url,
            title: &link.title,
            host: host(&link.url),
        })
        .collect()
}
