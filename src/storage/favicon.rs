use std::{fs, io, path::PathBuf};

use crate::role::Role;

const EXTENSION: &str = ".png";

pub struct Cache {
    folder: PathBuf,
    role: Role,
}

impl Cache {
    pub(super) fn new(folder: PathBuf, role: Role) -> Self {
        Self { folder, role }
    }

    fn path(&self, site: &str) -> PathBuf {
        self.folder.join(format!("{site}{EXTENSION}"))
    }

    pub fn read(&self, site: &str) -> Option<Vec<u8>> {
        fs::read(self.path(site)).ok()
    }

    pub fn write(&self, site: &str, png: &[u8]) -> io::Result<()> {
        if self.role.private() {
            return Ok(());
        }
        fs::create_dir_all(&self.folder)?;
        fs::write(self.path(site), png)
    }
}
