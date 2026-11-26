use std::{fmt, path::PathBuf};

use serde::{Deserialize, Deserializer};
use zeroize::Zeroizing;

use magpie_engine::bitwarden::CipherId;

use crate::{
    import::Scope,
    storage::{
        permissions::{Kind, Origin, State},
        profile::ProfileId,
        session::SpaceId,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Import {
    pub profile: PathBuf,
    bookmarks: bool,
    history: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(pub Zeroizing<String>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Host {
    UnitedStates,
    Europe,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Part {
    Username,
    Password,
    Code,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PasswordAction {
    SignIn {
        host: Host,
        address: String,
        email: String,
        password: Secret,
    },
    AnswerChallenge {
        method: String,
        code: Secret,
    },
    SendEmailCode,
    CancelSignIn,
    Unlock {
        password: Secret,
    },
    UnlockWithHello,
    Lock,
    Sync,
    SignOut,
    SetHello {
        on: bool,
    },
    Search {
        query: String,
    },
    Copy {
        id: CipherId,
        part: Part,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Ready,
    SetSetting {
        key: String,
        value: String,
    },
    SetSponsorblock {
        key: String,
        value: String,
    },
    UnpauseChannel {
        channel: String,
    },
    UnpauseSite {
        site: String,
    },
    ResetDarkSite {
        site: String,
    },
    ClearZoom {
        site: String,
    },
    SetPermission {
        origin: Origin,
        kind: Kind,
        state: State,
    },
    ChooseDownloadFolder,
    ClearBrowsingData,
    OpenDataFolder,
    CheckForUpdates,
    InstallUpdate,
    RunImport(Import),
    ExportBackup,
    ImportBackup,
    Passwords {
        action: PasswordAction,
    },
    CreateProfile {
        name: String,
    },
    RenameProfile {
        profile: ProfileId,
        name: String,
    },
    DeleteProfile {
        profile: ProfileId,
    },
    AssignProfile {
        space: SpaceId,
        profile: ProfileId,
    },
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Secret")
    }
}

impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|text| Self(Zeroizing::new(text)))
    }
}

impl Import {
    pub fn scope(&self) -> Option<Scope> {
        match (self.bookmarks, self.history) {
            (true, true) => Some(Scope::Everything),
            (true, false) => Some(Scope::Bookmarks),
            (false, true) => Some(Scope::History),
            (false, false) => None,
        }
    }
}

impl Message {
    pub fn parse(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}
