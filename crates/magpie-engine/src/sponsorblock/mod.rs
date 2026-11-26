pub mod api;
pub mod bridge;
pub mod cache;
pub mod category;
pub mod client;
pub mod config;
pub mod segment;
pub mod video;
pub mod view;

pub use bridge::{Reply, Request, answer};
pub use cache::Cache;
pub use category::{Categories, Category};
pub use client::{Client, Error};
pub use config::{Channel, Config, switch};
pub use segment::{Action, Segment, Skip, skips};
pub use video::VideoId;
pub use view::View;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bridge {
    WebView2,
    GeckoView,
}

const WEBVIEW2_SCRIPT: &str = concat!(
    "(",
    include_str!("youtube.js"),
    ")(",
    include_str!("webview2.js"),
    ");"
);
const GECKOVIEW_SCRIPT: &str = concat!(
    "(",
    include_str!("youtube.js"),
    ")(",
    include_str!("geckoview.js"),
    ");"
);

impl Bridge {
    pub fn script(self) -> &'static str {
        match self {
            Self::WebView2 => WEBVIEW2_SCRIPT,
            Self::GeckoView => GECKOVIEW_SCRIPT,
        }
    }
}
