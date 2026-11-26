use std::fs;
use std::path::Path;

use magpie::import::{Browser, Node, Profile, Report, Roots, Scope};
use magpie::storage::Store;
use magpie::storage::clock::Epoch;
use magpie::storage::history::Visit;
use magpie::storage::session::{Pinned, Space, SpaceId};
use rusqlite::Connection;
use tempfile::TempDir;

const NOW: Epoch = Epoch(1_800_000_000);

const LOCAL_STATE: &str = r#"{"profile": {"info_cache": {
    "Default": {"name": "Person 1"},
    "Profile 1": {"name": "Work"},
    "Profile 2": {"name": "Gone"}
}}}"#;

const BOOKMARKS: &str = r#"{"version": 1, "roots": {
    "bookmark_bar": {"type": "folder", "name": "Bookmarks bar", "children": [
        {"type": "url", "name": "News", "url": "https://news.example.com/"},
        {"type": "folder", "name": "Projects", "children": [
            {"type": "url", "name": "Tracker", "url": "https://tracker.example.org/board"},
            {"type": "url", "name": "Bookmarklet", "url": "javascript:alert(1)"},
            {"type": "folder", "name": "Archive", "children": [
                {"type": "url", "name": "Old wiki", "url": "http://wiki.example.net/"}
            ]}
        ]}
    ]},
    "other": {"type": "folder", "name": "Other bookmarks", "children": [
        {"type": "url", "name": "Settings", "url": "chrome://settings/"}
    ]},
    "synced": {"type": "folder", "name": "Mobile bookmarks", "children": []}
}}"#;

const PROFILES_INI: &str = "[General]\nStartWithLastProfile=1\n\n\
    [Profile1]\nName=empty\nIsRelative=1\nPath=Profiles/empty.profile\n\n\
    [Profile0]\nName=default-release\nIsRelative=1\nPath=Profiles/abcd.default-release\nDefault=1\n\n\
    [Install308046B0AF4A39CB]\nDefault=Profiles/abcd.default-release\n";

fn bookmark(title: &str, url: &str) -> Node {
    Node::Bookmark {
        title: title.to_owned(),
        url: url.to_owned(),
    }
}

fn folder(name: &str, children: Vec<Node>) -> Node {
    Node::Folder {
        name: name.to_owned(),
        children,
    }
}

fn visit(url: &str, title: &str, last_visit: u64, count: u32) -> Visit {
    Visit {
        url: url.to_owned(),
        title: title.to_owned(),
        last_visit: Epoch(last_visit),
        count,
    }
}

fn chrome_history(path: &Path) -> Connection {
    let connection = Connection::open(path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE urls (id INTEGER PRIMARY KEY, url TEXT, title TEXT, visit_count INTEGER,
                typed_count INTEGER, last_visit_time INTEGER, hidden INTEGER);
            INSERT INTO urls (url, title, visit_count, typed_count, last_visit_time, hidden) VALUES
                ('https://news.example.com/', 'News', 12, 0, 13344473600000000, 0),
                ('https://docs.example.org/guide', 'Guide', 3, 0, 13344473700500000, 0),
                ('https://news.example.com/', 'News again', 1, 0, 13344473000000000, 0),
                ('chrome://settings/', 'Settings', 9, 0, 13344473800000000, 0),
                ('file:///C:/notes.txt', 'Notes', 2, 0, 13344473800000000, 0),
                ('https://hidden.example.com/frame', '', 4, 0, 13344473800000000, 1),
                ('https://never.example.com/', NULL, 0, 0, 0, 0);",
        )
        .unwrap();
    connection
}

fn chrome_roots(root: &Path) -> Connection {
    let user_data = root.join("Google/Chrome/User Data");
    let default = user_data.join("Default");
    let work = user_data.join("Profile 1");
    fs::create_dir_all(&default).unwrap();
    fs::create_dir_all(&work).unwrap();
    fs::create_dir_all(user_data.join("Profile 3")).unwrap();
    fs::write(user_data.join("Local State"), LOCAL_STATE).unwrap();
    fs::write(default.join("Bookmarks"), BOOKMARKS).unwrap();
    fs::write(work.join("Bookmarks"), r#"{"roots": {}}"#).unwrap();
    chrome_history(&default.join("History"))
}

fn firefox_places(path: &Path) -> Connection {
    let connection = Connection::open(path).unwrap();
    connection
        .execute_batch(
            "PRAGMA journal_mode = WAL;
            PRAGMA wal_autocheckpoint = 0;
            CREATE TABLE moz_places (id INTEGER PRIMARY KEY, url TEXT, title TEXT,
                visit_count INTEGER, hidden INTEGER, last_visit_date INTEGER);
            CREATE TABLE moz_bookmarks (id INTEGER PRIMARY KEY, type INTEGER, fk INTEGER,
                parent INTEGER, position INTEGER, title TEXT, guid TEXT);
            INSERT INTO moz_places VALUES
                (1, 'https://recipes.example.com/', 'Recipes', 7, 0, 1700000000123456),
                (2, 'https://maps.example.org/', 'Maps', 2, 0, 1700000500000000),
                (3, 'place:sort=8&maxResults=10', 'Most visited', 0, 1, NULL),
                (4, 'about:preferences', 'Settings', 5, 0, 1700000900000000),
                (5, 'https://unvisited.example.net/', 'Bookmarked only', 0, 0, NULL),
                (6, 'https://tagged.example.com/', 'Tagged', 1, 0, 1600000000000000);
            INSERT INTO moz_bookmarks VALUES
                (1, 2, NULL, 0, 0, '', 'root________'),
                (2, 2, NULL, 1, 0, 'menu', 'menu________'),
                (3, 2, NULL, 1, 1, 'toolbar', 'toolbar_____'),
                (4, 2, NULL, 1, 2, 'tags', 'tags________'),
                (5, 2, NULL, 1, 3, 'unfiled', 'unfiled_____'),
                (6, 2, NULL, 1, 4, 'mobile', 'mobile______'),
                (10, 1, 2, 3, 1, 'Maps', 'bm_maps_____'),
                (11, 1, 1, 3, 0, 'Recipes', 'bm_recipes__'),
                (12, 2, NULL, 2, 0, 'Kitchen', 'fd_kitchen__'),
                (13, 1, 1, 12, 1, 'Soup', 'bm_soup_____'),
                (14, 3, NULL, 12, 0, NULL, 'sp_one______'),
                (15, 1, 3, 12, 2, 'Most visited', 'bm_query____'),
                (16, 2, NULL, 4, 0, 'cooking', 'tg_cooking__'),
                (17, 1, 6, 16, 0, NULL, 'bm_tagged___'),
                (18, 1, 5, 5, 0, NULL, 'bm_untitled_');",
        )
        .unwrap();
    connection
}

fn firefox_roots(root: &Path) -> Connection {
    let base = root.join("Mozilla/Firefox");
    let profile = base.join("Profiles/abcd.default-release");
    fs::create_dir_all(&profile).unwrap();
    fs::create_dir_all(base.join("Profiles/empty.profile")).unwrap();
    fs::write(base.join("profiles.ini"), PROFILES_INI).unwrap();
    firefox_places(&profile.join("places.sqlite"))
}

fn roots(home: &TempDir) -> Roots {
    Roots {
        local: home.path().join("Local"),
        roaming: home.path().join("Roaming"),
    }
}

fn profile(roots: &Roots, browser: Browser) -> Profile {
    roots
        .profiles()
        .into_iter()
        .find(|profile| profile.browser == browser)
        .unwrap()
}

#[test]
fn detects_profiles_that_hold_browser_data_with_their_display_names() {
    let home = TempDir::new().unwrap();
    let roots = roots(&home);
    let _chrome = chrome_roots(&roots.local);
    let _firefox = firefox_roots(&roots.roaming);
    let found: Vec<(Browser, String)> = roots
        .profiles()
        .into_iter()
        .map(|profile| (profile.browser, profile.name))
        .collect();
    assert_eq!(
        found,
        [
            (Browser::Chrome, "Person 1".to_owned()),
            (Browser::Chrome, "Work".to_owned()),
            (Browser::Firefox, "default-release".to_owned()),
        ]
    );
}

#[test]
fn chromium_import_keeps_folders_and_reads_history_while_the_browser_locks_it() {
    let home = TempDir::new().unwrap();
    let roots = roots(&home);
    let running = chrome_roots(&roots.local);
    running.execute_batch("BEGIN EXCLUSIVE;").unwrap();

    let imported = profile(&roots, Browser::Chrome).read(Scope::Everything);

    assert_eq!(imported.warnings, Vec::<String>::new());
    assert_eq!(
        imported.bookmarks,
        [folder(
            "Bookmarks bar",
            vec![
                bookmark("News", "https://news.example.com/"),
                folder(
                    "Projects",
                    vec![
                        bookmark("Tracker", "https://tracker.example.org/board"),
                        folder(
                            "Archive",
                            vec![bookmark("Old wiki", "http://wiki.example.net/")]
                        ),
                    ]
                ),
            ]
        )]
    );
    assert_eq!(
        imported.history,
        [
            visit("https://docs.example.org/guide", "Guide", 1_700_000_100, 3),
            visit("https://news.example.com/", "News", 1_700_000_000, 12),
        ]
    );
}

#[test]
fn firefox_import_skips_tags_separators_and_queries_and_sees_unflushed_writes() {
    let home = TempDir::new().unwrap();
    let roots = roots(&home);
    let _running = firefox_roots(&roots.roaming);

    let imported = profile(&roots, Browser::Firefox).read(Scope::Everything);

    assert_eq!(imported.warnings, Vec::<String>::new());
    assert_eq!(
        imported.bookmarks,
        [
            folder(
                "Bookmarks Toolbar",
                vec![
                    bookmark("Recipes", "https://recipes.example.com/"),
                    bookmark("Maps", "https://maps.example.org/"),
                ]
            ),
            folder(
                "Bookmarks Menu",
                vec![folder(
                    "Kitchen",
                    vec![bookmark("Soup", "https://recipes.example.com/")]
                )]
            ),
            folder(
                "Other Bookmarks",
                vec![bookmark("", "https://unvisited.example.net/")]
            ),
        ]
    );
    assert_eq!(
        imported.history,
        [
            visit("https://maps.example.org/", "Maps", 1_700_000_500, 2),
            visit("https://recipes.example.com/", "Recipes", 1_700_000_000, 7),
            visit("https://tagged.example.com/", "Tagged", 1_600_000_000, 1),
        ]
    );
}

#[test]
fn import_pins_bookmarks_in_the_active_space_and_merges_history_without_inflating_it() {
    let home = TempDir::new().unwrap();
    let roots = roots(&home);
    let _firefox = firefox_roots(&roots.roaming);
    let mut store = Store::open(&home.path().join("Magpie")).unwrap();
    store
        .session
        .spaces
        .push(Space::new(SpaceId(1), "Home", "Home"));
    store.session.active = Some(SpaceId(1));
    store.history_mut().merge(
        [visit(
            "https://maps.example.org/",
            "My maps",
            1_750_000_000,
            40,
        )],
        NOW,
    );
    let firefox = profile(&roots, Browser::Firefox);

    let first = firefox.import(&mut store, Scope::Everything, NOW);
    let again = firefox.import(&mut store, Scope::History, NOW);

    assert_eq!(
        first,
        Report {
            bookmarks: 4,
            folders: 4,
            visits: 3,
            warnings: Vec::new(),
        }
    );
    assert_eq!(again.visits, 3);
    assert_eq!(
        store.history().visit("https://maps.example.org/"),
        Some(&visit(
            "https://maps.example.org/",
            "My maps",
            1_750_000_000,
            40
        ))
    );
    assert_eq!(
        store.history().visit("https://recipes.example.com/"),
        Some(&visit(
            "https://recipes.example.com/",
            "Recipes",
            1_700_000_000,
            7
        ))
    );
    let space = store.session.active_space().unwrap();
    let [Pinned::Folder(imported)] = space.pinned.as_slice() else {
        panic!("expected one imported folder, found {:?}", space.pinned);
    };
    assert_eq!(imported.name, "Firefox (default-release)");
    let urls: Vec<&str> = space.pinned_tabs().iter().map(|tab| tab.url()).collect();
    assert_eq!(
        urls,
        [
            "https://recipes.example.com/",
            "https://maps.example.org/",
            "https://recipes.example.com/",
            "https://unvisited.example.net/",
        ]
    );
}

#[test]
fn missing_browser_files_become_warnings_instead_of_failures() {
    let home = TempDir::new().unwrap();
    let missing = Profile {
        browser: Browser::Edge,
        name: "Profile 1".to_owned(),
        folder: home.path().join("nowhere"),
    };
    let imported = missing.read(Scope::Everything);
    assert_eq!(imported.bookmarks, Vec::new());
    assert_eq!(imported.history, Vec::new());
    assert_eq!(imported.warnings.len(), 2);
}
