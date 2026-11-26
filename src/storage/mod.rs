pub mod archive;
pub mod arrange;
pub mod clock;
pub mod closed;
pub mod favicon;
pub mod history;
pub mod permissions;
pub mod profile;
pub mod query;
pub mod recovery;
pub mod search;
pub mod session;
pub mod thumbnail;

use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use crate::role::Role;
use archive::Archive;
use clock::Epoch;
use history::History;
use magpie_engine::file;
use profile::Profiles;
use search::{OpenTab, SearchResult, Sources};
use session::{Session, TabId};

const APP_FOLDER: &str = "Magpie";
const SESSION_FILE: &str = "session.json";
const SESSION_BACKUP_FILE: &str = "session.backup.json";
const HISTORY_FILE: &str = "history.json";
const ARCHIVE_FILE: &str = "archive.json";
const PROFILES_FILE: &str = "profiles.json";
const PAUSED_SITES_FILE: &str = "paused-sites.json";
const FAVICON_FOLDER: &str = "favicons";
const THUMBNAIL_FOLDER: &str = "thumbs";

pub struct Store {
    folder: PathBuf,
    role: Role,
    pub session: Session,
    history: Rc<RefCell<History>>,
    profiles: Rc<RefCell<Profiles>>,
    pub archive: Archive,
}

pub fn default_folder() -> io::Result<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA is not set"))?;
    Ok(PathBuf::from(local).join(APP_FOLDER))
}

impl Store {
    pub fn open(folder: &Path) -> io::Result<Self> {
        std::fs::create_dir_all(folder)?;
        let profiles: Profiles = file::load(&folder.join(PROFILES_FILE))?;
        let mut session: Session = recovery::load(
            &folder.join(SESSION_FILE),
            &folder.join(SESSION_BACKUP_FILE),
        )?;
        session.keep_profiles(&profiles);
        Ok(Self {
            folder: folder.to_owned(),
            role: Role::Primary,
            session,
            history: Rc::new(RefCell::new(file::load(&folder.join(HISTORY_FILE))?)),
            profiles: Rc::new(RefCell::new(profiles)),
            archive: file::load(&folder.join(ARCHIVE_FILE))?,
        })
    }

    fn sibling(&self, role: Role) -> Self {
        Self {
            folder: self.folder.clone(),
            role,
            session: Session::default(),
            history: Rc::clone(&self.history),
            profiles: Rc::clone(&self.profiles),
            archive: Archive::default(),
        }
    }

    pub fn new_window(&self) -> Self {
        self.sibling(Role::Secondary)
    }

    pub fn new_private_window(&self) -> Self {
        self.sibling(Role::Private)
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn open_default() -> io::Result<Self> {
        Self::open(&default_folder()?)
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    pub fn history(&self) -> Ref<'_, History> {
        self.history.borrow()
    }

    pub fn history_mut(&self) -> RefMut<'_, History> {
        self.history.borrow_mut()
    }

    pub fn favicons(&self) -> favicon::Cache {
        favicon::Cache::new(self.folder.join(FAVICON_FOLDER), self.role)
    }

    pub fn thumbnails(&self) -> thumbnail::Cache {
        thumbnail::Cache::new(self.folder.join(THUMBNAIL_FOLDER), self.role)
    }

    pub fn profiles(&self) -> Ref<'_, Profiles> {
        self.profiles.borrow()
    }

    pub fn profiles_mut(&self) -> RefMut<'_, Profiles> {
        self.profiles.borrow_mut()
    }

    pub fn save_profiles(&self) -> io::Result<()> {
        file::save(&self.folder.join(PROFILES_FILE), &*self.profiles())
    }

    pub fn record_visit(&self, url: &str, title: &str, now: Epoch) -> io::Result<()> {
        if self.role.private() {
            return Ok(());
        }
        self.history_mut().record(url, title, now);
        self.save_history()
    }

    pub fn forget_visit(&self, url: &str) -> io::Result<()> {
        self.history_mut().forget(url);
        self.save_history()
    }

    pub fn clear_history(&self) -> io::Result<()> {
        self.history_mut().clear();
        self.save_history()
    }

    pub fn save_session(&self) -> io::Result<()> {
        if !self.role.keeps_session() {
            return Ok(());
        }
        file::save(&self.folder.join(SESSION_FILE), &self.session)
    }

    pub fn save_history(&self) -> io::Result<()> {
        file::save(&self.folder.join(HISTORY_FILE), &*self.history())
    }

    pub fn save_archive(&self) -> io::Result<()> {
        if !self.role.keeps_session() {
            return Ok(());
        }
        file::save(&self.folder.join(ARCHIVE_FILE), &self.archive)
    }

    pub fn save(&self) -> io::Result<()> {
        self.save_session()?;
        self.save_history()?;
        self.save_archive()
    }

    pub fn paused_sites(&self) -> io::Result<BTreeSet<String>> {
        file::load(&self.folder.join(PAUSED_SITES_FILE))
    }

    pub fn save_paused_sites(&self, sites: &BTreeSet<String>) -> io::Result<()> {
        if self.role.private() {
            return Ok(());
        }
        file::save(&self.folder.join(PAUSED_SITES_FILE), sites)
    }

    pub fn replace_session(&mut self, mut session: Session) {
        session.shift_tab_ids(self.next_tab_id().0);
        session.keep_profiles(&self.profiles());
        self.session = session;
    }

    pub fn next_tab_id(&self) -> TabId {
        let open = self.session.spaces.iter().flat_map(|space| space.tabs());
        let archived = self.archive.list().map(|entry| &entry.tab);
        let largest = open.chain(archived).map(|tab| tab.id.0).max();
        TabId(largest.map_or(1, |id| id + 1))
    }

    pub fn sweep(&mut self, now: Epoch, idle: Duration) -> usize {
        self.archive.sweep(&mut self.session, now, idle)
    }

    pub fn archive_today(&mut self, now: Epoch) -> usize {
        for space in &mut self.session.spaces {
            space.active = None;
        }
        self.sweep(now, Duration::ZERO)
    }

    pub fn search(&self, query: &str, open_tabs: &[OpenTab], now: Epoch) -> Vec<SearchResult> {
        let history = self.history();
        let sources = Sources {
            open_tabs,
            session: &self.session,
            history: &history,
            archive: &self.archive,
            now,
        };
        search::search(query, &sources)
    }
}
