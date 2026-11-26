use magpie_engine::tracking::stripped;
use url::Url;

pub fn without_trackers(address: &str) -> String {
    Url::parse(address)
        .ok()
        .and_then(|url| stripped(&url))
        .map_or_else(|| address.to_owned(), String::from)
}
