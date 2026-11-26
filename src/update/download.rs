use std::{
    fmt,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use magpie_engine::update::{Download, Failure};
use sha2::{Digest, Sha256};
use ureq::{
    Body,
    http::{Response, header::LOCATION},
};
use url::Url;

#[derive(Debug)]
pub enum Fault {
    Network(Failure),
    Untrusted,
    Unverified,
    Mismatch,
    Disk(io::Error),
}

const OFFICIAL_HOSTS: [&str; 3] = [
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];
const OFFICIAL_SCHEME: &str = "https";
const MAX_REDIRECTS: usize = 5;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const CHUNK_BYTES: usize = 64 * 1024;
const PARTIAL_SUFFIX: &str = ".part";
const WHOLE_PERCENT: u64 = 100;

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Network(failure) => write!(formatter, "{failure}"),
            Self::Untrusted => write!(
                formatter,
                "The update isn't hosted on Magpie's GitHub releases, so it wasn't downloaded"
            ),
            Self::Unverified => write!(
                formatter,
                "The release has no checksum for this download, so it wasn't installed"
            ),
            Self::Mismatch => write!(
                formatter,
                "The download didn't match its checksum, so it wasn't installed"
            ),
            Self::Disk(error) => write!(formatter, "Couldn't save the update: {error}"),
        }
    }
}

impl From<io::Error> for Fault {
    fn from(error: io::Error) -> Self {
        Self::Disk(error)
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_per_read(Some(READ_TIMEOUT))
        .max_redirects(0)
        .build()
        .into()
}

fn official_host(url: &Url) -> bool {
    url.host_str()
        .is_some_and(|host| OFFICIAL_HOSTS.contains(&host))
}

fn official(address: &str) -> Result<Url, Fault> {
    let url = Url::parse(address).map_err(|_| Fault::Untrusted)?;
    let trusted = url.scheme() == OFFICIAL_SCHEME
        && official_host(&url)
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none();
    trusted.then_some(url).ok_or(Fault::Untrusted)
}

fn redirect(response: &Response<Body>, from: &Url) -> Result<Option<Url>, Fault> {
    if !response.status().is_redirection() {
        return Ok(None);
    }
    let location = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(Fault::Network(Failure::Unreadable))?;
    let next = from.join(location).map_err(|_| Fault::Untrusted)?;
    official(next.as_str()).map(Some)
}

fn request(address: &str) -> Result<Response<Body>, Fault> {
    let agent = agent();
    let mut url = official(address)?;
    for _ in 0..=MAX_REDIRECTS {
        let response = agent
            .get(url.as_str())
            .call()
            .map_err(|error| Fault::Network(error.into()))?;
        match redirect(&response, &url)? {
            Some(next) => url = next,
            None => return Ok(response),
        }
    }
    Err(Fault::Network(Failure::Unreadable))
}

fn expected_sha256(download: &Download) -> Result<&str, Fault> {
    download.sha256.as_deref().ok_or(Fault::Unverified)
}

fn file_name(download: &Download) -> Result<&str, Fault> {
    Path::new(&download.name)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| *name == download.name)
        .ok_or(Fault::Network(Failure::Unreadable))
}

fn percent(received: u64, size: u64) -> u8 {
    let done = (received * WHOLE_PERCENT)
        .checked_div(size)
        .unwrap_or(WHOLE_PERCENT);
    done.min(WHOLE_PERCENT) as u8
}

fn save(download: &Download, path: &Path, progress: &mut impl FnMut(u8)) -> Result<String, Fault> {
    let response = request(&download.url)?;
    let mut body = response
        .into_body()
        .into_with_config()
        .limit(download.size)
        .reader();
    let mut file = File::create(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; CHUNK_BYTES];
    let (mut received, mut reported) = (0, None);
    loop {
        let read = body
            .read(&mut buffer)
            .map_err(|_| Fault::Network(Failure::Offline))?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        hasher.update(chunk);
        file.write_all(chunk)?;
        received += read as u64;
        let done = percent(received, download.size);
        if reported != Some(done) {
            reported = Some(done);
            progress(done);
        }
    }
    file.sync_all()?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn download(
    download: &Download,
    folder: &Path,
    mut progress: impl FnMut(u8),
) -> Result<PathBuf, Fault> {
    let target = folder.join(file_name(download)?);
    let expected = expected_sha256(download)?;
    let partial = folder.join(format!("{}{PARTIAL_SUFFIX}", download.name));
    fs::create_dir_all(folder)?;
    let digest = save(download, &partial, &mut progress)?;
    if digest != expected {
        fs::remove_file(&partial)?;
        return Err(Fault::Mismatch);
    }
    fs::rename(&partial, &target)?;
    Ok(target)
}
