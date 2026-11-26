use serde::Deserialize;
use serde_json::Value;

use super::{cipher_string::CipherString, error::Error, item::Sealed};

pub struct Reply {
    pub protected_key: CipherString,
    pub logins: Vec<Sealed>,
}

#[derive(Deserialize)]
struct Wire {
    profile: Profile,
    ciphers: Vec<Value>,
}

#[derive(Deserialize)]
struct Profile {
    key: CipherString,
}

fn personal_login(cipher: &Value) -> bool {
    cipher["organizationId"].is_null()
        && cipher["deletedDate"].is_null()
        && !cipher["login"].is_null()
}

impl Reply {
    pub fn parse(body: &str) -> Result<Self, Error> {
        let wire: Wire = serde_json::from_str(body)?;
        let logins = wire
            .ciphers
            .into_iter()
            .filter(personal_login)
            .map(serde_json::from_value)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            protected_key: wire.profile.key,
            logins,
        })
    }
}
