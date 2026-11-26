use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

const TEMP_EXTENSION: &str = "tmp";
const CORRUPT_EXTENSION: &str = "corrupt";

fn sibling(path: &Path, extension: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".");
    name.push(extension);
    PathBuf::from(name)
}

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

pub fn replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temp = sibling(path, TEMP_EXTENSION);
    write_synced(&temp, bytes)?;
    fs::rename(&temp, path)
}

pub fn save<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    replace(path, &bytes)
}

fn parse<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn set_aside(path: &Path) -> io::Result<()> {
    fs::rename(path, corrupt_copy(path))
}

pub fn read<T: DeserializeOwned>(path: &Path) -> io::Result<Option<T>> {
    match parse(path) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
            set_aside(path)?;
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

pub fn load<T: DeserializeOwned + Default>(path: &Path) -> io::Result<T> {
    Ok(read(path)?.unwrap_or_default())
}

pub fn corrupt_copy(path: &Path) -> PathBuf {
    sibling(path, CORRUPT_EXTENSION)
}
