use std::path::PathBuf;

use serde::Deserialize;

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
    OpenPasswords,
    LockPasswords,
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
