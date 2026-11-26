use super::{Imported, Node, Report};
use crate::storage::Store;
use crate::storage::clock::Epoch;
use crate::storage::session::{Folder, FolderId, Pane, Pinned, Tab, TabId};

struct Pinner {
    next_tab: u64,
    next_folder: u64,
    now: Epoch,
    bookmarks: usize,
    folders: usize,
}

const NO_SPACE_WARNING: &str = "Bookmarks: no space to hold them";

impl Pinner {
    fn new(store: &Store, now: Epoch) -> Self {
        Self {
            next_tab: store.next_tab_id().0,
            next_folder: store.session.next_folder_id().0,
            now,
            bookmarks: 0,
            folders: 0,
        }
    }

    fn tab(&mut self, title: String, url: String) -> Pinned {
        let id = TabId(self.next_tab);
        self.next_tab += 1;
        self.bookmarks += 1;
        Pinned::Tab(Tab::new(id, Pane { url, title }, self.now))
    }

    fn folder(&mut self, name: String, children: Vec<Node>) -> Folder {
        let mut folder = Folder::new(FolderId(self.next_folder), name);
        self.next_folder += 1;
        folder.collapsed = true;
        folder.items = children.into_iter().map(|node| self.pin(node)).collect();
        folder
    }

    fn pin(&mut self, node: Node) -> Pinned {
        match node {
            Node::Bookmark { title, url } => self.tab(title, url),
            Node::Folder { name, children } => {
                self.folders += 1;
                Pinned::Folder(self.folder(name, children))
            }
        }
    }
}

impl Imported {
    pub fn apply(self, label: &str, store: &mut Store, now: Epoch) -> Report {
        let Imported {
            bookmarks,
            history,
            warnings,
        } = self;
        let mut report = Report {
            visits: history.len(),
            warnings,
            ..Report::default()
        };
        store.history_mut().merge(history, now);
        if bookmarks.is_empty() {
            return report;
        }
        let mut pinner = Pinner::new(store, now);
        let folder = pinner.folder(label.to_owned(), bookmarks);
        match store.session.active_space_mut() {
            Some(space) => {
                space.pinned.push(Pinned::Folder(folder));
                report.bookmarks = pinner.bookmarks;
                report.folders = pinner.folders;
            }
            None => report.warnings.push(NO_SPACE_WARNING.to_owned()),
        }
        report
    }
}
