mod chromium;
mod detect;
mod firefox;
mod pin;
mod snapshot;
mod visit;

use std::error::Error;
use std::path::PathBuf;

use crate::storage::Store;
use crate::storage::clock::Epoch;
use crate::storage::history::Visit;

pub use detect::{Roots, detect};

type Failure = Box<dyn Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Browser {
    Chrome,
    Edge,
    Brave,
    Firefox,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub browser: Browser,
    pub name: String,
    pub folder: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Bookmarks,
    History,
    Everything,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Folder { name: String, children: Vec<Node> },
    Bookmark { title: String, url: String },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Imported {
    pub bookmarks: Vec<Node>,
    pub history: Vec<Visit>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub bookmarks: usize,
    pub folders: usize,
    pub visits: usize,
    pub warnings: Vec<String>,
}

const WEB_SCHEMES: [&str; 2] = ["https://", "http://"];

fn importable(url: &str) -> bool {
    WEB_SCHEMES.iter().any(|scheme| url.starts_with(scheme))
}

impl Browser {
    pub fn name(self) -> &'static str {
        match self {
            Self::Chrome => "Chrome",
            Self::Edge => "Edge",
            Self::Brave => "Brave",
            Self::Firefox => "Firefox",
        }
    }
}

impl Scope {
    fn bookmarks(self) -> bool {
        matches!(self, Self::Bookmarks | Self::Everything)
    }

    fn history(self) -> bool {
        matches!(self, Self::History | Self::Everything)
    }
}

impl Imported {
    fn collect(
        scope: Scope,
        bookmarks: impl FnOnce() -> Result<Vec<Node>, Failure>,
        history: impl FnOnce() -> Result<Vec<Visit>, Failure>,
    ) -> Self {
        let mut imported = Self::default();
        if scope.bookmarks() {
            match bookmarks() {
                Ok(nodes) => imported.bookmarks = nodes,
                Err(error) => imported.warn("Bookmarks", &*error),
            }
        }
        if scope.history() {
            match history() {
                Ok(visits) => imported.history = visits,
                Err(error) => imported.warn("History", &*error),
            }
        }
        imported
    }

    fn failed(source: &str, error: &dyn Error) -> Self {
        let mut imported = Self::default();
        imported.warn(source, error);
        imported
    }

    fn warn(&mut self, source: &str, error: &dyn Error) {
        self.warnings.push(format!("{source}: {error}"));
    }
}

impl Profile {
    pub fn label(&self) -> String {
        format!("{} ({})", self.browser.name(), self.name)
    }

    pub fn read(&self, scope: Scope) -> Imported {
        match self.browser {
            Browser::Chrome | Browser::Edge | Browser::Brave => chromium::read(&self.folder, scope),
            Browser::Firefox => firefox::read(&self.folder, scope),
        }
    }

    pub fn import(&self, store: &mut Store, scope: Scope, now: Epoch) -> Report {
        self.read(scope).apply(&self.label(), store, now)
    }
}
