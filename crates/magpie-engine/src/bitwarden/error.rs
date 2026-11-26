use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Address(String),
    Network(ureq::Error),
    Rejected { status: u16, message: String },
    Response(serde_json::Error),
    Malformed(&'static str),
    Unsupported(String),
    WeakKdf,
    Tampered,
    WrongPassword,
    Expired,
    Locked,
    SignedOut,
    Random(getrandom::Error),
    Storage(io::Error),
}

const MESSAGE_PATHS: [&[&str]; 5] = [
    &["ErrorModel", "Message"],
    &["message"],
    &["Message"],
    &["error_description"],
    &["error"],
];

fn message<'a>(reply: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    path.iter()
        .try_fold(reply, |value, key| value.get(key))?
        .as_str()
}

impl Error {
    pub fn rejected(status: u16, body: &str) -> Self {
        let reply: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
        let message = MESSAGE_PATHS
            .iter()
            .find_map(|path| message(&reply, path))
            .unwrap_or(body);
        Self::Rejected {
            status,
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Address(address) => write!(formatter, "Not a server address: {address}"),
            Self::Network(error) => write!(formatter, "Bitwarden request failed: {error}"),
            Self::Rejected { status, message } => {
                write!(formatter, "Bitwarden refused ({status}): {message}")
            }
            Self::Response(error) => write!(formatter, "Bitwarden reply unreadable: {error}"),
            Self::Malformed(what) => write!(formatter, "Malformed {what}"),
            Self::Unsupported(what) => write!(formatter, "Not supported: {what}"),
            Self::WeakKdf => write!(formatter, "The server asked for unsafe key settings"),
            Self::Tampered => write!(formatter, "Encrypted data failed its integrity check"),
            Self::WrongPassword => write!(formatter, "Wrong master password"),
            Self::Expired => write!(formatter, "The session expired; sign in again"),
            Self::Locked => write!(formatter, "The vault is locked"),
            Self::SignedOut => write!(formatter, "Not signed in to Bitwarden"),
            Self::Random(error) => write!(formatter, "No secure randomness: {error}"),
            Self::Storage(error) => write!(formatter, "Vault file unavailable: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Storage(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Response(error)
    }
}
