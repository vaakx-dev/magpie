mod message;
mod reply;

use url::Url;

pub use message::{Host, Import, Message, Part, PasswordAction, Secret};
pub use reply::{BackupStatus, Challenge, Hello, LoginRow, Method, Passwords, Reply, Snapshot};

use super::{Asset, Page};

pub const ASSETS: [Asset; 3] = [
    Asset::html(include_bytes!("index.html")),
    Asset::css("/settings.css", include_bytes!("settings.css")),
    Asset::javascript("/settings.js", include_bytes!("settings.js")),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    General,
    Appearance,
    Layout,
    Search,
    Privacy,
    Passwords,
    Downloads,
    Languages,
    AddOns,
    Import,
    About,
}

const SECTIONS: [Section; 11] = [
    Section::General,
    Section::Appearance,
    Section::Layout,
    Section::Search,
    Section::Privacy,
    Section::Passwords,
    Section::Downloads,
    Section::Languages,
    Section::AddOns,
    Section::Import,
    Section::About,
];

impl Section {
    pub fn parse(anchor: &str) -> Option<Self> {
        SECTIONS
            .into_iter()
            .find(|section| section.anchor() == anchor)
    }

    fn anchor(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Appearance => "appearance",
            Self::Layout => "layout",
            Self::Search => "search",
            Self::Privacy => "privacy",
            Self::Passwords => "passwords",
            Self::Downloads => "downloads",
            Self::Languages => "languages",
            Self::AddOns => "addons",
            Self::Import => "import",
            Self::About => "about",
        }
    }

    pub fn url(self) -> Url {
        let mut url = Page::Settings.url();
        url.set_fragment(Some(self.anchor()));
        url
    }
}
