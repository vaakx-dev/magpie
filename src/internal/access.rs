use url::Url;

use super::SCHEME;

const ROOT_PATH: &str = "/";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Initiator {
    Browser,
    Internal,
    Web,
}

fn internal(url: &Url) -> bool {
    url.scheme() == SCHEME
}

fn normalized(url: &Url) -> String {
    let mut url = url.clone();
    if url.path().is_empty() {
        url.set_path(ROOT_PATH);
    }
    url.as_str().to_ascii_lowercase()
}

impl Initiator {
    pub fn page(source: Option<&Url>) -> Self {
        match source {
            Some(source) if internal(source) => Self::Internal,
            _ => Self::Web,
        }
    }
}

pub fn requested(requested: &Url, reported: &Url) -> bool {
    normalized(requested) == normalized(reported)
}

pub fn allowed(initiator: Initiator, target: &Url) -> bool {
    initiator != Initiator::Web || !internal(target)
}
