use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tempfile::TempDir;

use super::Failure;

const COPY_NAME: &str = "snapshot.sqlite";
const COMPANION_SUFFIXES: [&str; 2] = ["-wal", "-journal"];

pub struct Snapshot {
    connection: Connection,
    _folder: TempDir,
}

fn companion(path: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(suffix);
    PathBuf::from(name)
}

fn copy_companions(source: &Path, copy: &Path) -> io::Result<()> {
    COMPANION_SUFFIXES
        .iter()
        .map(|suffix| (companion(source, suffix), companion(copy, suffix)))
        .filter(|(from, _)| from.exists())
        .try_for_each(|(from, to)| fs::copy(from, to).map(drop))
}

impl Snapshot {
    pub fn open(database: &Path) -> Result<Self, Failure> {
        let folder = TempDir::new()?;
        let copy = folder.path().join(COPY_NAME);
        fs::copy(database, &copy)?;
        copy_companions(database, &copy)?;
        let connection = Connection::open(&copy)?;
        Ok(Self {
            connection,
            _folder: folder,
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}
