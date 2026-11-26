use std::collections::HashMap;

use serde::Deserialize;
use zeroize::Zeroizing;

use super::{device::DeviceId, error::Error};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Authenticator,
    Email,
    YubiKey,
    NewDevice,
}

pub struct Proof {
    pub method: Method,
    pub code: Zeroizing<String>,
}

pub struct PasswordGrant<'a> {
    pub email: &'a str,
    pub hash: &'a str,
    pub device: &'a DeviceId,
    pub proof: Option<&'a Proof>,
}

pub enum Grant<'a> {
    Password(PasswordGrant<'a>),
    Refresh(&'a str),
}

pub struct Tokens {
    pub access: Zeroizing<String>,
    pub expires_secs: u64,
    pub refresh: Option<Zeroizing<String>>,
}

pub enum Reply {
    Granted(Tokens),
    Challenged(Vec<Method>),
}

#[derive(Deserialize)]
struct Granted {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
}

#[derive(Deserialize)]
struct Refusal {
    #[serde(rename = "TwoFactorProviders2")]
    providers: Option<HashMap<String, serde_json::Value>>,
    error_description: Option<String>,
}

pub const CLIENT_ID: &str = "desktop";
pub const DEVICE_TYPE: &str = "6";
const DEVICE_NAME: &str = "Magpie";
const SCOPE: &str = "api offline_access";
const DO_NOT_REMEMBER: &str = "0";
const PROVIDERS: [(&str, Method); 3] = [
    ("0", Method::Authenticator),
    ("1", Method::Email),
    ("3", Method::YubiKey),
];
const NEW_DEVICE_REFUSAL: &str = "new device verification required";
const OK: u16 = 200;

fn provider(method: Method) -> Option<&'static str> {
    PROVIDERS
        .iter()
        .find(|(_, known)| *known == method)
        .map(|(provider, _)| *provider)
}

fn proof_fields(proof: &Proof) -> Vec<(&'static str, String)> {
    let code = proof.code.trim().to_owned();
    match provider(proof.method) {
        Some(provider) => vec![
            ("twoFactorToken", code),
            ("twoFactorProvider", provider.to_owned()),
            ("twoFactorRemember", DO_NOT_REMEMBER.to_owned()),
        ],
        None => vec![("newDeviceOtp", code)],
    }
}

fn challenge(providers: &HashMap<String, serde_json::Value>) -> Result<Reply, Error> {
    let methods: Vec<Method> = PROVIDERS
        .iter()
        .filter(|(provider, _)| providers.contains_key(*provider))
        .map(|(_, method)| *method)
        .collect();
    match methods.is_empty() {
        true => Err(Error::Unsupported(
            "this two-step login method; add an authenticator app, email or YubiKey".to_owned(),
        )),
        false => Ok(Reply::Challenged(methods)),
    }
}

fn granted(body: &str) -> Result<Reply, Error> {
    let granted: Granted = serde_json::from_str(body)?;
    Ok(Reply::Granted(Tokens {
        access: Zeroizing::new(granted.access_token),
        expires_secs: granted.expires_in,
        refresh: granted.refresh_token.map(Zeroizing::new),
    }))
}

impl Grant<'_> {
    pub fn email(&self) -> Option<&str> {
        match self {
            Self::Password(grant) => Some(grant.email),
            Self::Refresh(_) => None,
        }
    }

    pub fn form(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::Password(grant) => {
                let mut form = vec![
                    ("grant_type", "password".to_owned()),
                    ("username", grant.email.to_owned()),
                    ("password", grant.hash.to_owned()),
                    ("scope", SCOPE.to_owned()),
                    ("client_id", CLIENT_ID.to_owned()),
                    ("deviceType", DEVICE_TYPE.to_owned()),
                    ("deviceIdentifier", grant.device.as_str().to_owned()),
                    ("deviceName", DEVICE_NAME.to_owned()),
                ];
                form.extend(grant.proof.map(proof_fields).unwrap_or_default());
                form
            }
            Self::Refresh(token) => vec![
                ("grant_type", "refresh_token".to_owned()),
                ("client_id", CLIENT_ID.to_owned()),
                ("refresh_token", (*token).to_owned()),
            ],
        }
    }
}

impl Reply {
    pub fn parse(status: u16, body: &str) -> Result<Self, Error> {
        if status == OK {
            return granted(body);
        }
        let refusal: Option<Refusal> = serde_json::from_str(body).ok();
        let new_device = refusal
            .as_ref()
            .and_then(|refusal| refusal.error_description.as_deref())
            .is_some_and(|description| description.to_lowercase().contains(NEW_DEVICE_REFUSAL));
        match refusal.and_then(|refusal| refusal.providers) {
            Some(providers) => challenge(&providers),
            None if new_device => Ok(Self::Challenged(vec![Method::NewDevice])),
            None => Err(Error::rejected(status, body)),
        }
    }
}
