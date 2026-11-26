use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::{
    cipher_string::CipherString,
    error::Error,
    key::SymmetricKey,
    uri::{LoginUri, Page, SealedUri},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Zeroize)]
#[serde(transparent)]
pub struct CipherId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sealed {
    pub id: CipherId,
    pub key: Option<CipherString>,
    pub name: CipherString,
    pub notes: Option<CipherString>,
    #[serde(default)]
    pub favorite: bool,
    pub login: SealedLogin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SealedLogin {
    pub username: Option<CipherString>,
    pub password: Option<CipherString>,
    pub totp: Option<CipherString>,
    pub uris: Option<Vec<SealedUri>>,
}

pub struct NewLogin<'a> {
    pub name: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub uri: &'a str,
}

#[derive(Serialize)]
pub struct Draft {
    #[serde(rename = "type")]
    kind: u8,
    name: CipherString,
    notes: Option<CipherString>,
    favorite: bool,
    login: SealedLogin,
}

#[derive(Clone, Debug, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct Login {
    pub id: CipherId,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub totp: Option<String>,
    pub notes: Option<String>,
    pub uris: Vec<LoginUri>,
    pub favorite: bool,
}

const LOGIN_TYPE: u8 = 1;

fn sealed_text(text: &str, key: &SymmetricKey) -> Result<Option<CipherString>, Error> {
    (!text.is_empty())
        .then(|| CipherString::encrypt(text.as_bytes(), key))
        .transpose()
}

fn text(field: &Option<CipherString>, key: &SymmetricKey) -> Result<Option<String>, Error> {
    field
        .as_ref()
        .map(|value| value.decrypt_text(key))
        .transpose()
}

impl NewLogin<'_> {
    pub fn seal(&self, key: &SymmetricKey) -> Result<Draft, Error> {
        let uris = sealed_text(self.uri.trim(), key)?.map(|uri| {
            vec![SealedUri {
                uri: Some(uri),
                rule: None,
            }]
        });
        Ok(Draft {
            kind: LOGIN_TYPE,
            name: CipherString::encrypt(self.name.trim().as_bytes(), key)?,
            notes: None,
            favorite: false,
            login: SealedLogin {
                username: sealed_text(self.username.trim(), key)?,
                password: sealed_text(self.password, key)?,
                totp: None,
                uris,
            },
        })
    }
}

impl Sealed {
    fn item_key(&self, user_key: &SymmetricKey) -> Result<Option<SymmetricKey>, Error> {
        self.key
            .as_ref()
            .map(|key| SymmetricKey::from_bytes(&key.decrypt(user_key)?))
            .transpose()
    }

    pub fn open(&self, user_key: &SymmetricKey) -> Result<Login, Error> {
        let item_key = self.item_key(user_key)?;
        let key = item_key.as_ref().unwrap_or(user_key);
        let uris = self.login.uris.iter().flatten().map(|uri| uri.open(key));
        Ok(Login {
            id: self.id.clone(),
            name: self.name.decrypt_text(key)?,
            username: text(&self.login.username, key)?,
            password: text(&self.login.password, key)?,
            totp: text(&self.login.totp, key)?,
            notes: text(&self.notes, key)?,
            uris: uris
                .filter_map(Result::transpose)
                .collect::<Result<_, _>>()?,
            favorite: self.favorite,
        })
    }
}

impl Login {
    pub fn matches(&self, page: &Page) -> bool {
        self.uris.iter().any(|uri| uri.matches(page))
    }

    pub fn contains(&self, lowercase_query: &str) -> bool {
        let found = |text: &str| text.to_lowercase().contains(lowercase_query);
        found(&self.name)
            || self.username.as_deref().is_some_and(found)
            || self.uris.iter().any(|uri| found(&uri.uri))
    }
}
