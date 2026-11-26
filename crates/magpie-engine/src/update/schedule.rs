use std::{
    fs, io,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::file;

pub const CHECK_INTERVAL: Duration = Duration::from_secs(12 * 60 * 60);

pub struct Schedule {
    path: PathBuf,
}

impl Schedule {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    fn last_check(&self) -> Option<SystemTime> {
        let text = fs::read_to_string(&self.path).ok()?;
        let secs: u64 = text.trim().parse().ok()?;
        Some(UNIX_EPOCH + Duration::from_secs(secs))
    }

    pub fn due(&self, now: SystemTime) -> bool {
        self.last_check()
            .and_then(|last| now.duration_since(last).ok())
            .is_none_or(|elapsed| elapsed >= CHECK_INTERVAL)
    }

    pub fn record(&self, now: SystemTime) -> io::Result<()> {
        let secs = now
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_secs();
        if let Some(folder) = self.path.parent() {
            fs::create_dir_all(folder)?;
        }
        file::replace(&self.path, secs.to_string().as_bytes())
    }
}
