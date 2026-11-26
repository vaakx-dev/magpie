use std::fs;

use magpie::storage::permissions::{Kind, Origin, Permissions, State};
use url::Url;

fn origin(text: &str) -> Origin {
    Origin::parse(text).unwrap()
}

#[test]
fn origins_keep_only_scheme_host_and_port() {
    let page = Url::parse("HTTPS://Video.Example.COM:443/call?room=7#lobby").unwrap();

    assert_eq!(Origin::of(&page), Some(origin("https://video.example.com")));
    assert_eq!(
        origin("https://video.example.com/").host(),
        "video.example.com"
    );
    assert_eq!(origin("http://localhost:8080/app").host(), "localhost:8080");
    assert_ne!(origin("http://example.com"), origin("https://example.com"));
    assert_ne!(
        origin("https://example.com"),
        origin("https://example.com:8443")
    );
    assert_eq!(Origin::parse("magpie://settings"), None);
    assert_eq!(Origin::parse("file:///C:/notes.html"), None);
    assert_eq!(Origin::parse("not an address"), None);
}

#[test]
fn decisions_survive_a_reload_and_ask_forgets_them() {
    let folder = tempfile::tempdir().unwrap();
    let meet = origin("https://meet.example");
    let maps = origin("https://maps.example:8443");
    let mut permissions = Permissions::open(folder.path());
    permissions.set(&meet, Kind::Camera, State::Allow).unwrap();
    permissions
        .set(&meet, Kind::Microphone, State::Allow)
        .unwrap();
    permissions
        .set(&maps, Kind::Location, State::Block)
        .unwrap();
    permissions
        .set(&meet, Kind::Microphone, State::Ask)
        .unwrap();

    let reopened = Permissions::open(folder.path());
    assert_eq!(reopened.state(&meet, Kind::Camera), State::Allow);
    assert_eq!(reopened.state(&meet, Kind::Microphone), State::Ask);
    assert_eq!(reopened.state(&maps, Kind::Location), State::Block);
    let stored = fs::read_to_string(folder.path().join("permissions.conf")).unwrap();
    assert_eq!(
        stored,
        "https://maps.example:8443 location = block\nhttps://meet.example camera = allow\n"
    );
}

#[test]
fn a_rule_covers_only_its_own_origin_and_kind_and_the_last_one_wins() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(
        folder.path().join("permissions.conf"),
        "https://chat.example notifications = allow\n\
         https://chat.example notifications = block\n\
         https://chat.example teleport = allow\n\
         magpie://settings camera = allow\n\
         garbage\n",
    )
    .unwrap();
    let mut permissions = Permissions::open(folder.path());
    let chat = origin("https://chat.example");

    assert_eq!(permissions.state(&chat, Kind::Notifications), State::Block);
    permissions
        .set(&chat, Kind::Notifications, State::Allow)
        .unwrap();
    assert_eq!(permissions.state(&chat, Kind::Notifications), State::Allow);
    assert_eq!(permissions.state(&chat, Kind::Camera), State::Ask);
    for neighbour in [
        "http://chat.example",
        "https://chat.example:8443",
        "https://www.chat.example",
    ] {
        assert_eq!(
            permissions.state(&origin(neighbour), Kind::Notifications),
            State::Ask,
            "{neighbour}"
        );
    }
    assert_eq!(permissions.sites().len(), 1);
}

#[test]
fn private_window_decisions_never_reach_the_disk() {
    let folder = tempfile::tempdir().unwrap();
    let meet = origin("https://meet.example");
    let mut saved = Permissions::open(folder.path());
    saved.set(&meet, Kind::Camera, State::Allow).unwrap();
    let file = folder.path().join("permissions.conf");
    let before = fs::read_to_string(&file).unwrap();

    let mut private = saved.in_memory();
    private.set(&meet, Kind::Camera, State::Block).unwrap();
    private
        .set(
            &origin("https://secret.example"),
            Kind::Location,
            State::Allow,
        )
        .unwrap();

    assert_eq!(private.state(&meet, Kind::Camera), State::Block);
    assert_eq!(saved.state(&meet, Kind::Camera), State::Allow);
    assert_eq!(fs::read_to_string(&file).unwrap(), before);
    let reopened = Permissions::open(folder.path());
    assert_eq!(reopened.state(&meet, Kind::Camera), State::Allow);
    assert_eq!(reopened.sites().len(), 1);
}
