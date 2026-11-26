use std::{
    fs, io,
    path::{Path, PathBuf},
    slice,
};

use windows::{
    Security::Credentials::UI::{
        UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
    },
    Win32::{
        Foundation::{HLOCAL, HWND, LocalFree},
        Security::Cryptography::{
            CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
        },
        System::WinRT::IUserConsentVerifierInterop,
    },
    core::{HSTRING, factory, w},
};
use windows_future::IAsyncOperation;
use zeroize::{Zeroize, Zeroizing};

const KEY_FILE: &str = "hello.key";

#[derive(Clone)]
pub struct KeyFile(PathBuf);

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

impl KeyFile {
    pub fn in_folder(folder: &Path) -> Self {
        Self(folder.join(KEY_FILE))
    }

    pub fn exists(&self) -> bool {
        self.0.exists()
    }

    pub fn store(&self, key: &[u8]) -> io::Result<()> {
        fs::write(&self.0, protect(key)?)
    }

    pub fn load(&self) -> io::Result<Zeroizing<Vec<u8>>> {
        unprotect(&fs::read(&self.0)?)
    }

    pub fn forget(&self) -> io::Result<()> {
        match fs::remove_file(&self.0) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }
}

pub async fn available() -> bool {
    let Ok(check) = UserConsentVerifier::CheckAvailabilityAsync() else {
        return false;
    };
    check
        .await
        .is_ok_and(|availability| availability == UserConsentVerifierAvailability::Available)
}

pub async fn verify(window: HWND, reason: HSTRING) -> bool {
    let Ok(interop) = factory::<UserConsentVerifier, IUserConsentVerifierInterop>() else {
        return false;
    };
    let request: windows::core::Result<IAsyncOperation<UserConsentVerificationResult>> =
        unsafe { interop.RequestVerificationForWindowAsync(window, &reason) };
    let Ok(request) = request else {
        return false;
    };
    request
        .await
        .is_ok_and(|result| result == UserConsentVerificationResult::Verified)
}
