use std::collections::BTreeMap;

use serde::Serialize;
use url::Url;

use crate::config::Switch;
use magpie_engine::blocker::web_site;

const SCRIPT: &str = include_str!("darkmode.js");

const SITE_SEPARATOR: char = ',';

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Off,
    Auto,
    Forced,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub enabled: Switch,
    sites: BTreeMap<String, Mode>,
}

#[derive(Serialize)]
struct Rules<'a> {
    standard: Mode,
    sites: &'a BTreeMap<String, Mode>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Message<'a> {
    DarkMode { rules: Rules<'a> },
}

fn json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("the dark mode rules serialize")
}

impl Config {
    fn standard(&self) -> Mode {
        match self.enabled {
            Switch::On => Mode::Auto,
            Switch::Off => Mode::Off,
        }
    }

    fn turned_on(&self) -> Mode {
        match self.enabled {
            Switch::On => Mode::Auto,
            Switch::Off => Mode::Forced,
        }
    }

    fn site_mode(&self, site: &str) -> Mode {
        self.sites.get(site).copied().unwrap_or(self.standard())
    }

    pub fn mode(&self, page: &Url) -> Mode {
        web_site(page).map_or(Mode::Off, |site| self.site_mode(&site))
    }

    pub fn toggle(&mut self, page: &Url) {
        let Some(site) = web_site(page) else {
            return;
        };
        let next = match self.site_mode(&site) {
            Mode::Off => self.turned_on(),
            Mode::Auto | Mode::Forced => Mode::Off,
        };
        match next == self.standard() {
            true => self.sites.remove(&site),
            false => self.sites.insert(site, next),
        };
    }

    pub fn reset(&mut self, site: &str) {
        self.sites.remove(site);
    }

    pub fn exceptions(&self) -> &BTreeMap<String, Mode> {
        &self.sites
    }

    fn rules(&self) -> Rules<'_> {
        Rules {
            standard: self.standard(),
            sites: &self.sites,
        }
    }

    pub fn script(&self) -> String {
        format!("({SCRIPT})({});", json(&self.rules()))
    }

    pub fn message(&self) -> String {
        json(&Message::DarkMode {
            rules: self.rules(),
        })
    }

    pub fn set_sites(&mut self, mode: Mode, text: &str) {
        self.sites.retain(|_, kept| *kept != mode);
        let sites = text
            .split(SITE_SEPARATOR)
            .map(str::trim)
            .filter(|site| !site.is_empty())
            .map(|site| (site.to_ascii_lowercase(), mode));
        self.sites.extend(sites);
    }

    pub fn sites_text(&self, mode: Mode) -> String {
        let sites: Vec<&str> = self
            .sites
            .iter()
            .filter(|(_, kept)| **kept == mode)
            .map(|(site, _)| site.as_str())
            .collect();
        sites.join(&format!("{SITE_SEPARATOR} "))
    }
}
