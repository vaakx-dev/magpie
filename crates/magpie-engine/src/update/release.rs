use serde::Deserialize;
use url::Url;

use super::{Channel, Version};

#[derive(Deserialize)]
struct UncheckedAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Deserialize)]
struct UncheckedRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    assets: Vec<UncheckedAsset>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub url: Url,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: Version,
    pub page: Url,
    pub notes: String,
    pub assets: Vec<Asset>,
}

const VERSION_LINE: &str = "Version:";

fn secure(address: &str) -> Option<Url> {
    Url::parse(address)
        .ok()
        .filter(|url| url.scheme() == "https")
}

impl UncheckedAsset {
    fn checked(self) -> Option<Asset> {
        Some(Asset {
            url: secure(&self.browser_download_url)?,
            name: self.name,
            size: self.size,
        })
    }
}

fn stated_version(notes: &str) -> Option<Version> {
    notes
        .lines()
        .find_map(|line| line.trim().strip_prefix(VERSION_LINE))
        .and_then(Version::parse)
}

fn version(tag: &str, notes: &str, channel: Channel) -> Option<Version> {
    match channel {
        Channel::Stable => Version::parse(tag),
        Channel::Nightly => stated_version(notes),
    }
}

impl Release {
    pub fn parse(json: &str, channel: Channel) -> Option<Self> {
        let UncheckedRelease {
            tag_name,
            html_url,
            body,
            assets,
        } = serde_json::from_str(json).ok()?;
        let notes = body.unwrap_or_default();
        Some(Self {
            version: version(&tag_name, &notes, channel)?,
            page: secure(&html_url)?,
            notes,
            assets: assets
                .into_iter()
                .filter_map(UncheckedAsset::checked)
                .collect(),
        })
    }
}
