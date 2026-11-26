use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use gpui::{App, Context, Image, ImageFormat};
use url::Url;

use super::browser::Browser;
use crate::storage::{
    session::{Space, Tab},
    thumbnail::Cache,
};

const FRESH_FOR: Duration = Duration::from_secs(10);

struct Shot {
    image: Arc<Image>,
    taken: Option<Instant>,
}

pub struct Thumbnails {
    cache: Cache,
    shots: HashMap<String, Shot>,
}

fn image(jpeg: Vec<u8>) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Jpeg, jpeg))
}

fn report(written: std::io::Result<()>) {
    if let Err(error) = written {
        eprintln!("magpie: could not save a page preview: {error}");
    }
}

impl Thumbnails {
    pub fn new(cache: Cache) -> Self {
        Self {
            cache,
            shots: HashMap::new(),
        }
    }

    pub fn image(&self, page: &str) -> Option<Arc<Image>> {
        self.shots.get(page).map(|shot| Arc::clone(&shot.image))
    }

    pub fn fresh(&self, page: &str) -> bool {
        self.shots
            .get(page)
            .and_then(|shot| shot.taken)
            .is_some_and(|taken| taken.elapsed() < FRESH_FOR)
    }

    pub fn load(&mut self, page: &str) {
        if self.shots.contains_key(page) {
            return;
        }
        if let Some(jpeg) = self.cache.read(page) {
            let shot = Shot {
                image: image(jpeg),
                taken: None,
            };
            self.shots.insert(page.to_owned(), shot);
        }
    }

    pub fn keep(&mut self, page: &str, jpeg: Vec<u8>, cx: &App) {
        let shot = Shot {
            image: image(jpeg),
            taken: Some(Instant::now()),
        };
        let cache = self.cache.clone();
        let (saved, written) = (page.to_owned(), Arc::clone(&shot.image));
        cx.background_executor()
            .spawn(async move { report(cache.write(&saved, written.bytes())) })
            .detach();
        self.shots.insert(page.to_owned(), shot);
    }

    pub fn sweep(&mut self, open: HashSet<String>, cx: &App) {
        self.shots.retain(|page, _| open.contains(page));
        let cache = self.cache.clone();
        cx.background_executor()
            .spawn(async move {
                if let Err(error) = cache.sweep(open.iter().map(String::as_str)) {
                    eprintln!("magpie: could not clear old page previews: {error}");
                }
            })
            .detach();
    }
}

impl Browser {
    pub(super) fn keep_snapshot(&mut self, page: &Url, jpeg: Vec<u8>, cx: &mut Context<Self>) {
        self.thumbnails.keep(page.as_str(), jpeg, cx);
        cx.notify();
    }

    pub(super) fn sweep_thumbnails(&mut self, cx: &mut Context<Self>) {
        let open = self
            .store
            .session
            .spaces
            .iter()
            .flat_map(Space::tabs)
            .flat_map(Tab::panes)
            .map(|pane| pane.url.clone())
            .collect();
        self.thumbnails.sweep(open, cx);
    }
}
