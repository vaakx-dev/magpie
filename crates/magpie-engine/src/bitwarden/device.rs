use std::{fs, io, path::Path};

use uuid::Uuid;

use crate::file;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceId(String);

const DEVICE_FILE: &str = "device";

impl DeviceId {
    pub fn load_or_create(folder: &Path) -> io::Result<Self> {
        let path = folder.join(DEVICE_FILE);
        let stored = match fs::read_to_string(&path) {
            Ok(text) => Uuid::parse_str(text.trim()).ok().map(|id| id.to_string()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        if let Some(id) = stored {
            return Ok(Self(id));
        }
        let id = Uuid::new_v4().to_string();
        fs::create_dir_all(folder)?;
        file::replace(&path, id.as_bytes())?;
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
