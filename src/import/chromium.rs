use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::snapshot::Snapshot;
use super::visit::{self, Table};
use super::{Failure, Imported, Node, Scope, importable};
use crate::storage::clock::Epoch;
use crate::storage::history::Visit;

#[derive(Deserialize)]
struct BookmarkFile {
    roots: Roots,
}

#[derive(Deserialize)]
struct Roots {
    bookmark_bar: Option<Entry>,
    other: Option<Entry>,
    synced: Option<Entry>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Entry {
    Url { name: String, url: String },
    Folder { name: String, children: Vec<Entry> },
}

const BOOKMARKS_FILE: &str = "Bookmarks";
const HISTORY_FILE: &str = "History";
const MICROS_PER_SECOND: i64 = 1_000_000;
const SECONDS_FROM_1601_TO_1970: i64 = 11_644_473_600;
const URLS: Table = Table {
    query: "SELECT url, title, visit_count, last_visit_time FROM urls \
        WHERE hidden = 0 AND last_visit_time > 0 ORDER BY last_visit_time DESC LIMIT ?1",
    epoch: unix_time,
};

fn node(entry: Entry) -> Option<Node> {
    match entry {
        Entry::Url { name, url } if importable(&url) => Some(Node::Bookmark { title: name, url }),
        Entry::Url { .. } => None,
        Entry::Folder { name, children } => Some(Node::Folder {
            name,
            children: children.into_iter().filter_map(node).collect(),
        }),
    }
}

fn holds_bookmarks(node: &Node) -> bool {
    matches!(node, Node::Folder { children, .. } if !children.is_empty())
}

fn bookmarks(path: &Path) -> Result<Vec<Node>, Failure> {
    let file: BookmarkFile = serde_json::from_slice(&fs::read(path)?)?;
    let Roots {
        bookmark_bar,
        other,
        synced,
    } = file.roots;
    Ok([bookmark_bar, other, synced]
        .into_iter()
        .flatten()
        .filter_map(node)
        .filter(holds_bookmarks)
        .collect())
}

fn unix_time(webkit_micros: i64) -> Epoch {
    let seconds = webkit_micros / MICROS_PER_SECOND - SECONDS_FROM_1601_TO_1970;
    Epoch(u64::try_from(seconds).unwrap_or(0))
}

fn history(path: &Path) -> Result<Vec<Visit>, Failure> {
    let snapshot = Snapshot::open(path)?;
    visit::recent(snapshot.connection(), &URLS)
}

pub fn read(folder: &Path, scope: Scope) -> Imported {
    Imported::collect(
        scope,
        || bookmarks(&folder.join(BOOKMARKS_FILE)),
        || history(&folder.join(HISTORY_FILE)),
    )
}
