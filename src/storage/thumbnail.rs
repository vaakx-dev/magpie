use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use sha2::{Digest, Sha256};

use crate::role::Role;

const EXTENSION: &str = ".jpg";
const NAME_BYTES: usize = 16;
const ORPHAN_GRACE: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone)]
pub struct Cache {
    folder: PathBuf,
    role: Role,
}

fn name(page: &str) -> String {
    let digest = Sha256::digest(page.as_bytes());
    let hex: String = digest[..NAME_BYTES]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("{hex}{EXTENSION}")
}

fn age(path: &Path, now: SystemTime) -> io::Result<Duration> {
    let modified = fs::metadata(path)?.modified()?;
    Ok(now.duration_since(modified).unwrap_or_default())
}

impl Cache {
    pub(super) fn new(folder: PathBuf, role: Role) -> Self {
        Self { folder, role }
    }

    fn path(&self, page: &str) -> PathBuf {
        self.folder.join(name(page))
    }

    pub fn read(&self, page: &str) -> Option<Vec<u8>> {
        fs::read(self.path(page)).ok()
    }

    pub fn write(&self, page: &str, jpeg: &[u8]) -> io::Result<()> {
        if self.role.private() {
            return Ok(());
        }
        fs::create_dir_all(&self.folder)?;
        fs::write(self.path(page), jpeg)
    }

    pub fn sweep<'a>(&self, open: impl IntoIterator<Item = &'a str>) -> io::Result<()> {
        if !self.role.keeps_session() || !self.folder.exists() {
            return Ok(());
        }
        let kept: HashSet<String> = open.into_iter().map(name).collect();
        let now = SystemTime::now();
        let orphans = fs::read_dir(&self.folder)?
            .filter_map(Result::ok)
            .filter(|entry| !kept.contains(entry.file_name().to_string_lossy().as_ref()))
            .map(|entry| entry.path())
            .filter(|path| age(path, now).is_ok_and(|age| age >= ORPHAN_GRACE));
        for orphan in orphans {
            fs::remove_file(orphan)?;
        }
        Ok(())
    }
}
