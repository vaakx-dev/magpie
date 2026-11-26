use zeroize::Zeroizing;

use super::error::Error;

pub const LENGTH: usize = 20;
pub const CLASSES: [&str; 4] = [
    "ABCDEFGHJKLMNPQRSTUVWXYZ",
    "abcdefghijkmnopqrstuvwxyz",
    "23456789",
    "!@#$%^&*-_=+",
];

const BYTE_VALUES: usize = 256;

fn complete(password: &str) -> bool {
    CLASSES
        .iter()
        .all(|class| password.chars().any(|character| class.contains(character)))
}

fn random_text(alphabet: &[u8]) -> Result<Zeroizing<String>, Error> {
    let unbiased_limit = BYTE_VALUES - BYTE_VALUES % alphabet.len();
    let mut text = Zeroizing::new(String::with_capacity(LENGTH));
    let mut bytes = Zeroizing::new([0; LENGTH * 2]);
    while text.len() < LENGTH {
        getrandom::fill(bytes.as_mut()).map_err(Error::Random)?;
        let characters = bytes
            .iter()
            .map(|byte| usize::from(*byte))
            .filter(|value| *value < unbiased_limit)
            .map(|value| char::from(alphabet[value % alphabet.len()]))
            .take(LENGTH - text.len());
        text.extend(characters);
    }
    Ok(text)
}

pub fn generate() -> Result<Zeroizing<String>, Error> {
    let alphabet = CLASSES.concat();
    loop {
        let candidate = random_text(alphabet.as_bytes())?;
        if complete(&candidate) {
            return Ok(candidate);
        }
    }
}
