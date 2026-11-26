use serde::Deserialize;
use url::Url;

use super::Version;

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

impl Release {
    pub fn parse(json: &str) -> Option<Self> {
        let unchecked: UncheckedRelease = serde_json::from_str(json).ok()?;
        Some(Self {
            version: Version::parse(&unchecked.tag_name)?,
            page: secure(&unchecked.html_url)?,
            notes: unchecked.body.unwrap_or_default(),
            assets: unchecked
                .assets
                .into_iter()
                .filter_map(UncheckedAsset::checked)
                .collect(),
        })
    }
}
