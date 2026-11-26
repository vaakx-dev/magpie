use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, Row};

use super::snapshot::Snapshot;
use super::visit::{self, Table};
use super::{Failure, Imported, Node, Scope, importable};
use crate::storage::clock::Epoch;

enum Kind {
    Bookmark(String),
    Folder,
    Other,
}

struct Entry {
    id: i64,
    parent: i64,
    guid: String,
    title: String,
    kind: Kind,
}

struct Tree<'a> {
    children: HashMap<i64, Vec<&'a Entry>>,
}

const PLACES_FILE: &str = "places.sqlite";
const MICROS_PER_SECOND: i64 = 1_000_000;
const TYPE_BOOKMARK: i64 = 1;
const TYPE_FOLDER: i64 = 2;
const ROOTS: [(&str, &str); 4] = [
    ("toolbar_____", "Bookmarks Toolbar"),
    ("menu________", "Bookmarks Menu"),
    ("unfiled_____", "Other Bookmarks"),
    ("mobile______", "Mobile Bookmarks"),
];
const ENTRIES: &str = "SELECT b.id, b.parent, b.guid, b.title, b.type, p.url \
    FROM moz_bookmarks b LEFT JOIN moz_places p ON p.id = b.fk \
    ORDER BY b.parent, b.position";
const PLACES: Table = Table {
    query: "SELECT url, title, visit_count, last_visit_date FROM moz_places \
        WHERE hidden = 0 AND visit_count > 0 AND last_visit_date IS NOT NULL \
        ORDER BY last_visit_date DESC LIMIT ?1",
    epoch: unix_time,
};

fn unix_time(pr_time_micros: i64) -> Epoch {
    Epoch(u64::try_from(pr_time_micros / MICROS_PER_SECOND).unwrap_or(0))
}

fn kind(kind: i64, url: Option<String>) -> Kind {
    match (kind, url) {
        (TYPE_BOOKMARK, Some(url)) => Kind::Bookmark(url),
        (TYPE_FOLDER, _) => Kind::Folder,
        _ => Kind::Other,
    }
}

fn entry(row: &Row) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        parent: row.get(1)?,
        guid: row.get(2)?,
        title: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
        kind: kind(row.get(4)?, row.get(5)?),
    })
}

fn entries(connection: &Connection) -> Result<Vec<Entry>, Failure> {
    let mut statement = connection.prepare(ENTRIES)?;
    let rows = statement.query_map([], entry)?;
    Ok(rows.collect::<rusqlite::Result<Vec<Entry>>>()?)
}

impl<'a> Tree<'a> {
    fn new(entries: &'a [Entry]) -> Self {
        let mut children: HashMap<i64, Vec<&Entry>> = HashMap::new();
        for entry in entries {
            children.entry(entry.parent).or_default().push(entry);
        }
        Self { children }
    }

    fn nodes(&self, parent: i64) -> Vec<Node> {
        self.children
            .get(&parent)
            .into_iter()
            .flatten()
            .filter_map(|entry| self.node(entry))
            .collect()
    }

    fn node(&self, entry: &Entry) -> Option<Node> {
        match &entry.kind {
            Kind::Bookmark(url) if importable(url) => Some(Node::Bookmark {
                title: entry.title.clone(),
                url: url.clone(),
            }),
            Kind::Folder => Some(Node::Folder {
                name: entry.title.clone(),
                children: self.nodes(entry.id),
            }),
            Kind::Bookmark(_) | Kind::Other => None,
        }
    }
}

fn root(entries: &[Entry], tree: &Tree, guid: &str, name: &str) -> Option<Node> {
    let folder = entries.iter().find(|entry| entry.guid == guid)?;
    let children = tree.nodes(folder.id);
    let filled = !children.is_empty();
    filled.then(|| Node::Folder {
        name: name.to_owned(),
        children,
    })
}

fn bookmarks(connection: &Connection) -> Result<Vec<Node>, Failure> {
    let entries = entries(connection)?;
    let tree = Tree::new(&entries);
    Ok(ROOTS
        .iter()
        .filter_map(|(guid, name)| root(&entries, &tree, guid, name))
        .collect())
}

pub fn read(folder: &Path, scope: Scope) -> Imported {
    match Snapshot::open(&folder.join(PLACES_FILE)) {
        Ok(snapshot) => Imported::collect(
            scope,
            || bookmarks(snapshot.connection()),
            || visit::recent(snapshot.connection(), &PLACES),
        ),
        Err(error) => Imported::failed(PLACES_FILE, &*error),
    }
}
