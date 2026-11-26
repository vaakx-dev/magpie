use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use super::{cipher_string::CipherString, error::Error, item::Sealed, kdf::Kdf, server::Server};
use crate::file;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub server: Server,
    pub email: String,
    pub kdf: Kdf,
    pub protected_key: CipherString,
    pub refresh_token: CipherString,
    pub logins: Vec<Sealed>,
}

const VAULT_FILE: &str = "vault.json";

impl Snapshot {
    pub fn load(folder: &Path) -> Result<Option<Self>, Error> {
        let text = match fs::read_to_string(folder.join(VAULT_FILE)) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(Error::Storage(error)),
        };
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|error| Error::Storage(io::Error::other(error)))
    }

    pub fn save(&self, folder: &Path) -> Result<(), Error> {
        fs::create_dir_all(folder)?;
        file::save(&folder.join(VAULT_FILE), self)?;
        Ok(())
    }

    pub fn delete(folder: &Path) -> Result<(), Error> {
        match fs::remove_file(folder.join(VAULT_FILE)) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(Error::Storage(error)),
            _ => Ok(()),
        }
    }
}
