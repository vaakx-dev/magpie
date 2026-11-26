use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{config::Settings, storage::session::Session};

const FORMAT: u32 = 1;
const APP: &str = "magpie";
const BYTE_ORDER_MARK: char = '\u{feff}';

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Backup {
    app: String,
    format: u32,
    settings: String,
    sponsorblock: String,
    session: Session,
}

#[derive(Debug)]
pub enum Error {
    Unreadable(serde_json::Error),
    NotMagpie,
    NewerFormat(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unreadable(error) => {
                write!(formatter, "This file isn't a readable backup: {error}")
            }
            Self::NotMagpie => formatter.write_str("This file isn't a Magpie backup"),
            Self::NewerFormat(format) => write!(
                formatter,
                "This backup is from a newer Magpie (format {format}). Update Magpie to restore it"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl Backup {
    pub fn new(settings: &Settings, session: &Session) -> Self {
        Self {
            app: APP.to_owned(),
            format: FORMAT,
            settings: settings.text(),
            sponsorblock: settings.sponsorblock.text(),
            session: session.clone(),
        }
    }

    pub fn parse(json: &str) -> Result<Self, Error> {
        let text = json.strip_prefix(BYTE_ORDER_MARK).unwrap_or(json);
        let backup: Self = serde_json::from_str(text).map_err(Error::Unreadable)?;
        if backup.app != APP {
            return Err(Error::NotMagpie);
        }
        if backup.format > FORMAT {
            return Err(Error::NewerFormat(backup.format));
        }
        Ok(backup)
    }

    pub fn json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a backup serializes to JSON")
    }

    pub fn settings(&self) -> Settings {
        Settings::parse_with_sponsorblock(&self.settings, &self.sponsorblock)
    }

    pub fn into_session(self) -> Session {
        self.session
    }
}
