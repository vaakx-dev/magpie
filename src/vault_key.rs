use std::{
    fs, io,
    path::{Path, PathBuf},
    slice,
};

use magpie_engine::bitwarden::{Error, SymmetricKey, Vault};
use windows::{
    Win32::{
        Foundation::{HLOCAL, LocalFree},
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
    },
    core::w,
};
use zeroize::{Zeroize, Zeroizing};

const FILE_NAME: &str = "vault.key";

#[derive(Clone)]
pub struct VaultKey(PathBuf);

fn blob(bytes: &[u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr().cast_mut(),
    }
}

fn taken(blob: CRYPT_INTEGER_BLOB) -> Zeroizing<Vec<u8>> {
    let bytes = unsafe { slice::from_raw_parts_mut(blob.pbData, blob.cbData as usize) };
    let copy = Zeroizing::new(bytes.to_vec());
    bytes.zeroize();
    unsafe { LocalFree(Some(HLOCAL(blob.pbData.cast()))) };
    copy
}

fn protect(secret: &[u8]) -> io::Result<Zeroizing<Vec<u8>>> {
    let mut sealed = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptProtectData(
            &blob(secret),
            w!("Magpie Bitwarden key"),
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut sealed,
        )
    }
    .map_err(io::Error::other)?;
    Ok(taken(sealed))
}

fn unprotect(sealed: &[u8]) -> io::Result<Zeroizing<Vec<u8>>> {
    let mut secret = CRYPT_INTEGER_BLOB::default();
    unsafe {
        CryptUnprotectData(
            &blob(sealed),
            None,
            None,
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut secret,
        )
    }
    .map_err(io::Error::other)?;
    Ok(taken(secret))
}

impl VaultKey {
    pub fn in_folder(folder: &Path) -> Self {
        Self(folder.join(FILE_NAME))
    }

    pub fn saved(&self) -> bool {
        self.0.exists()
    }

    pub fn save(&self, vault: &Vault) -> Result<(), Error> {
        let key = vault.key()?.bytes();
        Ok(fs::write(&self.0, protect(&key)?)?)
    }

    pub fn unlock(&self, vault: &mut Vault) -> Result<(), Error> {
        let key = SymmetricKey::from_bytes(&unprotect(&fs::read(&self.0)?)?)?;
        vault.unlock_with_key(key)
    }

    pub fn lock(&self, vault: &mut Vault) -> io::Result<()> {
        vault.lock();
        self.forget()
    }

    pub fn sign_out(&self, vault: &mut Vault) -> Result<(), Error> {
        let signed_out = vault.sign_out();
        self.forget()?;
        signed_out
    }

    fn forget(&self) -> io::Result<()> {
        match fs::remove_file(&self.0) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }
}
