use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::{
    cipher_string::CipherString,
    error::Error,
    key::SymmetricKey,
    totp::Totp,
    uri::{LoginUri, Page, SealedUri},
};

pub type Unread = Map<String, Value>;

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
    #[serde(flatten)]
    pub unread: Unread,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SealedLogin {
    pub username: Option<CipherString>,
    pub password: Option<CipherString>,
    pub totp: Option<CipherString>,
    pub uris: Option<Vec<SealedUri>>,
    #[serde(flatten)]
    pub unread: Unread,
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
const PASSWORD_HISTORY_LIMIT: usize = 5;

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
                unread: Unread::new(),
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
                unread: Unread::new(),
            },
        })
    }
}

fn remembered(old: Option<&CipherString>, history: Option<&Value>, now: &str) -> Value {
    let earlier = history.and_then(Value::as_array).into_iter().flatten();
    let entry = old.map(|password| json!({ "password": password, "lastUsedDate": now }));
    let entries: Vec<Value> = entry
        .into_iter()
        .chain(earlier.cloned())
        .take(PASSWORD_HISTORY_LIMIT)
        .collect();
    Value::Array(entries)
}

impl Sealed {
    fn item_key(&self, user_key: &SymmetricKey) -> Result<Option<SymmetricKey>, Error> {
        self.key
            .as_ref()
            .map(|key| SymmetricKey::from_bytes(&key.decrypt(user_key)?))
            .transpose()
    }

    pub fn with_password(
        &self,
        password: &str,
        user_key: &SymmetricKey,
        now: &str,
    ) -> Result<Value, Error> {
        let item_key = self.item_key(user_key)?;
        let key = item_key.as_ref().unwrap_or(user_key);
        let mut changed = self.clone();
        let history = remembered(
            self.login.password.as_ref(),
            self.unread.get("passwordHistory"),
            now,
        );
        changed.login.password = Some(CipherString::encrypt(password.as_bytes(), key)?);
        changed
            .login
            .unread
            .insert("passwordRevisionDate".to_owned(), json!(now));
        changed.unread.insert("passwordHistory".to_owned(), history);
        let mut edit = serde_json::to_value(&changed)?;
        edit["type"] = json!(LOGIN_TYPE);
        edit["lastKnownRevisionDate"] =
            self.unread.get("revisionDate").cloned().unwrap_or_default();
        Ok(edit)
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
    pub fn authenticator(&self) -> Option<Totp> {
        Totp::parse(self.totp.as_deref()?).ok()
    }

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
