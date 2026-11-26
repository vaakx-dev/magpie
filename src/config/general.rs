use std::{borrow::Cow, path::PathBuf, time::Duration};

use url::{Url, form_urlencoded};

use super::{Choosable, Language, Switch};

const QUERY: &str = "%s";
const HOUR_SECS: u64 = 60 * 60;
const DEFAULT_HOME_PAGE: &str = "https://duckduckgo.com/";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Engine {
    #[default]
    DuckDuckGo,
    Google,
    Brave,
    Bing,
    Kagi,
    Custom,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Search {
    pub engine: Engine,
    pub custom: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Startup {
    #[default]
    Restore,
    HomePage,
    NewTab,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArchiveAfter {
    Off,
    #[default]
    TwelveHours,
    OneDay,
    OneWeek,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Downloads {
    pub folder: Option<PathBuf>,
    pub ask: Switch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct General {
    pub language: Language,
    pub search: Search,
    pub startup: Startup,
    pub home_page: String,
    pub archive_after: ArchiveAfter,
    pub downloads: Downloads,
}

impl Choosable for Engine {
    const ALL: &'static [Self] = &[
        Self::DuckDuckGo,
        Self::Google,
        Self::Brave,
        Self::Bing,
        Self::Kagi,
        Self::Custom,
    ];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::DuckDuckGo => "DuckDuckGo",
            Self::Google => "Google",
            Self::Brave => "Brave",
            Self::Bing => "Bing",
            Self::Kagi => "Kagi",
            Self::Custom => "Custom",
        })
    }
}

impl Choosable for Startup {
    const ALL: &'static [Self] = &[Self::Restore, Self::HomePage, Self::NewTab];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Restore => "Restore session",
            Self::HomePage => "Home page",
            Self::NewTab => "New tab",
        })
    }
}

impl Choosable for ArchiveAfter {
    const ALL: &'static [Self] = &[Self::Off, Self::TwelveHours, Self::OneDay, Self::OneWeek];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Off => "Off",
            Self::TwelveHours => "12 hours",
            Self::OneDay => "24 hours",
            Self::OneWeek => "7 days",
        })
    }
}

impl Engine {
    fn template(self) -> Option<&'static str> {
        match self {
            Self::DuckDuckGo => Some("https://duckduckgo.com/?q=%s"),
            Self::Google => Some("https://www.google.com/search?q=%s"),
            Self::Brave => Some("https://search.brave.com/search?q=%s"),
            Self::Bing => Some("https://www.bing.com/search?q=%s"),
            Self::Kagi => Some("https://kagi.com/search?q=%s"),
            Self::Custom => None,
        }
    }
}

fn fill(template: &str, query: &str) -> Option<Url> {
    let terms: String = form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let url = Url::parse(&template.replace(QUERY, &terms)).ok()?;
    let web = matches!(url.scheme(), "http" | "https");
    (template.contains(QUERY) && web).then_some(url)
}

impl Search {
    pub fn custom_valid(&self) -> bool {
        fill(&self.custom, "").is_some()
    }

    pub fn url(&self, query: &str) -> Url {
        let template = self.engine.template().unwrap_or(&self.custom);
        fill(template, query)
            .or_else(|| fill(Engine::default().template()?, query))
            .expect("the built-in search templates are valid")
    }
}

impl ArchiveAfter {
    pub fn idle(self) -> Option<Duration> {
        let hours = match self {
            Self::Off => return None,
            Self::TwelveHours => 12,
            Self::OneDay => 24,
            Self::OneWeek => 7 * 24,
        };
        Some(Duration::from_secs(hours * HOUR_SECS))
    }
}

impl Default for General {
    fn default() -> Self {
        Self {
            language: Language::default(),
            search: Search::default(),
            startup: Startup::default(),
            home_page: DEFAULT_HOME_PAGE.to_owned(),
            archive_after: ArchiveAfter::default(),
            downloads: Downloads::default(),
        }
    }
}
