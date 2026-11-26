use std::path::PathBuf;

use serde::{Deserialize, Deserializer, de::Error};

use crate::{
    address::address,
    storage::{
        arrange::{Item, Spot},
        session::{FolderId, Link, SpaceId, TabId},
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PageAction {
    Ready,
    OpenUrl {
        url: String,
    },
    Navigate {
        text: String,
    },
    FocusTab {
        tab: TabId,
    },
    DeleteHistory {
        url: String,
    },
    ClearHistory,
    RemoveFavorite {
        space: SpaceId,
        url: String,
    },
    RemovePinned {
        tab: TabId,
    },
    EditFavorite {
        space: SpaceId,
        url: String,
        #[serde(deserialize_with = "typed_link")]
        edited: Link,
    },
    RenameTab {
        tab: TabId,
        title: String,
    },
    RenameFolder {
        folder: FolderId,
        name: String,
    },
    MoveFavorite {
        space: SpaceId,
        url: String,
        slot: usize,
    },
    MovePinned {
        item: Item,
        spot: Spot,
    },
    OpenDownload {
        path: PathBuf,
    },
    ShowDownloadInFolder {
        path: PathBuf,
    },
    PauseDownload {
        path: PathBuf,
    },
    ResumeDownload {
        path: PathBuf,
    },
    CancelDownload {
        path: PathBuf,
    },
    ClearDownloads,
    OpenSettingsSection {
        section: String,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Message {
    MagpiePage(PageAction),
}

fn typed_link<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Link, D::Error> {
    let typed = Link::deserialize(deserializer)?;
    let url = address(&typed.url).ok_or_else(|| D::Error::custom("not a web address"))?;
    Ok(Link {
        url: url.into(),
        title: typed.title,
    })
}

impl PageAction {
    pub fn parse(json: &str) -> Option<Self> {
        let Message::MagpiePage(action) = serde_json::from_str(json).ok()?;
        Some(action)
    }
}
