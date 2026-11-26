use base64::{Engine, engine::general_purpose::STANDARD};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use super::{cipher_string::CipherString, error::Error};

pub const KEY_BYTES: usize = 32;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; KEY_BYTES]);

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SymmetricKey {
    encryption: [u8; KEY_BYTES],
    mac: [u8; KEY_BYTES],
}

const PASSWORD_HASH_ROUNDS: u32 = 1;
const ENCRYPTION_INFO: &[u8] = b"enc";
const MAC_INFO: &[u8] = b"mac";

fn expand(prk: &[u8; KEY_BYTES], info: &[u8]) -> [u8; KEY_BYTES] {
    let hkdf = Hkdf::<Sha256>::from_prk(prk).expect("a 32-byte key is a valid HKDF PRK");
    let mut output = [0; KEY_BYTES];
    hkdf.expand(info, &mut output)
        .expect("32 bytes is a valid HKDF output length");
    output
}

impl MasterKey {
    pub fn new(bytes: [u8; KEY_BYTES]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; KEY_BYTES] {
        &self.0
    }

    pub fn password_hash(&self, password: &str) -> Zeroizing<String> {
        let mut hash = Zeroizing::new([0; KEY_BYTES]);
        pbkdf2::pbkdf2_hmac::<Sha256>(
            &self.0,
            password.as_bytes(),
            PASSWORD_HASH_ROUNDS,
            hash.as_mut(),
        );
        Zeroizing::new(STANDARD.encode(hash.as_ref()))
    }

    pub fn stretch(&self) -> SymmetricKey {
        SymmetricKey {
            encryption: expand(&self.0, ENCRYPTION_INFO),
            mac: expand(&self.0, MAC_INFO),
        }
    }

    pub fn unseal(&self, protected: &CipherString) -> Result<SymmetricKey, Error> {
        SymmetricKey::from_bytes(&protected.decrypt(&self.stretch())?)
    }
}

impl SymmetricKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let (encryption, mac) = bytes
            .split_at_checked(KEY_BYTES)
            .filter(|(_, mac)| mac.len() == KEY_BYTES)
            .ok_or(Error::Malformed("symmetric key"))?;
        Ok(Self {
            encryption: encryption.try_into().expect("split at 32 bytes"),
            mac: mac.try_into().expect("checked to be 32 bytes"),
        })
    }

    pub fn bytes(&self) -> Zeroizing<Vec<u8>> {
        let mut bytes = Zeroizing::new(Vec::with_capacity(2 * KEY_BYTES));
        bytes.extend_from_slice(&self.encryption);
        bytes.extend_from_slice(&self.mac);
        bytes
    }

    pub fn encryption(&self) -> &[u8; KEY_BYTES] {
        &self.encryption
    }

    pub fn mac(&self) -> &[u8; KEY_BYTES] {
        &self.mac
    }
}
