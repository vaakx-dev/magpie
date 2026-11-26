use std::path::PathBuf;

use magpie::{
    download::{Download, Speed, State},
    pages::{
        LocalTime, PageAction, bookmarks_payload, downloads_payload, history_payload,
        newtab_payload,
    },
    storage::{
        arrange::{Item, Spot},
        clock::Epoch,
        history::History,
        session::{
            Folder, FolderId, Link, Pane, Pinned, Section, Session, Space, SpaceId, Tab, TabId,
        },
    },
};
use serde_json::{Value, json};

const DAY_SECS: u64 = 24 * 60 * 60;
const HOUR_SECS: u64 = 60 * 60;
const MINUTE_SECS: u64 = 60;
const UTC_MIDNIGHT: u64 = 20_000 * DAY_SECS;
const BRISBANE_OFFSET_SECS: u64 = 10 * HOUR_SECS;
const LOCAL_MIDNIGHT: u64 = UTC_MIDNIGHT - BRISBANE_OFFSET_SECS;

fn visit_urls(group: &Value) -> Vec<&str> {
    group["visits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|visit| visit["url"].as_str().unwrap())
        .collect()
}

#[test]
fn history_groups_visits_by_local_calendar_day_newest_first() {
    let mut history = History::default();
    let visits = [
        (
            "https://seven.example/",
            LOCAL_MIDNIGHT - 6 * DAY_SECS - MINUTE_SECS,
        ),
        (
            "https://six.example/",
            LOCAL_MIDNIGHT - 5 * DAY_SECS - MINUTE_SECS,
        ),
        (
            "https://two.example/",
            LOCAL_MIDNIGHT - DAY_SECS - MINUTE_SECS,
        ),
        ("https://late.example/", LOCAL_MIDNIGHT - 30 * MINUTE_SECS),
        ("https://early.example/", LOCAL_MIDNIGHT + 30 * MINUTE_SECS),
        (
            "https://www.morning.example/a",
            LOCAL_MIDNIGHT + 8 * HOUR_SECS,
        ),
    ];
    for (url, at) in visits {
        history.record(url, "", Epoch(at));
    }
    let now = LocalTime {
        now: Epoch(LOCAL_MIDNIGHT + 9 * HOUR_SECS),
        utc_offset_secs: BRISBANE_OFFSET_SECS as i64,
    };

    let payload = history_payload(&history, now);
    let groups = payload["groups"].as_array().unwrap();
    let summary: Vec<(&str, Vec<&str>)> = groups
        .iter()
        .map(|group| (group["period"].as_str().unwrap(), visit_urls(group)))
        .collect();

    assert_eq!(
        summary,
        [
            (
                "Today",
                vec!["https://www.morning.example/a", "https://early.example/"]
            ),
            ("Yesterday", vec!["https://late.example/"]),
            (
                "This Week",
                vec!["https://two.example/", "https://six.example/"]
            ),
            ("Older", vec!["https://seven.example/"]),
        ]
    );
    assert_eq!(
        groups[0]["visits"][0],
        json!({
            "url": "https://www.morning.example/a",
            "title": "",
            "host": "www.morning.example",
            "visited": LOCAL_MIDNIGHT + 8 * HOUR_SECS,
        })
    );
}

#[test]
fn history_leaves_out_periods_without_visits() {
    let mut history = History::default();
    history.record("https://old.example/", "Old", Epoch(DAY_SECS));
    let now = LocalTime {
        now: Epoch(UTC_MIDNIGHT),
        utc_offset_secs: 0,
    };

    let payload = history_payload(&history, now);

    assert_eq!(payload["groups"].as_array().unwrap().len(), 1);
    assert_eq!(payload["groups"][0]["period"], "Older");
}

fn link(url: &str, title: &str) -> Link {
    Link {
        url: url.to_owned(),
        title: title.to_owned(),
    }
}

fn pinned_tab(id: u64, url: &str, title: &str) -> Pinned {
    let pane = Pane {
        url: url.to_owned(),
        title: title.to_owned(),
    };
    Pinned::Tab(Tab::new(TabId(id), pane, Epoch(0)))
}

fn work_space() -> Space {
    let mut space = Space::new(SpaceId(2), "Work", "blue");
    space.favorites = vec![link("https://mail.example.com/inbox", "Mail")];
    let mut folder = Folder::new(FolderId(5), "Docs");
    folder.collapsed = true;
    folder.items = vec![pinned_tab(9, "https://docs.example.com/", "Docs home")];
    space.pinned = vec![
        pinned_tab(8, "https://board.example.com/", "Board"),
        Pinned::Folder(folder),
    ];
    space
}

#[test]
fn bookmarks_list_every_space_with_favorites_and_nested_pinned_items() {
    let session = Session {
        spaces: vec![Space::new(SpaceId(1), "Personal", "teal"), work_space()],
        active: Some(SpaceId(2)),
    };

    assert_eq!(
        bookmarks_payload(&session),
        json!({
            "active": 2,
            "spaces": [
                { "id": 1, "name": "Personal", "favorites": [], "pinned": [] },
                {
                    "id": 2,
                    "name": "Work",
                    "favorites": [
                        { "url": "https://mail.example.com/inbox", "title": "Mail", "host": "mail.example.com" },
                    ],
                    "pinned": [
                        { "kind": "tab", "id": 8, "url": "https://board.example.com/", "title": "Board", "host": "board.example.com" },
                        {
                            "kind": "folder",
                            "id": 5,
                            "name": "Docs",
                            "collapsed": true,
                            "items": [
                                { "kind": "tab", "id": 9, "url": "https://docs.example.com/", "title": "Docs home", "host": "docs.example.com" },
                            ],
                        },
                    ],
                },
            ],
        })
    );
}

#[test]
fn newtab_shows_the_space_favorites_as_shortcuts() {
    assert_eq!(
        newtab_payload(&work_space()),
        json!({
            "space": "Work",
            "favorites": [
                { "url": "https://mail.example.com/inbox", "title": "Mail", "host": "mail.example.com" },
            ],
        })
    );
}

fn download(name: &str, received_bytes: u64, total_bytes: Option<u64>, state: State) -> Download {
    Download {
        url: format!("https://files.example.com/{name}"),
        path: PathBuf::from(format!(r"C:\Downloads\{name}")),
        received_bytes,
        total_bytes,
        state,
        pausable: true,
        speed: None,
        note: None,
    }
}

#[test]
fn downloads_put_active_transfers_first_with_progress_and_status() {
    let downloads = [
        download("done.zip", 2048, Some(2048), State::Completed),
        download("broken.iso", 10, Some(4096), State::Interrupted),
        Download {
            speed: Some(Speed {
                bytes_per_sec: 1024,
            }),
            ..download("live.mp4", 1024, Some(4096), State::InProgress)
        },
        download("held.msi", 2048, Some(8192), State::Paused),
        download("stopped.exe", 0, None, State::Cancelled),
    ];

    let payload = downloads_payload(&downloads);

    assert_eq!(
        payload["downloads"][0],
        json!({
            "name": "live.mp4",
            "url": "https://files.example.com/live.mp4",
            "path": r"C:\Downloads\live.mp4",
            "status": "active",
            "pausable": true,
            "progress": 0.25,
            "readout": "1.0 KB of 4.0 KB · 1.0 KB/s · 3s left",
            "size": "4.0 KB",
        })
    );
    let statuses: Vec<&str> = payload["downloads"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["status"].as_str().unwrap())
        .collect();
    assert_eq!(
        payload["downloads"][1]["readout"],
        "Paused · 2.0 KB of 8.0 KB"
    );
    assert_eq!(
        statuses,
        ["active", "paused", "completed", "failed", "cancelled"]
    );
}

#[test]
fn actions_parse_from_page_messages() {
    let cases = [
        (
            r#"{"kind":"magpie_page","page":"history","action":"ready"}"#,
            PageAction::Ready,
        ),
        (
            r#"{"kind":"magpie_page","page":"newtab","action":"open_url","url":"https://example.com/"}"#,
            PageAction::OpenUrl {
                url: "https://example.com/".to_owned(),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"newtab","action":"navigate","text":"rust lifetimes"}"#,
            PageAction::Navigate {
                text: "rust lifetimes".to_owned(),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"focus_tab","tab":8}"#,
            PageAction::FocusTab { tab: TabId(8) },
        ),
        (
            r#"{"kind":"magpie_page","page":"history","action":"delete_history","url":"https://a.example/"}"#,
            PageAction::DeleteHistory {
                url: "https://a.example/".to_owned(),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"history","action":"clear_history"}"#,
            PageAction::ClearHistory,
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"remove_favorite","space":2,"url":"https://b.example/"}"#,
            PageAction::RemoveFavorite {
                space: SpaceId(2),
                url: "https://b.example/".to_owned(),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"remove_pinned","tab":9}"#,
            PageAction::RemovePinned { tab: TabId(9) },
        ),
        (
            r#"{"kind":"magpie_page","page":"downloads","action":"open_download","path":"C:\\Downloads\\a.zip"}"#,
            PageAction::OpenDownload {
                path: PathBuf::from(r"C:\Downloads\a.zip"),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"downloads","action":"show_download_in_folder","path":"C:\\Downloads\\a.zip"}"#,
            PageAction::ShowDownloadInFolder {
                path: PathBuf::from(r"C:\Downloads\a.zip"),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"downloads","action":"pause_download","path":"C:\\Downloads\\b.iso"}"#,
            PageAction::PauseDownload {
                path: PathBuf::from(r"C:\Downloads\b.iso"),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"downloads","action":"resume_download","path":"C:\\Downloads\\b.iso"}"#,
            PageAction::ResumeDownload {
                path: PathBuf::from(r"C:\Downloads\b.iso"),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"downloads","action":"cancel_download","path":"C:\\Downloads\\b.iso"}"#,
            PageAction::CancelDownload {
                path: PathBuf::from(r"C:\Downloads\b.iso"),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"downloads","action":"clear_downloads"}"#,
            PageAction::ClearDownloads,
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"open_settings_section","section":"import"}"#,
            PageAction::OpenSettingsSection {
                section: "import".to_owned(),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"edit_favorite","space":2,"url":"https://b.example/","edited":{"title":"Bee","url":" bee.example "}}"#,
            PageAction::EditFavorite {
                space: SpaceId(2),
                url: "https://b.example/".to_owned(),
                edited: link("https://bee.example/", "Bee"),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"rename_tab","tab":8,"title":"Kanban"}"#,
            PageAction::RenameTab {
                tab: TabId(8),
                title: "Kanban".to_owned(),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"rename_folder","folder":5,"name":"Reading"}"#,
            PageAction::RenameFolder {
                folder: FolderId(5),
                name: "Reading".to_owned(),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"move_favorite","space":2,"url":"https://b.example/","slot":3}"#,
            PageAction::MoveFavorite {
                space: SpaceId(2),
                url: "https://b.example/".to_owned(),
                slot: 3,
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"move_pinned","item":{"kind":"tab","id":8},"spot":{"after":{"kind":"folder","id":5}}}"#,
            PageAction::MovePinned {
                item: Item::Tab(TabId(8)),
                spot: Spot::After(Item::Folder(FolderId(5))),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"move_pinned","item":{"kind":"folder","id":5},"spot":{"into":6}}"#,
            PageAction::MovePinned {
                item: Item::Folder(FolderId(5)),
                spot: Spot::Into(FolderId(6)),
            },
        ),
        (
            r#"{"kind":"magpie_page","page":"bookmarks","action":"move_pinned","item":{"kind":"tab","id":8},"spot":{"top":"pinned"}}"#,
            PageAction::MovePinned {
                item: Item::Tab(TabId(8)),
                spot: Spot::Top(Section::Pinned),
            },
        ),
    ];
    for (json, expected) in cases {
        assert_eq!(PageAction::parse(json), Some(expected), "{json}");
    }
}

#[test]
fn actions_reject_foreign_or_malformed_messages() {
    let rejected = [
        r#"{"kind":"background","url":"https://example.com/"}"#,
        r#"{"kind":"magpie_page","action":"format_disk"}"#,
        r#"{"kind":"magpie_page","action":"open_url"}"#,
        r#"{"kind":"magpie_page","action":"focus_tab","tab":"eight"}"#,
        r#"{"kind":"magpie_page","action":"move_favorite","space":2,"url":"https://b.example/","slot":-1}"#,
        r#"{"kind":"magpie_page","action":"move_pinned","item":{"kind":"tab","id":8},"spot":{"sideways":5}}"#,
        r#"{"kind":"magpie_page","action":"rename_folder","folder":5}"#,
        r#"{"kind":"magpie_page","action":"edit_favorite","space":2,"url":"https://b.example/","edited":{"title":"Bee","url":"not a site"}}"#,
        "not json",
    ];
    for json in rejected {
        assert_eq!(PageAction::parse(json), None, "{json}");
    }
}

fn work_session() -> Session {
    let mut space = work_space();
    space.favorites.extend([
        link("https://news.example.com/", "News"),
        link("https://chat.example.com/", "Chat"),
    ]);
    space.today = vec![Tab::new(
        TabId(20),
        Pane {
            url: "https://today.example.com/".to_owned(),
            title: "Today".to_owned(),
        },
        Epoch(0),
    )];
    Session {
        spaces: vec![space],
        active: Some(SpaceId(2)),
    }
}

fn favorite_titles(session: &Session) -> Vec<&str> {
    let favorites = &session.spaces[0].favorites;
    favorites.iter().map(|link| link.title.as_str()).collect()
}

fn pinned_outline(items: &[Pinned]) -> String {
    let names: Vec<String> = items
        .iter()
        .map(|item| match item {
            Pinned::Tab(tab) => tab.id.0.to_string(),
            Pinned::Folder(folder) => format!("{}[{}]", folder.id.0, pinned_outline(&folder.items)),
        })
        .collect();
    names.join(" ")
}

#[test]
fn renames_trim_names_and_ignore_blanks_and_unknown_items() {
    let mut session = work_session();
    let mail = "https://mail.example.com/inbox";

    let titled = |title: &str| link(mail, title);

    assert!(session.edit_favorite(SpaceId(2), mail, titled("  Inbox ")));
    assert!(!session.edit_favorite(SpaceId(2), mail, titled("   ")));
    assert!(!session.edit_favorite(SpaceId(2), mail, titled("Inbox")));
    assert!(!session.edit_favorite(SpaceId(2), "https://gone.example/", titled("Gone")));
    assert!(!session.edit_favorite(SpaceId(9), mail, titled("Elsewhere")));
    assert!(session.rename_folder(FolderId(5), " Reading "));
    assert!(!session.rename_folder(FolderId(5), ""));
    assert!(!session.rename_folder(FolderId(99), "Missing"));
    assert!(session.rename_tab(TabId(8), " Kanban "));
    assert!(!session.rename_tab(TabId(99), "Missing"));

    let saved: Session = serde_json::from_value(serde_json::to_value(&session).unwrap()).unwrap();
    assert_eq!(saved, session);
    let payload = bookmarks_payload(&saved);
    let space = &payload["spaces"][0];
    assert_eq!(space["favorites"][0]["title"], "Inbox");
    assert_eq!(space["pinned"][0]["title"], "Kanban");
    assert_eq!(space["pinned"][1]["name"], "Reading");
}

#[test]
fn renaming_a_tab_to_nothing_brings_back_its_page_title() {
    let mut session = work_session();

    assert!(session.rename_tab(TabId(8), "Kanban"));
    assert!(session.rename_tab(TabId(8), "  "));

    assert_eq!(
        bookmarks_payload(&session)["spaces"][0]["pinned"][0]["title"],
        "Board"
    );
    assert!(!session.rename_tab(TabId(8), ""));
}

#[test]
fn editing_a_favorite_moves_its_url_unless_another_favorite_has_it() {
    let mut session = work_session();
    let news = "https://news.example.com/";

    assert!(session.edit_favorite(SpaceId(2), news, link("https://paper.example/", "Paper")));
    assert!(!session.edit_favorite(
        SpaceId(2),
        "https://paper.example/",
        link("https://chat.example.com/", "Chat")
    ));

    let favorites = &session.spaces[0].favorites;
    assert!(favorites.contains(&link("https://paper.example/", "Paper")));
    assert!(!favorites.iter().any(|favorite| favorite.url == news));
    assert_eq!(favorite_titles(&session), ["Mail", "Paper", "Chat"]);
}

#[test]
fn favorites_move_by_url_with_the_slot_clamped_to_the_list() {
    let mut session = work_session();

    assert!(session.move_favorite(SpaceId(2), "https://mail.example.com/inbox", 99));
    assert_eq!(favorite_titles(&session), ["News", "Chat", "Mail"]);
    assert!(session.move_favorite(SpaceId(2), "https://chat.example.com/", 0));
    assert_eq!(favorite_titles(&session), ["Chat", "News", "Mail"]);
    assert!(!session.move_favorite(SpaceId(2), "https://gone.example/", 0));
    assert!(!session.move_favorite(SpaceId(9), "https://chat.example.com/", 2));
    assert_eq!(favorite_titles(&session), ["Chat", "News", "Mail"]);
}

#[test]
fn pinned_items_reorder_only_within_the_pinned_section() {
    let mut session = work_session();
    let original = session.clone();

    assert!(!session.arrange_pinned(Item::Tab(TabId(8)), Spot::Top(Section::Today)));
    assert!(!session.arrange_pinned(Item::Tab(TabId(8)), Spot::Before(Item::Tab(TabId(20)))));
    assert!(!session.arrange_pinned(Item::Tab(TabId(20)), Spot::Top(Section::Pinned)));
    assert!(!session.arrange_pinned(Item::Tab(TabId(99)), Spot::Top(Section::Pinned)));
    assert!(!session.arrange_pinned(Item::Folder(FolderId(5)), Spot::Into(FolderId(5))));
    assert_eq!(session, original);

    assert!(session.arrange_pinned(Item::Tab(TabId(8)), Spot::After(Item::Folder(FolderId(5)))));
    assert_eq!(pinned_outline(&session.spaces[0].pinned), "5[9] 8");
    assert!(session.arrange_pinned(Item::Tab(TabId(8)), Spot::Into(FolderId(5))));
    assert_eq!(pinned_outline(&session.spaces[0].pinned), "5[9 8]");
    assert!(session.arrange_pinned(Item::Tab(TabId(8)), Spot::Before(Item::Tab(TabId(9)))));
    assert_eq!(pinned_outline(&session.spaces[0].pinned), "5[8 9]");
    assert!(session.arrange_pinned(Item::Tab(TabId(9)), Spot::Top(Section::Pinned)));
    assert_eq!(pinned_outline(&session.spaces[0].pinned), "9 5[8]");
}
