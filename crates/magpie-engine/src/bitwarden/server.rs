use serde::{Deserialize, Serialize};
use url::Url;

use super::error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Server {
    UnitedStates,
    Europe,
    SelfHosted(Url),
}

const UNITED_STATES_HOSTS: [&str; 2] = ["vault.bitwarden.com", "bitwarden.com"];
const EUROPE_HOSTS: [&str; 2] = ["vault.bitwarden.eu", "bitwarden.eu"];
const WEB_SCHEMES: [&str; 2] = ["https", "http"];

fn with_scheme(input: &str) -> String {
    match input.contains("://") {
        true => input.to_owned(),
        false => format!("https://{input}"),
    }
}

fn base(mut url: Url) -> Url {
    url.set_query(None);
    url.set_fragment(None);
    let path = url.path().trim_end_matches('/').to_owned();
    url.set_path(&path);
    url
}

impl Server {
    pub fn parse(input: &str) -> Result<Self, Error> {
        let invalid = || Error::Address(input.to_owned());
        let url = Url::parse(&with_scheme(input.trim())).map_err(|_| invalid())?;
        let host = url.host_str().ok_or_else(invalid)?;
        if !WEB_SCHEMES.contains(&url.scheme()) {
            return Err(invalid());
        }
        if UNITED_STATES_HOSTS.contains(&host) {
            return Ok(Self::UnitedStates);
        }
        if EUROPE_HOSTS.contains(&host) {
            return Ok(Self::Europe);
        }
        Ok(Self::SelfHosted(base(url)))
    }

    pub fn address(&self) -> String {
        match self {
            Self::UnitedStates => "https://vault.bitwarden.com".to_owned(),
            Self::Europe => "https://vault.bitwarden.eu".to_owned(),
            Self::SelfHosted(url) => url.as_str().trim_end_matches('/').to_owned(),
        }
    }

    pub fn identity(&self) -> String {
        match self {
            Self::UnitedStates => "https://identity.bitwarden.com".to_owned(),
            Self::Europe => "https://identity.bitwarden.eu".to_owned(),
            Self::SelfHosted(_) => format!("{}/identity", self.address()),
        }
    }

    pub fn api(&self) -> String {
        match self {
            Self::UnitedStates => "https://api.bitwarden.com".to_owned(),
            Self::Europe => "https://api.bitwarden.eu".to_owned(),
            Self::SelfHosted(_) => format!("{}/api", self.address()),
        }
    }
}

impl TryFrom<String> for Server {
    type Error = Error;

    fn try_from(address: String) -> Result<Self, Error> {
        Self::parse(&address)
    }
}

impl From<Server> for String {
    fn from(server: Server) -> Self {
        server.address()
    }
}
