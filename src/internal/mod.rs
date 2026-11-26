pub mod access;
pub mod icon;
pub mod settings;

use url::Url;

use crate::pages::{self, PageAction};

pub const SCHEME: &str = "magpie";
pub const BRAND: &str = "Magpie";

const HTML: &str = "text/html; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const JAVASCRIPT: &str = "text/javascript; charset=utf-8";
const PNG: &str = "image/png";
const DOCUMENT_PATH: &str = "/";
const ICON_PATH: &str = "/magpie.png";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Settings,
    History,
    Bookmarks,
    Downloads,
    NewTab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resource {
    pub bytes: &'static [u8],
    pub mime: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Settings(settings::Message),
    Action(Page, PageAction),
}

pub struct Asset {
    path: &'static str,
    resource: Resource,
}

pub const PAGES: [Page; 5] = [
    Page::Settings,
    Page::History,
    Page::Bookmarks,
    Page::Downloads,
    Page::NewTab,
];

const SHARED: [Asset; 1] = [Asset::css("/theme.css", include_bytes!("theme.css"))];

impl Asset {
    pub const fn html(bytes: &'static [u8]) -> Self {
        Self::new(DOCUMENT_PATH, bytes, HTML)
    }

    pub const fn css(path: &'static str, bytes: &'static [u8]) -> Self {
        Self::new(path, bytes, CSS)
    }

    pub const fn javascript(path: &'static str, bytes: &'static [u8]) -> Self {
        Self::new(path, bytes, JAVASCRIPT)
    }

    const fn new(path: &'static str, bytes: &'static [u8], mime: &'static str) -> Self {
        Self {
            path,
            resource: Resource { bytes, mime },
        }
    }
}

impl Page {
    pub fn of(url: &Url) -> Option<Self> {
        let host = url.host_str().filter(|_| url.scheme() == SCHEME)?;
        PAGES
            .into_iter()
            .find(|page| page.host().eq_ignore_ascii_case(host))
    }

    fn host(self) -> &'static str {
        match self {
            Self::Settings => "settings",
            Self::History => "history",
            Self::Bookmarks => "bookmarks",
            Self::Downloads => "downloads",
            Self::NewTab => "newtab",
        }
    }

    pub fn url(self) -> Url {
        Url::parse(&format!("{SCHEME}://{}", self.host())).expect("page addresses are valid")
    }

    fn assets(self) -> &'static [Asset] {
        match self {
            Self::Settings => &settings::ASSETS,
            Self::History => &pages::HISTORY,
            Self::Bookmarks => &pages::BOOKMARKS,
            Self::Downloads => &pages::DOWNLOADS,
            Self::NewTab => &pages::NEW_TAB,
        }
    }

    pub fn message(self, json: &str) -> Option<Message> {
        match self {
            Self::Settings => settings::Message::parse(json).map(Message::Settings),
            Self::History | Self::Bookmarks | Self::Downloads | Self::NewTab => {
                PageAction::parse(json).map(|action| Message::Action(self, action))
            }
        }
    }
}

fn path(url: &Url) -> &str {
    match url.path() {
        "" => DOCUMENT_PATH,
        path => path,
    }
}

pub fn resource(url: &Url) -> Option<Resource> {
    let page = Page::of(url)?;
    match path(url) {
        ICON_PATH => Some(Resource {
            bytes: icon::png(),
            mime: PNG,
        }),
        path => SHARED
            .iter()
            .chain(page.assets())
            .find(|asset| asset.path == path)
            .map(|asset| asset.resource),
    }
}
