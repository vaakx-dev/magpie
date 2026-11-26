use std::fmt;

use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct VideoId(String);

const LENGTH: usize = 11;
const PREFIX_BYTES: usize = 2;
const WATCH_HOSTS: [&str; 6] = [
    "youtube.com",
    "www.youtube.com",
    "m.youtube.com",
    "music.youtube.com",
    "youtube-nocookie.com",
    "www.youtube-nocookie.com",
];
const SHORT_HOSTS: [&str; 2] = ["youtu.be", "www.youtu.be"];
const ID_PATHS: [&str; 4] = ["shorts", "embed", "live", "v"];

fn id_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-' || character == '_'
}

fn query_id(url: &Url) -> Option<String> {
    url.query_pairs()
        .find(|(key, _)| key == "v")
        .map(|(_, value)| value.into_owned())
}

fn watch_id(url: &Url) -> Option<String> {
    let mut parts = url.path_segments()?;
    match (parts.next()?, parts.next()) {
        ("watch", _) => query_id(url),
        (kind, Some(id)) if ID_PATHS.contains(&kind) => Some(id.to_owned()),
        _ => None,
    }
}

fn short_id(url: &Url) -> Option<String> {
    url.path_segments()?.next().map(str::to_owned)
}

impl VideoId {
    pub fn parse(text: &str) -> Option<Self> {
        let valid = text.len() == LENGTH && text.chars().all(id_character);
        valid.then(|| Self(text.to_owned()))
    }

    pub fn from_url(url: &Url) -> Option<Self> {
        let host = url.host_str()?;
        let id = match host {
            host if WATCH_HOSTS.contains(&host) => watch_id(url),
            host if SHORT_HOSTS.contains(&host) => short_id(url),
            _ => None,
        }?;
        Self::parse(&id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn hash_prefix(&self) -> String {
        Sha256::digest(self.0.as_bytes())[..PREFIX_BYTES]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

impl TryFrom<String> for VideoId {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::parse(&text).ok_or_else(|| format!("not a YouTube video id: {text}"))
    }
}

impl fmt::Display for VideoId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
