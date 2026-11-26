use serde::Serialize;
use serde_json::{Value, json};

use super::link::{Shortcut, host, shortcuts};
use crate::storage::session::{FolderId, Pinned, Session, Space, SpaceId, TabId};

#[derive(Serialize)]
struct SpaceEntry<'a> {
    id: SpaceId,
    name: &'a str,
    favorites: Vec<Shortcut<'a>>,
    pinned: Vec<PinnedEntry<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PinnedEntry<'a> {
    Tab {
        id: TabId,
        url: &'a str,
        title: &'a str,
        host: String,
    },
    Folder {
        id: FolderId,
        name: &'a str,
        collapsed: bool,
        items: Vec<PinnedEntry<'a>>,
    },
}

fn pinned(items: &[Pinned]) -> Vec<PinnedEntry<'_>> {
    items
        .iter()
        .map(|item| match item {
            Pinned::Tab(tab) => PinnedEntry::Tab {
                id: tab.id,
                url: tab.url(),
                title: tab.title(),
                host: host(tab.url()),
            },
            Pinned::Folder(folder) => PinnedEntry::Folder {
                id: folder.id,
                name: &folder.name,
                collapsed: folder.collapsed,
                items: pinned(&folder.items),
            },
        })
        .collect()
}

fn space_entry(space: &Space) -> SpaceEntry<'_> {
    SpaceEntry {
        id: space.id,
        name: &space.name,
        favorites: shortcuts(&space.favorites),
        pinned: pinned(&space.pinned),
    }
}

pub fn bookmarks_payload(session: &Session) -> Value {
    let spaces: Vec<SpaceEntry> = session.spaces.iter().map(space_entry).collect();
    json!({
        "active": session.active,
        "spaces": spaces,
    })
}
