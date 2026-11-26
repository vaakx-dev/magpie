use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use magpie_engine::file;

const SENTINEL_FILE: &str = "running";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Clean,
    Unexpected,
}

pub struct Sentinel {
    path: PathBuf,
}

impl Sentinel {
    pub fn raise(folder: &Path) -> io::Result<(Self, Exit)> {
        fs::create_dir_all(folder)?;
        let path = folder.join(SENTINEL_FILE);
        let exit = match path.try_exists()? {
            true => Exit::Unexpected,
            false => Exit::Clean,
        };
        fs::write(&path, [])?;
        Ok((Self { path }, exit))
    }

    pub fn lower(&self) -> io::Result<()> {
        match fs::remove_file(&self.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            removed => removed,
        }
    }
}

pub fn load<T: Serialize + DeserializeOwned + Default>(
    path: &Path,
    backup: &Path,
) -> io::Result<T> {
    match file::read(path)? {
        Some(value) => {
            file::save(backup, &value)?;
            Ok(value)
        }
        None => file::load(backup),
    }
}
