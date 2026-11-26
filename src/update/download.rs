use std::{
    fmt,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use magpie_engine::update::{Download, Failure};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub enum Fault {
    Network(Failure),
    Mismatch,
    Disk(io::Error),
}

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const CHUNK_BYTES: usize = 64 * 1024;
const PARTIAL_SUFFIX: &str = ".part";
const WHOLE_PERCENT: u64 = 100;

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Network(failure) => write!(formatter, "{failure}"),
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
        .build()
        .into()
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
    let response = agent()
        .get(&download.url)
        .call()
        .map_err(|error| Fault::Network(error.into()))?;
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

fn verify(download: &Download, digest: &str) -> Result<(), Fault> {
    match &download.sha256 {
        Some(expected) if expected != digest => Err(Fault::Mismatch),
        _ => Ok(()),
    }
}

pub fn download(
    download: &Download,
    folder: &Path,
    mut progress: impl FnMut(u8),
) -> Result<PathBuf, Fault> {
    let target = folder.join(file_name(download)?);
    let partial = folder.join(format!("{}{PARTIAL_SUFFIX}", download.name));
    fs::create_dir_all(folder)?;
    let digest = save(download, &partial, &mut progress)?;
    if let Err(fault) = verify(download, &digest) {
        fs::remove_file(&partial)?;
        return Err(fault);
    }
    fs::rename(&partial, &target)?;
    Ok(target)
}
