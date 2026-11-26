mod list;
pub mod snapshot;

use std::{
    collections::{BTreeSet, HashSet},
    sync::{
        Arc, RwLock, RwLockWriteGuard,
        atomic::{AtomicBool, Ordering},
    },
};

use adblock::{Engine, FilterSet, lists::ParseOptions, request::Request};
use url::Url;

pub use list::subscriptions;

const BASELINE: &str = include_str!("baseline.txt");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource {
    Document,
    Subdocument,
    Stylesheet,
    Image,
    Media,
    Font,
    Script,
    Fetch,
    WebSocket,
    Ping,
    Other,
}

impl Resource {
    fn kind(self) -> &'static str {
        match self {
            Resource::Document => "document",
            Resource::Subdocument => "sub_frame",
            Resource::Stylesheet => "stylesheet",
            Resource::Image => "image",
            Resource::Media => "media",
            Resource::Font => "font",
            Resource::Script => "script",
            Resource::Fetch => "xmlhttprequest",
            Resource::WebSocket => "websocket",
            Resource::Ping => "ping",
            Resource::Other => "other",
        }
    }
}

pub struct Shield {
    engine: Arc<RwLock<Engine>>,
    paused: RwLock<BTreeSet<String>>,
    enabled: AtomicBool,
}

pub fn site(url: &Url) -> Option<String> {
    url.host_str()
        .map(|host| host.trim_start_matches("www.").to_owned())
}

pub fn stylesheet(selectors: &[String]) -> String {
    selectors
        .iter()
        .map(|selector| format!("{selector}{{display:none !important}}\n"))
        .collect()
}

pub fn web_site(url: &Url) -> Option<String> {
    matches!(url.scheme(), "http" | "https")
        .then(|| site(url))
        .flatten()
}

impl Default for Shield {
    fn default() -> Self {
        Self::new([BASELINE.to_owned()])
    }
}

impl Shield {
    pub fn new(lists: impl IntoIterator<Item = String>) -> Self {
        Self {
            engine: Arc::new(RwLock::new(engine(lists))),
            paused: RwLock::default(),
            enabled: AtomicBool::new(true),
        }
    }

    pub fn private(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            paused: RwLock::new(self.paused_sites()),
            enabled: AtomicBool::new(self.enabled()),
        }
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn pause(&self, site: &str) {
        self.paused_mut().insert(site.to_owned());
    }

    pub fn resume(&self, site: &str) {
        self.paused_mut().remove(site);
    }

    pub fn paused_sites(&self) -> BTreeSet<String> {
        self.paused.read().expect("shield lock poisoned").clone()
    }

    pub fn active_on(&self, page: &Url) -> bool {
        self.enabled() && !self.paused_on(page)
    }

    pub fn paused_on(&self, page: &Url) -> bool {
        site(page).is_some_and(|site| {
            self.paused
                .read()
                .expect("shield lock poisoned")
                .contains(&site)
        })
    }

    fn paused_mut(&self) -> RwLockWriteGuard<'_, BTreeSet<String>> {
        self.paused.write().expect("shield lock poisoned")
    }

    pub fn replace(&self, lists: impl IntoIterator<Item = String>) {
        let fresh = engine(lists);
        *self.engine.write().expect("shield lock poisoned") = fresh;
    }

    pub fn blocks(&self, url: &Url, page: &Url, resource: Resource) -> bool {
        if !self.active_on(page) {
            return false;
        }
        let Ok(request) = Request::new(url.as_str(), page.as_str(), resource.kind(), "get") else {
            return false;
        };
        self.engine
            .read()
            .expect("shield lock poisoned")
            .check_network_request(&request)
            .should_block()
    }

    pub fn page_selectors(&self, page: &Url) -> Vec<String> {
        if !self.active_on(page) {
            return Vec::new();
        }
        let engine = self.engine.read().expect("shield lock poisoned");
        let resources = engine.url_cosmetic_resources(page.as_str());
        resources.hide_selectors.into_iter().collect()
    }

    pub fn class_id_selectors(
        &self,
        page: &Url,
        classes: &HashSet<String>,
        ids: &HashSet<String>,
    ) -> Vec<String> {
        if !self.active_on(page) {
            return Vec::new();
        }
        let engine = self.engine.read().expect("shield lock poisoned");
        let resources = engine.url_cosmetic_resources(page.as_str());
        match resources.generichide {
            true => Vec::new(),
            false => engine.hidden_class_id_selectors(classes, ids, &resources.exceptions),
        }
    }
}

fn engine(lists: impl IntoIterator<Item = String>) -> Engine {
    let mut filters = FilterSet::new(false);
    for list in lists {
        filters.add_filter_list(list, ParseOptions::default());
    }
    Engine::new_with_filter_set(filters)
}
