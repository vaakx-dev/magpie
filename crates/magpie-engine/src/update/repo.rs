use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repo {
    owner: String,
    name: String,
}

const MAGPIE_OWNER: &str = "vaakx-dev";
const MAGPIE_NAME: &str = "magpie";

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

    pub fn latest_release_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
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
