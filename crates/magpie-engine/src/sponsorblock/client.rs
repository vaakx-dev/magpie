use std::{
    fmt,
    sync::{Mutex, PoisonError},
    time::{Duration, Instant},
};

use super::{api, cache::Cache, category::Categories, segment::Segment, video::VideoId};

#[derive(Debug)]
pub enum Error {
    Network(ureq::Error),
    Response(serde_json::Error),
}

pub struct Client {
    agent: ureq::Agent,
    cache: Mutex<Cache>,
}

const TIMEOUT: Duration = Duration::from_secs(3);
const CACHE_ENTRIES: usize = 256;
const CACHE_LIFETIME: Duration = Duration::from_secs(15 * 60);
const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;
const NOT_FOUND: u16 = 404;

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Network(error) => write!(formatter, "SponsorBlock request failed: {error}"),
            Self::Response(error) => write!(formatter, "SponsorBlock reply unreadable: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl Default for Client {
    fn default() -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .build()
            .into();
        Self {
            agent,
            cache: Mutex::new(Cache::new(CACHE_ENTRIES, CACHE_LIFETIME)),
        }
    }
}

impl Client {
    fn cache(&self) -> std::sync::MutexGuard<'_, Cache> {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn body(&self, address: &str) -> Result<Option<String>, Error> {
        let response = match self.agent.get(address).call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(NOT_FOUND)) => return Ok(None),
            Err(error) => return Err(Error::Network(error)),
        };
        response
            .into_body()
            .with_config()
            .limit(MAX_RESPONSE_BYTES)
            .read_to_string()
            .map(Some)
            .map_err(Error::Network)
    }

    fn download(&self, video: &VideoId, categories: Categories) -> Result<Vec<Segment>, Error> {
        let Some(body) = self.body(&api::address(video, categories))? else {
            return Ok(Vec::new());
        };
        api::parse(&body, video).map_err(Error::Response)
    }

    pub fn segments(&self, video: &VideoId, categories: Categories) -> Result<Vec<Segment>, Error> {
        if categories.is_empty() {
            return Ok(Vec::new());
        }
        let key = (video.clone(), categories);
        if let Some(segments) = self.cache().get(&key, Instant::now()) {
            return Ok(segments);
        }
        let segments = self.download(video, categories)?;
        self.cache().insert(key, segments.clone(), Instant::now());
        Ok(segments)
    }
}
