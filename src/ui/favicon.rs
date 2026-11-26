use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use gpui::{AnyElement, App, Context, Entity, Hsla, Image, ImageFormat, img, prelude::*, px};
use url::Url;

use super::icon::{ICON_PX, Icon};
use crate::{
    internal::{Page, icon},
    storage::favicon::Cache,
};
use magpie_engine::blocker::web_site;

const CORNER_PX: f32 = 3.0;

pub struct Favicons {
    cache: Cache,
    sites: HashMap<String, Option<Arc<Image>>>,
}

static MAGPIE: LazyLock<Arc<Image>> = LazyLock::new(|| image(icon::png().to_vec()));

fn image(png: Vec<u8>) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Png, png))
}

pub fn magpie() -> Arc<Image> {
    Arc::clone(&MAGPIE)
}

impl Favicons {
    pub fn new(cache: Cache) -> Self {
        Self {
            cache,
            sites: HashMap::new(),
        }
    }

    pub fn icon(&mut self, page: &str) -> Option<Arc<Image>> {
        let page = Url::parse(page).ok()?;
        match Page::of(&page) {
            Some(_) => Some(magpie()),
            None => self.site_icon(&page),
        }
    }

    fn site_icon(&mut self, page: &Url) -> Option<Arc<Image>> {
        let site = web_site(page)?;
        let cache = &self.cache;
        self.sites
            .entry(site)
            .or_insert_with_key(|site| cache.read(site).map(image))
            .clone()
    }

    fn save(&self, site: &str, png: &[u8]) {
        if let Err(error) = self.cache.write(site, png) {
            eprintln!("magpie: could not cache the favicon of {site}: {error}");
        }
    }

    pub fn store(&mut self, page: &Url, png: Vec<u8>, cx: &mut Context<Self>) {
        let Some(site) = web_site(page) else {
            return;
        };
        let fresh = image(png);
        let known = self
            .sites
            .get(&site)
            .and_then(Option::as_ref)
            .is_some_and(|known| known.id() == fresh.id());
        if known {
            return;
        }
        self.save(&site, fresh.bytes());
        self.sites.insert(site, Some(fresh));
        cx.notify();
    }
}

fn favicon(image: Option<Arc<Image>>, fallback: Hsla) -> AnyElement {
    match image {
        Some(image) => img(image)
            .size(px(ICON_PX))
            .flex_none()
            .rounded(px(CORNER_PX))
            .into_any_element(),
        None => Icon::Globe.tinted(fallback).into_any_element(),
    }
}

pub fn site_icon(
    favicons: &Entity<Favicons>,
    page: &str,
    fallback: impl Into<Hsla>,
    cx: &mut App,
) -> AnyElement {
    let image = favicons.update(cx, |favicons, _| favicons.icon(page));
    favicon(image, fallback.into())
}
