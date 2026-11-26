mod common;

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use magpie::{
    config::{Scheme, Settings},
    import::{Report, Scope},
    internal::{
        Message, Page, resource,
        settings::{
            self, Hello, Host, PasswordAction, Passwords, Reply, Secret, Section, Snapshot,
        },
    },
    pages::PageAction,
    storage::{
        permissions::{Kind, Origin, Rules, State},
        profile::{ProfileId, Profiles},
        session::{Space, SpaceId},
    },
    update::Phase,
    zoom::Zoom,
};
use magpie_engine::update::Offer;
use serde_json::{Value, json};
use url::Url;
use zeroize::Zeroizing;

use common::node_accepts;

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

fn url(address: &str) -> Url {
    Url::parse(address).unwrap()
}

fn mime(address: &str) -> Option<&'static str> {
    resource(&url(address)).map(|resource| resource.mime)
}

fn origin(text: &str) -> Origin {
    Origin::parse(text).unwrap()
}

fn state(settings: &Settings, scheme: Scheme, default_downloads: Option<PathBuf>) -> Value {
    let mut profiles = Profiles::default();
    let work = profiles.create("Work").unwrap();
    let mut desk = Space::new(SpaceId(2), "Desk", "Desk");
    desk.profile = work;
    let spaces = [Space::new(SpaceId(1), "Home", "Personal"), desk];
    let permissions = BTreeMap::from([(
        origin("https://meet.example"),
        Rules::from([(Kind::Camera, State::Allow), (Kind::Location, State::Block)]),
    )]);
    let snapshot = Snapshot {
        settings,
        scheme,
        paused_sites: BTreeSet::from(["ads.example".to_owned()]),
        zooms: BTreeMap::from([("example.com".to_owned(), Zoom(110))]),
        permissions: &permissions,
        default_downloads,
        data_folder: Path::new("C:/Data/Magpie"),
        passwords: Passwords::Locked {
            email: Some("sam@example.com".to_owned()),
            hello: Hello {
                available: true,
                on: true,
            },
            problem: None,
        },
        profiles: &profiles,
        spaces: &spaces,
        update: Phase::Downloading {
            offer: Offer {
                version: "2.0.0".to_owned(),
                page: "https://github.com/example/magpie/releases/tag/v2.0.0".to_owned(),
                notes: "Faster tabs".to_owned(),
                download: None,
            },
            percent: 40,
        },
    };
    serde_json::from_str(&Reply::state(snapshot).json()).unwrap()
}

fn settings_message(message: Value) -> Option<settings::Message> {
    match Page::Settings.message(&message.to_string())? {
        Message::Settings(message) => Some(message),
        Message::Action(..) => None,
    }
}

fn attribute_values<'a>(html: &'a str, attribute: &str) -> Vec<&'a str> {
    let marker = format!("{attribute}=\"");
    html.split(marker.as_str())
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect()
}

#[test]
fn recognises_only_known_magpie_pages() {
    assert_eq!(Page::of(&url("magpie://settings")), Some(Page::Settings));
    assert_eq!(
        Page::of(&url("magpie://settings/#privacy")),
        Some(Page::Settings)
    );
    assert_eq!(Page::of(&url("magpie://history")), Some(Page::History));
    assert_eq!(Page::of(&url("magpie://Bookmarks")), Some(Page::Bookmarks));
    assert_eq!(Page::of(&url("magpie://downloads/")), Some(Page::Downloads));
    assert_eq!(Page::of(&url("magpie://newtab")), Some(Page::NewTab));
    assert_eq!(Page::of(&url("magpie://nowhere")), None);
    assert_eq!(Page::of(&url("https://settings/")), None);
    assert_eq!(Page::of(&Section::Privacy.url()), Some(Page::Settings));
    assert_eq!(Section::Privacy.url().as_str(), "magpie://settings#privacy");
    assert_eq!(
        Section::parse("passwords").map(Section::url),
        Some(url("magpie://settings#passwords"))
    );
}

#[test]
fn serves_each_page_its_document_and_assets() {
    assert_eq!(mime("magpie://settings"), Some("text/html; charset=utf-8"));
    assert_eq!(
        mime("magpie://settings/#appearance"),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        mime("magpie://settings/settings.css"),
        Some("text/css; charset=utf-8")
    );
    assert_eq!(
        mime("magpie://settings/theme.css"),
        Some("text/css; charset=utf-8")
    );
    assert_eq!(
        mime("magpie://settings/settings.js"),
        Some("text/javascript; charset=utf-8")
    );
    assert_eq!(mime("magpie://settings/missing.js"), None);
    assert_eq!(mime("magpie://nowhere/"), None);
    assert_eq!(mime("https://settings/settings.css"), None);
    let icon = resource(&url("magpie://settings/magpie.png")).unwrap();
    assert_eq!(icon.mime, "image/png");
    assert!(icon.bytes.starts_with(PNG_SIGNATURE));
}

#[test]
fn parses_settings_page_messages() {
    assert_eq!(
        settings_message(json!({"type": "set_setting", "key": "theme", "value": "Dark"})),
        Some(settings::Message::SetSetting {
            key: "theme".to_owned(),
            value: "Dark".to_owned(),
        })
    );
    assert_eq!(
        settings_message(json!({"type": "clear_zoom", "site": "example.com"})),
        Some(settings::Message::ClearZoom {
            site: "example.com".to_owned()
        })
    );
    assert_eq!(
        settings_message(json!({
            "type": "set_permission",
            "origin": "HTTPS://Meet.Example:443",
            "kind": "microphone",
            "state": "block",
        })),
        Some(settings::Message::SetPermission {
            origin: origin("https://meet.example"),
            kind: Kind::Microphone,
            state: State::Block,
        })
    );
    let internal_page = json!({
        "type": "set_permission",
        "origin": "magpie://settings",
        "kind": "camera",
        "state": "allow",
    });
    assert_eq!(settings_message(internal_page), None);
    assert_eq!(
        settings_message(json!({"type": "open_data_folder"})),
        Some(settings::Message::OpenDataFolder)
    );
    assert_eq!(
        settings_message(json!({"type": "install_update"})),
        Some(settings::Message::InstallUpdate)
    );
    let sign_in = json!({
        "type": "passwords",
        "action": {
            "kind": "sign_in",
            "host": "custom",
            "address": "https://vault.example.com",
            "email": "sam@example.com",
            "password": "hunter2",
        },
    });
    assert_eq!(
        settings_message(sign_in),
        Some(settings::Message::Passwords {
            action: PasswordAction::SignIn {
                host: Host::Custom,
                address: "https://vault.example.com".to_owned(),
                email: "sam@example.com".to_owned(),
                password: Secret(Zeroizing::new("hunter2".to_owned())),
            },
        })
    );
    assert_eq!(
        settings_message(json!({"type": "passwords", "action": {"kind": "set_hello", "on": true}})),
        Some(settings::Message::Passwords {
            action: PasswordAction::SetHello { on: true },
        })
    );
    assert_eq!(
        settings_message(json!({"type": "assign_profile", "space": 3, "profile": "default"})),
        Some(settings::Message::AssignProfile {
            space: SpaceId(3),
            profile: ProfileId::default(),
        })
    );
    let escaping_profile = json!({"type": "delete_profile", "profile": "../Default"});
    assert_eq!(settings_message(escaping_profile), None);
    assert_eq!(settings_message(json!({"type": "format_disk"})), None);
    assert_eq!(settings_message(json!({"type": "set_setting"})), None);
    assert_eq!(settings_message(json!("ready")), None);
}

#[test]
fn companion_pages_accept_only_page_actions() {
    let clear = json!({"kind": "magpie_page", "action": "clear_history"}).to_string();
    assert_eq!(
        Page::History.message(&clear),
        Some(Message::Action(Page::History, PageAction::ClearHistory))
    );
    assert_eq!(Page::Settings.message(&clear), None);
    let setting = json!({"type": "set_setting", "key": "theme", "value": "Dark"}).to_string();
    assert_eq!(Page::Downloads.message(&setting), None);
}

#[test]
fn companion_pages_load_only_served_assets_without_inline_code() {
    for page in [
        Page::History,
        Page::Bookmarks,
        Page::Downloads,
        Page::NewTab,
    ] {
        let document = resource(&page.url()).unwrap();
        let html = std::str::from_utf8(document.bytes).unwrap();
        assert!(!html.contains("<style"), "{page:?}");
        assert!(!html.contains("<script>"), "{page:?}");
        let linked = attribute_values(html, "href")
            .into_iter()
            .filter(|href| !href.contains(':'));
        let sources = attribute_values(html, "src");
        let scripts: Vec<_> = sources
            .iter()
            .copied()
            .filter(|source| source.ends_with(".js"))
            .collect();
        assert_eq!(scripts.len(), 2, "{page:?}");
        for name in linked.chain(sources.iter().copied()) {
            let asset = page.url().join(name).unwrap();
            assert!(resource(&asset).is_some(), "{page:?} {name}");
        }
        for name in scripts {
            let script = resource(&page.url().join(name).unwrap()).unwrap();
            assert!(node_accepts(script.bytes), "{page:?} {name}");
        }
    }
}

#[test]
fn import_messages_choose_what_to_import() {
    let scope = |bookmarks: bool, history: bool| {
        let message = json!({
            "type": "run_import",
            "profile": "C:/Profiles/Default",
            "bookmarks": bookmarks,
            "history": history,
        });
        match settings_message(message) {
            Some(settings::Message::RunImport(import)) => {
                assert_eq!(import.profile, PathBuf::from("C:/Profiles/Default"));
                import.scope()
            }
            other => panic!("not an import: {other:?}"),
        }
    };
    assert_eq!(scope(true, true), Some(Scope::Everything));
    assert_eq!(scope(true, false), Some(Scope::Bookmarks));
    assert_eq!(scope(false, true), Some(Scope::History));
    assert_eq!(scope(false, false), None);
}

#[test]
fn state_reflects_settings_changed_by_the_page() {
    let mut settings = Settings::default();
    let changes = [
        ("theme", "Dark"),
        ("accent", "#123456"),
        ("search_engine", "Custom"),
        ("custom_search", "https://search.example/?q=%s"),
        ("download_folder", "D:/Saved"),
        ("not_a_setting", "On"),
        ("shield", "Sideways"),
    ];
    for (key, value) in changes {
        settings.set(key, value);
    }
    let state = state(&settings, Scheme::Dark, Some(PathBuf::from("C:/Downloads")));
    assert_eq!(state["type"], "state");
    assert_eq!(state["settings"]["theme"], "Dark");
    assert_eq!(state["settings"].get("accent"), None);
    assert_eq!(state["settings"]["shield"], "On");
    assert_eq!(state["settings"].get("not_a_setting"), None);
    assert_eq!(state["scheme"], "dark");
    assert_eq!(state["custom_search_valid"], true);
    assert_eq!(state["downloads_folder"], "D:/Saved");
    assert_eq!(state["paused_sites"], json!(["ads.example"]));
    assert_eq!(
        state["zooms"],
        json!([{"site": "example.com", "zoom": "110%"}])
    );
    assert_eq!(
        state["permissions"],
        json!([
            {"origin": "https://meet.example", "kind": "camera", "label": "Camera", "state": "allow"},
            {"origin": "https://meet.example", "kind": "location", "label": "Location", "state": "block"},
        ])
    );
    let themes: Vec<&str> = state["choices"]["theme"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|choice| choice["value"].as_str())
        .collect();
    assert_eq!(themes, ["System", "Light", "Dark"]);
    assert_eq!(
        state["passwords"],
        json!({
            "status": "locked",
            "email": "sam@example.com",
            "hello": {"available": true, "on": true},
            "problem": null,
        })
    );
    assert_eq!(
        state["update"],
        json!({
            "state": "downloading",
            "offer": {
                "version": "2.0.0",
                "page": "https://github.com/example/magpie/releases/tag/v2.0.0",
                "notes": "Faster tabs",
                "download": null,
            },
            "percent": 40,
        })
    );
    let work = &state["profiles"][1]["id"];
    assert_eq!(
        state["profiles"],
        json!([
            {"id": "default", "name": "Default", "built_in": true},
            {"id": work, "name": "Work", "built_in": false},
        ])
    );
    assert_eq!(
        state["spaces"],
        json!([
            {"id": 1, "name": "Home", "profile": "default"},
            {"id": 2, "name": "Desk", "profile": work},
        ])
    );
}

#[test]
fn state_falls_back_to_the_default_download_folder() {
    let settings = Settings::default();
    let state = state(
        &settings,
        Scheme::Light,
        Some(PathBuf::from("C:/Downloads")),
    );
    assert_eq!(state["settings"]["download_folder"], "");
    assert_eq!(state["downloads_folder"], "C:/Downloads");
    assert_eq!(state["scheme"], "light");
}

#[test]
fn import_summaries_count_what_arrived() {
    let report = Report {
        bookmarks: 1,
        visits: 2,
        warnings: vec!["History: locked".to_owned()],
        ..Report::default()
    };
    let imported: Value = serde_json::from_str(&Reply::imported(report).json()).unwrap();
    assert_eq!(
        imported,
        json!({
            "type": "imported",
            "summary": "✓ Imported 1 bookmark, 2 history entries",
            "warnings": ["History: locked"],
        })
    );
}
