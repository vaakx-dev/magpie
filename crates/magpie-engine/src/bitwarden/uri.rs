use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use url::{Host, Url};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::{cipher_string::CipherString, error::Error, item::Unread, key::SymmetricKey};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum MatchRule {
    #[default]
    Domain,
    Host,
    StartsWith,
    Exact,
    RegularExpression,
    Never,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedUri {
    pub uri: Option<CipherString>,
    #[serde(rename = "match")]
    pub rule: Option<MatchRule>,
    #[serde(flatten)]
    pub unread: Unread,
}

#[derive(Clone, Debug, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct LoginUri {
    pub uri: String,
    #[zeroize(skip)]
    pub rule: MatchRule,
}

pub struct Page<'a> {
    address: &'a str,
    url: Url,
}

const MATCH_RULES: [MatchRule; 6] = [
    MatchRule::Domain,
    MatchRule::Host,
    MatchRule::StartsWith,
    MatchRule::Exact,
    MatchRule::RegularExpression,
    MatchRule::Never,
];

fn url_of(uri: &str) -> Option<Url> {
    match uri.contains("://") {
        true => Url::parse(uri).ok(),
        false => Url::parse(&format!("http://{uri}")).ok(),
    }
}

fn base_domain(url: &Url) -> Option<&str> {
    match url.host()? {
        Host::Domain(domain) => Some(psl::domain_str(domain).unwrap_or(domain)),
        Host::Ipv4(_) | Host::Ipv6(_) => url.host_str(),
    }
}

fn same_host(first: &Url, second: &Url) -> bool {
    first.host_str().is_some()
        && (first.host_str(), first.port()) == (second.host_str(), second.port())
}

impl TryFrom<u8> for MatchRule {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Error> {
        MATCH_RULES
            .get(usize::from(value))
            .copied()
            .ok_or(Error::Unsupported(format!("URI match type {value}")))
    }
}

impl From<MatchRule> for u8 {
    fn from(rule: MatchRule) -> Self {
        rule as u8
    }
}

impl<'a> Page<'a> {
    pub fn parse(address: &'a str) -> Option<Self> {
        let url = Url::parse(address).ok()?;
        Some(Self { address, url })
    }
}

impl SealedUri {
    pub fn open(&self, key: &SymmetricKey) -> Result<Option<LoginUri>, Error> {
        let Some(uri) = &self.uri else {
            return Ok(None);
        };
        Ok(Some(LoginUri {
            uri: uri.decrypt_text(key)?,
            rule: self.rule.unwrap_or_default(),
        }))
    }
}

impl LoginUri {
    pub fn matches(&self, page: &Page) -> bool {
        match self.rule {
            MatchRule::Domain => url_of(&self.uri).is_some_and(|url| {
                base_domain(&url).is_some_and(|domain| Some(domain) == base_domain(&page.url))
            }),
            MatchRule::Host => url_of(&self.uri).is_some_and(|url| same_host(&url, &page.url)),
            MatchRule::StartsWith => page.address.starts_with(self.uri.as_str()),
            MatchRule::Exact => page.address == self.uri,
            MatchRule::RegularExpression => RegexBuilder::new(&self.uri)
                .case_insensitive(true)
                .build()
                .is_ok_and(|pattern| pattern.is_match(page.address)),
            MatchRule::Never => false,
        }
    }
}
