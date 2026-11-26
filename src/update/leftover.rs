use std::{fs, io, path::Path};

use super::FAILURE_FILE;

pub fn failure(folder: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(folder.join(FAILURE_FILE)) {
        Ok(message) => Ok(Some(message.trim().to_owned())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn clear(folder: &Path) -> io::Result<()> {
    match fs::remove_dir_all(folder) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        removed => removed,
    }
}
