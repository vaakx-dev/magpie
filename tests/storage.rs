use std::fs;
use std::time::Duration;

use magpie::storage::Store;
use magpie::storage::clock::Epoch;
use magpie::storage::history::History;
use magpie::storage::search::{OpenTab, Source};
use magpie::storage::session::{Folder, FolderId, Link, Pane, Pinned, Space, SpaceId, Tab, TabId};
use magpie_engine::file;

const HOUR: Duration = Duration::from_secs(60 * 60);
const DAY: Duration = Duration::from_secs(24 * 60 * 60);
const IDLE: Duration = Duration::from_secs(12 * 60 * 60);
const START: Epoch = Epoch(1_800_000_000);

fn pane(url: &str, title: &str) -> Pane {
    Pane {
        url: url.to_owned(),
        title: title.to_owned(),
    }
}

fn tab(id: u64, url: &str, title: &str, last_active: Epoch) -> Tab {
    Tab::new(TabId(id), pane(url, title), last_active)
}

fn work_space() -> Space {
    let mut space = Space::new(SpaceId(1), "Work", "Work");
    space.favorites.push(Link {
        url: "https://mail.example.com/".to_owned(),
        title: "Mail".to_owned(),
    });
    let mut docs = Folder::new(FolderId(10), "Docs");
    docs.collapsed = true;
    let mut specs = Folder::new(FolderId(11), "Specs");
    specs.items.push(Pinned::Tab(tab(
        3,
        "https://specs.example.com/",
        "Specs",
        START,
    )));
    docs.items.push(Pinned::Tab(tab(
        2,
        "https://docs.example.com/",
        "Docs home",
        START,
    )));
    docs.items.push(Pinned::Folder(specs));
    space.pinned.push(Pinned::Tab(tab(
        1,
        "https://tracker.example.com/",
        "Tracker",
        START,
    )));
    space.pinned.push(Pinned::Folder(docs));
    let mut split = tab(4, "https://left.example.com/", "Left", START);
    split.split(pane("https://right.example.com/", "Right"));
    space.today.push(split);
    space.active = Some(TabId(4));
    space
}

#[test]
fn session_survives_save_and_reopen() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    store.session.spaces.push(work_space());
    store.session.active = Some(SpaceId(1));
    store.save().unwrap();

    let reopened = Store::open(folder.path()).unwrap();

    assert_eq!(reopened.session, store.session);
    let space = &reopened.session.spaces[0];
    let pinned_urls: Vec<&str> = space.pinned_tabs().iter().map(|tab| tab.url()).collect();
    assert_eq!(
        pinned_urls,
        [
            "https://tracker.example.com/",
            "https://docs.example.com/",
            "https://specs.example.com/"
        ]
    );
    assert_eq!(space.today[0].url(), "https://right.example.com/");
    assert_eq!(space.today[0].panes().len(), 2);
    let leftovers: Vec<_> = fs::read_dir(folder.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| !name.ends_with(".json"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temporary files left behind: {leftovers:?}"
    );
}

#[test]
fn pinned_folders_can_be_collapsed_and_tabs_found_inside_them() {
    let mut space = work_space();

    space.folder_mut(FolderId(11)).unwrap().collapsed = true;
    let activated = space.activate(TabId(3), START.after(HOUR));

    assert!(activated);
    assert_eq!(space.active, Some(TabId(3)));
    assert_eq!(
        space.tab_mut(TabId(3)).unwrap().last_active,
        START.after(HOUR)
    );
    assert!(space.folder_mut(FolderId(11)).unwrap().collapsed);
    assert!(!space.activate(TabId(99), START));
}

#[test]
fn corrupted_session_falls_back_to_empty_and_keeps_the_bad_file() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    store.session.spaces.push(work_space());
    store
        .history_mut()
        .record("https://example.com/", "Example", START);
    store.save().unwrap();
    let session_path = folder.path().join("session.json");
    fs::write(&session_path, b"{\"spaces\": [{\"id\": 1, \"na").unwrap();

    let mut reopened = Store::open(folder.path()).unwrap();

    assert!(reopened.session.spaces.is_empty());
    assert_eq!(*reopened.history(), *store.history());
    assert_eq!(
        fs::read(file::corrupt_copy(&session_path)).unwrap(),
        b"{\"spaces\": [{\"id\": 1, \"na"
    );
    reopened.session.spaces.push(work_space());
    reopened.save_session().unwrap();
    assert_eq!(Store::open(folder.path()).unwrap().session.spaces.len(), 1);
}

#[test]
fn tab_with_out_of_range_focus_is_treated_as_corrupt() {
    let folder = tempfile::tempdir().unwrap();
    let session = r#"{"spaces":[{"id":1,"name":"Work","theme":"Work","accent":0,"favorites":[],"pinned":[],
        "today":[{"id":4,"panes":[{"url":"https://a.example/","title":"A"}],"focused":3,"last_active":0}],
        "active":null}],"active":null}"#;
    fs::write(folder.path().join("session.json"), session).unwrap();

    let store = Store::open(folder.path()).unwrap();

    assert!(store.session.spaces.is_empty());
}

#[test]
fn idle_today_tabs_are_archived_after_twelve_hours_and_can_be_restored() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    let mut space = work_space();
    space
        .today
        .push(tab(5, "https://stale.example.com/", "Stale recipe", START));
    space.today.push(tab(
        6,
        "https://fresh.example.com/",
        "Fresh",
        START.after(2 * HOUR),
    ));
    store.session.spaces.push(space);
    let now = START.after(13 * HOUR);

    let archived = store.sweep(now, IDLE);

    assert_eq!(archived, 1);
    let today: Vec<TabId> = store.session.spaces[0]
        .today
        .iter()
        .map(|tab| tab.id)
        .collect();
    assert_eq!(today, [TabId(4), TabId(6)]);
    assert_eq!(store.session.spaces[0].pinned_tabs().len(), 3);
    store.save().unwrap();

    let mut reopened = Store::open(folder.path()).unwrap();
    let found: Vec<TabId> = reopened
        .archive
        .search("recipe")
        .iter()
        .map(|entry| entry.tab.id)
        .collect();
    assert_eq!(found, [TabId(5)]);
    assert!(reopened.archive.search("fresh").is_empty());

    let later = now.after(HOUR);
    let restored_into = reopened
        .archive
        .restore(TabId(5), &mut reopened.session, later);

    assert_eq!(restored_into, Some(SpaceId(1)));
    assert_eq!(reopened.archive.list().count(), 0);
    let space = &reopened.session.spaces[0];
    assert_eq!(space.active, Some(TabId(5)));
    assert_eq!(space.today.first().unwrap().last_active, later);
    reopened.sweep(later, IDLE);
    assert!(
        reopened.session.spaces[0]
            .today
            .iter()
            .any(|tab| tab.id == TabId(5))
    );
}

#[test]
fn tabs_leave_nested_pinned_folders_and_new_ids_skip_archived_tabs() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    let mut space = work_space();
    space
        .today
        .push(tab(9, "https://stale.example.com/", "Stale", START));
    store.session.spaces.push(space);
    store.sweep(START.after(DAY), IDLE);

    let space = &mut store.session.spaces[0];
    let moved = space.remove(TabId(3)).unwrap();

    assert_eq!(moved.url(), "https://specs.example.com/");
    assert!(space.tab(TabId(3)).is_none());
    assert_eq!(space.pinned_tabs().len(), 2);
    assert_eq!(store.next_tab_id(), TabId(10));
}

#[test]
fn archive_threshold_is_configurable() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    let mut space = work_space();
    space
        .today
        .push(tab(5, "https://idle.example.com/", "Idle", START));
    store.session.spaces.push(space);

    assert_eq!(store.sweep(START.after(13 * HOUR), DAY), 0);
    assert_eq!(store.sweep(START.after(DAY), DAY), 1);
}

#[test]
fn restoring_a_tab_whose_space_was_deleted_uses_the_active_space() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    let mut gone = Space::new(SpaceId(2), "Gone", "Lab");
    gone.today
        .push(tab(7, "https://lost.example.com/", "Lost", START));
    store.session.spaces.push(work_space());
    store.session.spaces.push(gone);
    store.session.active = Some(SpaceId(1));
    store.sweep(START.after(DAY), IDLE);
    store.session.spaces.retain(|space| space.id != SpaceId(2));

    let restored_into = store
        .archive
        .restore(TabId(7), &mut store.session, START.after(DAY));

    assert_eq!(restored_into, Some(SpaceId(1)));
    assert!(
        store.session.spaces[0]
            .today
            .iter()
            .any(|tab| tab.id == TabId(7))
    );
}

#[test]
fn visits_are_saved_in_the_background_and_flushed_when_the_store_closes() {
    let folder = tempfile::tempdir().unwrap();
    let history_path = folder.path().join("history.json");
    let store = Store::open(folder.path()).unwrap();

    store.record_visit("https://news.example.com/", "News", START);
    store.record_visit("https://mail.example.com/", "Mail", START);

    assert!(!history_path.exists());
    drop(store);
    let reopened = Store::open(folder.path()).unwrap();
    assert_eq!(reopened.history().visits().count(), 2);

    reopened.forget_visit("https://news.example.com/");
    reopened.save().unwrap();
    let saved: History = file::load(&history_path).unwrap();
    assert_eq!(saved, *reopened.history());
}

#[test]
fn history_counts_repeat_visits_and_ranks_by_frecency() {
    let store = Store::open(tempfile::tempdir().unwrap().path()).unwrap();
    let now = START.after(100 * DAY);
    store
        .history_mut()
        .record("https://old.example.com/", "Example archive", START);
    store
        .history_mut()
        .record("https://new.example.com/", "Example news", now);
    store
        .history_mut()
        .record("https://new.example.com/", "", now);
    store
        .history_mut()
        .record("https://new.example.com/", "Example news today", now);

    let history = store.history();
    let visit = history.visit("https://new.example.com/").unwrap();
    assert_eq!(visit.count, 3);
    assert_eq!(visit.title, "Example news today");
    assert_eq!(history.visits().count(), 2);

    let results = store.search("example", &[], now);
    let urls: Vec<&str> = results.iter().map(|result| result.url.as_str()).collect();
    assert_eq!(
        urls,
        ["https://new.example.com/", "https://old.example.com/"]
    );
}

#[test]
fn command_bar_search_ranks_open_tabs_first_and_merges_duplicates() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    let mut space = work_space();
    space.today.push(tab(
        8,
        "https://docs.example.com/archived",
        "Docs archived page",
        START,
    ));
    store.session.spaces.push(space);
    store.sweep(START.after(DAY), IDLE);
    let now = START.after(DAY);
    store
        .history_mut()
        .record("https://docs.example.com/guide", "Docs guide", now);
    store
        .history_mut()
        .record("https://tracker.example.com/", "Tracker", now);
    let open = [OpenTab {
        id: TabId(1),
        title: "Tracker",
        url: "https://tracker.example.com/",
    }];

    let results = store.search("docs", &open, now);
    let sources: Vec<(Source, &str)> = results
        .iter()
        .map(|result| (result.source, result.url.as_str()))
        .collect();
    assert_eq!(
        sources,
        [
            (Source::Pinned, "https://docs.example.com/"),
            (Source::History, "https://docs.example.com/guide"),
            (Source::Archived, "https://docs.example.com/archived"),
        ]
    );

    let tracker = store.search("track", &open, now);
    assert_eq!(tracker.len(), 1);
    assert_eq!(tracker[0].source, Source::OpenTab);
    assert_eq!(tracker[0].tab, Some(TabId(1)));

    let both_terms = store.search("docs guide", &open, now);
    assert_eq!(both_terms.len(), 1);
    assert_eq!(both_terms[0].url, "https://docs.example.com/guide");
    assert!(store.search("nothing-matches", &open, now).is_empty());
}
