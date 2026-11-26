use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use url::Url;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Algorithm {
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Totp {
    secret: Vec<u8>,
    #[zeroize(skip)]
    algorithm: Algorithm,
    digits: u32,
    period_secs: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code {
    pub digits: String,
    pub remaining_secs: u64,
}

const OTPAUTH_PREFIX: &str = "otpauth://";
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const BASE32_IGNORED: &[u8] = b" -=";
const BASE32_BITS: u32 = 5;
const DEFAULT_DIGITS: u32 = 6;
const DIGITS: std::ops::RangeInclusive<u32> = 1..=10;
const DEFAULT_PERIOD_SECS: u64 = 30;
const TRUNCATION_MASK: u32 = 0x7fff_ffff;
const OFFSET_MASK: u8 = 0x0f;

fn base32_value(symbol: u8) -> Result<u32, Error> {
    BASE32_ALPHABET
        .iter()
        .position(|letter| *letter == symbol.to_ascii_uppercase())
        .map(|value| value as u32)
        .ok_or(Error::Malformed("TOTP secret"))
}

fn base32(text: &str) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    let mut buffer: u32 = 0;
    let mut buffered_bits = 0;
    for symbol in text.bytes().filter(|byte| !BASE32_IGNORED.contains(byte)) {
        buffer = (buffer << BASE32_BITS) | base32_value(symbol)?;
        buffered_bits += BASE32_BITS;
        if buffered_bits >= u8::BITS {
            buffered_bits -= u8::BITS;
            bytes.push((buffer >> buffered_bits) as u8);
            buffer &= (1 << buffered_bits) - 1;
        }
    }
    match bytes.is_empty() {
        true => Err(Error::Malformed("TOTP secret")),
        false => Ok(bytes),
    }
}

fn algorithm(name: &str) -> Result<Algorithm, Error> {
    match name.to_ascii_uppercase().as_str() {
        "SHA1" => Ok(Algorithm::Sha1),
        "SHA256" => Ok(Algorithm::Sha256),
        "SHA512" => Ok(Algorithm::Sha512),
        _ => Err(Error::Unsupported(format!("TOTP algorithm {name}"))),
    }
}

fn sign<M: Mac + hmac::digest::KeyInit>(key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut mac = <M as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}

fn truncate(digest: &[u8]) -> u32 {
    let offset = usize::from(digest[digest.len() - 1] & OFFSET_MASK);
    let window: [u8; 4] = digest[offset..offset + 4]
        .try_into()
        .expect("HMAC digests are at least 20 bytes");
    u32::from_be_bytes(window) & TRUNCATION_MASK
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

impl Totp {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let text = text.trim();
        let is_uri = text
            .get(..OTPAUTH_PREFIX.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(OTPAUTH_PREFIX));
        match is_uri {
            true => Self::from_uri(text),
            false => Self::from_secret(text),
        }
    }

    fn from_secret(secret: &str) -> Result<Self, Error> {
        Ok(Self {
            secret: base32(secret)?,
            algorithm: Algorithm::Sha1,
            digits: DEFAULT_DIGITS,
            period_secs: DEFAULT_PERIOD_SECS,
        })
    }

    fn from_uri(text: &str) -> Result<Self, Error> {
        let uri = Url::parse(text).map_err(|_| Error::Malformed("otpauth URI"))?;
        let parameter = |name: &str| {
            uri.query_pairs()
                .find(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.into_owned())
        };
        let secret = parameter("secret").ok_or(Error::Malformed("otpauth URI"))?;
        let digits = parameter("digits").map_or(Ok(DEFAULT_DIGITS), |digits| digits.parse());
        let period_secs =
            parameter("period").map_or(Ok(DEFAULT_PERIOD_SECS), |period| period.parse());
        let (Ok(digits), Ok(period_secs)) = (digits, period_secs) else {
            return Err(Error::Malformed("otpauth URI"));
        };
        if !DIGITS.contains(&digits) || period_secs == 0 {
            return Err(Error::Malformed("otpauth URI"));
        }
        Ok(Self {
            secret: base32(&secret)?,
            algorithm: parameter("algorithm")
                .map_or(Ok(Algorithm::Sha1), |name| algorithm(&name))?,
            digits,
            period_secs,
        })
    }

    fn digest(&self, counter: u64) -> Vec<u8> {
        let message = counter.to_be_bytes();
        match self.algorithm {
            Algorithm::Sha1 => sign::<Hmac<Sha1>>(&self.secret, &message),
            Algorithm::Sha256 => sign::<Hmac<Sha256>>(&self.secret, &message),
            Algorithm::Sha512 => sign::<Hmac<Sha512>>(&self.secret, &message),
        }
    }

    pub fn code_at(&self, unix_secs: u64) -> Code {
        let value = u64::from(truncate(&self.digest(unix_secs / self.period_secs)));
        let digits = self.digits as usize;
        Code {
            digits: format!("{:0digits$}", value % 10u64.pow(self.digits)),
            remaining_secs: self.period_secs - unix_secs % self.period_secs,
        }
    }

    pub fn code(&self) -> Code {
        self.code_at(unix_secs())
    }
}
