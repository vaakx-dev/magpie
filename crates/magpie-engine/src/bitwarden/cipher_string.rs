use std::{fmt, str::FromStr};

use aes::Aes256;
use base64::{Engine, engine::general_purpose::STANDARD};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::Zeroizing;

use super::{error::Error, key::SymmetricKey};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CipherString {
    iv: [u8; IV_BYTES],
    data: Vec<u8>,
    mac: [u8; MAC_BYTES],
}

const IV_BYTES: usize = 16;
const MAC_BYTES: usize = 32;
const AES_CBC_HMAC: &str = "2";

fn decoded<const BYTES: usize>(part: &str) -> Result<[u8; BYTES], Error> {
    decoded_vec(part)?
        .try_into()
        .map_err(|_| Error::Malformed("cipher string"))
}

fn decoded_vec(part: &str) -> Result<Vec<u8>, Error> {
    STANDARD
        .decode(part)
        .map_err(|_| Error::Malformed("cipher string"))
}

fn authenticator(key: &SymmetricKey, iv: &[u8], data: &[u8]) -> Hmac<Sha256> {
    let mut mac =
        <Hmac<Sha256> as Mac>::new_from_slice(key.mac()).expect("HMAC accepts any key length");
    mac.update(iv);
    mac.update(data);
    mac
}

impl CipherString {
    pub fn encrypt(plain: &[u8], key: &SymmetricKey) -> Result<Self, Error> {
        let mut iv = [0; IV_BYTES];
        getrandom::fill(&mut iv).map_err(Error::Random)?;
        let data = cbc::Encryptor::<Aes256>::new(key.encryption().into(), &iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(plain);
        let mac = authenticator(key, &iv, &data)
            .finalize()
            .into_bytes()
            .into();
        Ok(Self { iv, data, mac })
    }

    pub fn decrypt(&self, key: &SymmetricKey) -> Result<Zeroizing<Vec<u8>>, Error> {
        authenticator(key, &self.iv, &self.data)
            .verify_slice(&self.mac)
            .map_err(|_| Error::Tampered)?;
        cbc::Decryptor::<Aes256>::new(key.encryption().into(), &self.iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&self.data)
            .map(Zeroizing::new)
            .map_err(|_| Error::Malformed("cipher padding"))
    }

    pub fn decrypt_text(&self, key: &SymmetricKey) -> Result<String, Error> {
        let mut plain = self.decrypt(key)?;
        String::from_utf8(std::mem::take(&mut *plain)).map_err(|_| Error::Malformed("UTF-8 text"))
    }
}

impl FromStr for CipherString {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Error> {
        let (kind, body) = text
            .split_once('.')
            .ok_or(Error::Malformed("cipher string"))?;
        if kind != AES_CBC_HMAC {
            return Err(Error::Unsupported(format!("encryption type {kind}")));
        }
        let parts: Vec<&str> = body.split('|').collect();
        let [iv, data, mac] = parts[..] else {
            return Err(Error::Malformed("cipher string"));
        };
        Ok(Self {
            iv: decoded(iv)?,
            data: decoded_vec(data)?,
            mac: decoded(mac)?,
        })
    }
}

impl fmt::Display for CipherString {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{AES_CBC_HMAC}.{}|{}|{}",
            STANDARD.encode(self.iv),
            STANDARD.encode(&self.data),
            STANDARD.encode(self.mac)
        )
    }
}

impl TryFrom<String> for CipherString {
    type Error = Error;

    fn try_from(text: String) -> Result<Self, Error> {
        text.parse()
    }
}

impl From<CipherString> for String {
    fn from(cipher: CipherString) -> Self {
        cipher.to_string()
    }
}
