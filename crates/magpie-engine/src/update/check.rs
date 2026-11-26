use std::{fmt, time::Duration};

use serde::Serialize;

use super::{Asset, Channel, Package, Platform, Release, Repo, Version, checksum, package};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Offline,
    NotFound,
    RateLimited,
    Refused(u16),
    Unreadable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Download {
    pub name: String,
    pub url: String,
    pub size: u64,
    pub sha256: Option<String>,
    pub package: Package,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Offer {
    pub version: String,
    pub page: String,
    pub notes: String,
    pub download: Option<Download>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    UpToDate,
    Available(Offer),
    Failed { message: String },
}

pub struct Request<'a> {
    pub repo: &'a Repo,
    pub current: &'a Version,
    pub platform: Platform<'a>,
    pub channel: Channel,
}

const USER_AGENT: &str = "Magpie";
const ACCEPT: &str = "application/vnd.github+json";
const TIMEOUT: Duration = Duration::from_secs(15);
const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;
const NOT_FOUND: u16 = 404;
const FORBIDDEN: u16 = 403;
const TOO_MANY_REQUESTS: u16 = 429;

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Offline => write!(formatter, "Can't reach GitHub. Check your connection"),
            Self::NotFound => write!(formatter, "GitHub couldn't find the update"),
            Self::RateLimited => write!(formatter, "GitHub is busy. Try again later"),
            Self::Refused(code) => write!(formatter, "GitHub answered with error {code}"),
            Self::Unreadable => write!(formatter, "The release on GitHub is unreadable"),
        }
    }
}

impl From<ureq::Error> for Failure {
    fn from(error: ureq::Error) -> Self {
        match error {
            ureq::Error::StatusCode(NOT_FOUND) => Self::NotFound,
            ureq::Error::StatusCode(FORBIDDEN | TOO_MANY_REQUESTS) => Self::RateLimited,
            ureq::Error::StatusCode(code) => Self::Refused(code),
            _ => Self::Offline,
        }
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

pub fn fetch(address: &str) -> Result<String, Failure> {
    agent()
        .get(address)
        .header("Accept", ACCEPT)
        .call()?
        .into_body()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_to_string()
        .map_err(|_| Failure::Unreadable)
}

fn sha256(
    release: &Release,
    asset: &Asset,
    fetch: &impl Fn(&str) -> Result<String, Failure>,
) -> Result<Option<String>, Failure> {
    let Some(listing) = checksum::listing(&release.assets, &asset.name) else {
        return Ok(None);
    };
    let text = fetch(listing.url.as_str())?;
    Ok(checksum::sha256(&text, &asset.name))
}

fn download(
    release: &Release,
    platform: Platform,
    fetch: &impl Fn(&str) -> Result<String, Failure>,
) -> Result<Option<Download>, Failure> {
    let Some((asset, package)) = package::pick(&release.assets, platform) else {
        return Ok(None);
    };
    Ok(Some(Download {
        name: asset.name.clone(),
        url: asset.url.to_string(),
        size: asset.size,
        sha256: sha256(release, asset, fetch)?,
        package,
    }))
}

fn latest(
    request: &Request,
    fetch: &impl Fn(&str) -> Result<String, Failure>,
) -> Result<Status, Failure> {
    let body = match fetch(&request.repo.release_url(request.channel)) {
        Err(Failure::NotFound) => return Ok(Status::UpToDate),
        fetched => fetched?,
    };
    let release = Release::parse(&body, request.channel).ok_or(Failure::Unreadable)?;
    if !request.channel.offers(&release.version, request.current) {
        return Ok(Status::UpToDate);
    }
    let download = download(&release, request.platform, fetch)?;
    Ok(Status::Available(Offer {
        version: release.version.to_string(),
        page: release.page.to_string(),
        notes: release.notes,
        download,
    }))
}

pub fn check(request: &Request, fetch: impl Fn(&str) -> Result<String, Failure>) -> Status {
    latest(request, &fetch).unwrap_or_else(|failure| Status::Failed {
        message: failure.to_string(),
    })
}
