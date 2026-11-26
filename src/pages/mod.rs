mod action;
mod bookmarks;
mod downloads;
mod history;
mod link;
mod local_time;
mod newtab;

use serde_json::{Value, json};

use crate::{config::Scheme, internal::Asset};

pub use action::PageAction;
pub use bookmarks::bookmarks_payload;
pub use downloads::downloads_payload;
pub use history::history_payload;
pub use local_time::LocalTime;
pub use newtab::newtab_payload;

const STYLESHEET: Asset = Asset::css("/shared.css", include_bytes!("shared.css"));
const BRIDGE: Asset = Asset::javascript("/shared.js", include_bytes!("shared.js"));

pub const HISTORY: [Asset; 4] = [
    Asset::html(include_bytes!("history.html")),
    STYLESHEET,
    BRIDGE,
    Asset::javascript("/history.js", include_bytes!("history.js")),
];

pub const BOOKMARKS: [Asset; 5] = [
    Asset::html(include_bytes!("bookmarks.html")),
    STYLESHEET,
    Asset::css("/bookmarks.css", include_bytes!("bookmarks.css")),
    BRIDGE,
    Asset::javascript("/bookmarks.js", include_bytes!("bookmarks.js")),
];

pub const DOWNLOADS: [Asset; 5] = [
    Asset::html(include_bytes!("downloads.html")),
    STYLESHEET,
    Asset::css("/downloads.css", include_bytes!("downloads.css")),
    BRIDGE,
    Asset::javascript("/downloads.js", include_bytes!("downloads.js")),
];

pub const NEW_TAB: [Asset; 5] = [
    Asset::html(include_bytes!("newtab.html")),
    STYLESHEET,
    Asset::css("/newtab.css", include_bytes!("newtab.css")),
    BRIDGE,
    Asset::javascript("/newtab.js", include_bytes!("newtab.js")),
];

pub fn update(scheme: Scheme, state: Value) -> String {
    json!({ "scheme": scheme, "state": state }).to_string()
}
