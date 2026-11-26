use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::{
    error::Error,
    key::{KEY_BYTES, MasterKey},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Kdf {
    Pbkdf2 {
        iterations: u32,
    },
    Argon2id {
        iterations: u32,
        memory_mib: u32,
        parallelism: u32,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Prelogin {
    kdf: u8,
    kdf_iterations: u32,
    kdf_memory: Option<u32>,
    kdf_parallelism: Option<u32>,
}

const PBKDF2: u8 = 0;
const ARGON2ID: u8 = 1;
const PBKDF2_ITERATIONS: std::ops::RangeInclusive<u32> = 5_000..=2_000_000;
const ARGON2_ITERATIONS: std::ops::RangeInclusive<u32> = 2..=10;
const ARGON2_MEMORY_MIB: std::ops::RangeInclusive<u32> = 16..=1024;
const ARGON2_PARALLELISM: std::ops::RangeInclusive<u32> = 1..=16;
const KIB_PER_MIB: u32 = 1024;

fn email_salt(email: &str) -> String {
    email.trim().to_lowercase()
}

impl Kdf {
    pub fn from_prelogin(body: &str) -> Result<Self, Error> {
        let prelogin: Prelogin = serde_json::from_str(body)?;
        let kdf = match (prelogin.kdf, prelogin.kdf_memory, prelogin.kdf_parallelism) {
            (PBKDF2, _, _) => Self::Pbkdf2 {
                iterations: prelogin.kdf_iterations,
            },
            (ARGON2ID, Some(memory_mib), Some(parallelism)) => Self::Argon2id {
                iterations: prelogin.kdf_iterations,
                memory_mib,
                parallelism,
            },
            (kind, _, _) => return Err(Error::Unsupported(format!("key derivation type {kind}"))),
        };
        kdf.safe()
    }

    fn safe(self) -> Result<Self, Error> {
        let safe = match self {
            Self::Pbkdf2 { iterations } => PBKDF2_ITERATIONS.contains(&iterations),
            Self::Argon2id {
                iterations,
                memory_mib,
                parallelism,
            } => {
                ARGON2_ITERATIONS.contains(&iterations)
                    && ARGON2_MEMORY_MIB.contains(&memory_mib)
                    && ARGON2_PARALLELISM.contains(&parallelism)
            }
        };
        safe.then_some(self).ok_or(Error::WeakKdf)
    }

    pub fn master_key(&self, password: &str, email: &str) -> Result<MasterKey, Error> {
        let salt = email_salt(email);
        let mut key = Zeroizing::new([0; KEY_BYTES]);
        match *self {
            Self::Pbkdf2 { iterations } => pbkdf2::pbkdf2_hmac::<Sha256>(
                password.as_bytes(),
                salt.as_bytes(),
                iterations,
                key.as_mut(),
            ),
            Self::Argon2id {
                iterations,
                memory_mib,
                parallelism,
            } => {
                let params = Params::new(
                    memory_mib * KIB_PER_MIB,
                    iterations,
                    parallelism,
                    Some(KEY_BYTES),
                )
                .map_err(|_| Error::WeakKdf)?;
                Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
                    .hash_password_into(
                        password.as_bytes(),
                        &Sha256::digest(salt.as_bytes()),
                        key.as_mut(),
                    )
                    .map_err(|_| Error::WeakKdf)?;
            }
        }
        Ok(MasterKey::new(*key))
    }
}
