use std::fmt;

use super::Channel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repo {
    owner: String,
    name: String,
}

const MAGPIE_OWNER: &str = "vaakx-dev";
const MAGPIE_NAME: &str = "magpie";
const NIGHTLY_TAG: &str = "nightly";

fn part(text: &str) -> Option<String> {
    let allowed = |character: char| character.is_ascii_alphanumeric() || "-_.".contains(character);
    let valid = !text.is_empty() && text.chars().all(allowed);
    valid.then(|| text.to_owned())
}

impl Repo {
    pub fn parse(text: &str) -> Option<Self> {
        let (owner, name) = text.trim().split_once('/')?;
        Some(Self {
            owner: part(owner)?,
            name: part(name)?,
        })
    }

    pub fn release_url(&self, channel: Channel) -> String {
        let release = match channel {
            Channel::Stable => "latest".to_owned(),
            Channel::Nightly => format!("tags/{NIGHTLY_TAG}"),
        };
        format!(
            "https://api.github.com/repos/{}/{}/releases/{release}",
            self.owner, self.name
        )
    }
}

impl Default for Repo {
    fn default() -> Self {
        Self {
            owner: MAGPIE_OWNER.to_owned(),
            name: MAGPIE_NAME.to_owned(),
        }
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}/{}", self.owner, self.name)
    }
}
