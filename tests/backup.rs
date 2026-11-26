use std::time::Duration;

use magpie::backup::{Backup, Error};
use magpie::config::{Settings, Switch, Theme};
use magpie::storage::Store;
use magpie::storage::clock::Epoch;
use magpie::storage::session::{
    Folder, FolderId, Link, Pane, Pinned, Session, Space, SpaceId, Tab, TabId,
};
use magpie_engine::sponsorblock::Category;

const START: Epoch = Epoch(1_800_000_000);
const IDLE: Duration = Duration::from_secs(12 * 60 * 60);

fn tab(id: u64, url: &str) -> Tab {
    let pane = Pane {
        url: url.to_owned(),
        title: url.to_owned(),
    };
    Tab::new(TabId(id), pane, START)
}

fn session() -> Session {
    let mut space = Space::new(SpaceId(3), "Reading", "Reading");
    space.favorites.push(Link {
        url: "https://news.example.com/".to_owned(),
        title: "News".to_owned(),
    });
    let mut folder = Folder::new(FolderId(7), "Recipes");
    folder
        .items
        .push(Pinned::Tab(tab(2, "https://soup.example.com/")));
    space.pinned.push(Pinned::Folder(folder));
    space.today.push(tab(1, "https://today.example.com/"));
    space.active = Some(TabId(1));
    Session {
        spaces: vec![space],
        active: Some(SpaceId(3)),
    }
}

fn settings() -> Settings {
    let mut settings = Settings {
        theme: Theme::Dark,
        background_play: Switch::On,
        ..Settings::default()
    };
    settings.general.home_page = "https://home.example.com/".to_owned();
    settings.sponsorblock.enabled = false;
    settings.sponsorblock.categories.remove(Category::Sponsor);
    settings
        .sponsorblock
        .channel_allowlist
        .push("Some Creator".to_owned());
    settings
}

#[test]
fn a_backup_brings_back_the_same_spaces_and_settings() {
    let json = Backup::new(&settings(), &session()).json();

    let restored = Backup::parse(&json).unwrap();
    let saved_by_notepad = format!("\u{feff}{json}");
    assert_eq!(Backup::parse(&saved_by_notepad).unwrap(), restored);

    assert_eq!(restored.settings(), settings());
    assert_eq!(restored.into_session(), session());
}

#[test]
fn files_that_are_not_magpie_backups_are_refused() {
    let other_app = r#"{"app":"other","format":1,"settings":"","sponsorblock":"","session":{"spaces":[],"active":null}}"#;
    let from_the_future = r#"{"app":"magpie","format":99,"settings":"","sponsorblock":"","session":{"spaces":[],"active":null}}"#;

    assert!(matches!(Backup::parse(other_app), Err(Error::NotMagpie)));
    assert!(matches!(
        Backup::parse(from_the_future),
        Err(Error::NewerFormat(99))
    ));
    assert!(matches!(
        Backup::parse("[1, 2, 3]"),
        Err(Error::Unreadable(_))
    ));
}

#[test]
fn restored_tabs_never_share_an_id_with_an_archived_tab() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    let mut current = Space::new(SpaceId(1), "Work", "Work");
    current.today.push(tab(1, "https://stale.example.com/"));
    store.session.spaces.push(current);
    store.sweep(START.after(IDLE * 2), IDLE);
    let archived: Vec<TabId> = store.archive.list().map(|entry| entry.tab.id).collect();

    store.replace_session(session());

    let space = &store.session.spaces[0];
    let ids: Vec<TabId> = space.tabs().map(|tab| tab.id).collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.iter().all(|id| !archived.contains(id)));
    assert_eq!(
        space.active_tab().unwrap().url(),
        "https://today.example.com/"
    );
    assert_eq!(space.pinned_tabs()[0].url(), "https://soup.example.com/");
}
