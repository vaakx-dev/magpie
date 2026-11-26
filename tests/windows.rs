use std::collections::BTreeSet;

use magpie::storage::Store;
use magpie::storage::clock::Epoch;
use magpie::storage::session::{Pane, Space, SpaceId, Tab, TabId};
use magpie::zoom::{Zoom, Zooms};
use magpie_engine::blocker::Shield;

const NOW: Epoch = Epoch(1_800_000_000);
const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfavicon";

fn space(id: u64, name: &str, url: &str) -> Space {
    let mut space = Space::new(SpaceId(id), name, name);
    let pane = Pane {
        url: url.to_owned(),
        title: name.to_owned(),
    };
    space.today.push(Tab::new(TabId(id), pane, NOW));
    space
}

#[test]
fn only_the_primary_window_saves_its_session() {
    let folder = tempfile::tempdir().unwrap();
    let mut primary = Store::open(folder.path()).unwrap();
    primary
        .session
        .spaces
        .push(space(1, "Work", "https://work.example.com/"));
    primary.save_session().unwrap();
    let mut secondary = primary.new_window();
    let mut private = primary.new_private_window();

    secondary
        .session
        .spaces
        .push(space(2, "Errands", "https://shop.example.com/"));
    private
        .session
        .spaces
        .push(space(3, "Secret", "https://secret.example.com/"));
    secondary.save().unwrap();
    private.save().unwrap();

    let reopened = Store::open(folder.path()).unwrap();
    assert!(
        secondary
            .session
            .spaces
            .iter()
            .all(|space| space.id != SpaceId(1))
    );
    assert_eq!(reopened.session, primary.session);
}

#[test]
fn private_windows_record_no_history_while_other_windows_share_it() {
    let folder = tempfile::tempdir().unwrap();
    let primary = Store::open(folder.path()).unwrap();
    let secondary = primary.new_window();
    let private = primary.new_private_window();

    secondary.record_visit("https://news.example.com/", "News", NOW);
    private.record_visit("https://secret.example.com/", "Secret", NOW);

    assert!(
        primary
            .history()
            .visit("https://news.example.com/")
            .is_some()
    );
    primary.save_history().unwrap();
    let reopened = Store::open(folder.path()).unwrap();
    let urls: Vec<String> = reopened
        .history()
        .visits()
        .map(|visit| visit.url.clone())
        .collect();
    assert_eq!(urls, ["https://news.example.com/"]);
}

#[test]
fn private_windows_cache_no_favicons_on_disk() {
    let folder = tempfile::tempdir().unwrap();
    let primary = Store::open(folder.path()).unwrap();
    let private = primary.new_private_window();

    primary.favicons().write("news.example.com", PNG).unwrap();
    private.favicons().write("secret.example.com", PNG).unwrap();

    let reopened = Store::open(folder.path()).unwrap().favicons();
    assert_eq!(reopened.read("news.example.com").as_deref(), Some(PNG));
    assert_eq!(reopened.read("secret.example.com"), None);
}

#[test]
fn private_windows_keep_shield_pauses_in_memory() {
    let folder = tempfile::tempdir().unwrap();
    let primary = Store::open(folder.path()).unwrap();
    let private = primary.new_private_window();
    let shield = Shield::default();
    shield.pause("news.example.com");
    primary.save_paused_sites(&shield.paused_sites()).unwrap();
    let private_shield = shield.private();

    private_shield.pause("secret.example.com");
    private_shield.resume("news.example.com");
    private
        .save_paused_sites(&private_shield.paused_sites())
        .unwrap();

    let saved = Store::open(folder.path()).unwrap().paused_sites().unwrap();
    assert_eq!(saved, BTreeSet::from(["news.example.com".to_owned()]));
    assert_eq!(shield.paused_sites(), saved);
}

#[test]
fn private_windows_zoom_from_saved_levels_without_saving_their_own() {
    let folder = tempfile::tempdir().unwrap();
    let mut zooms = Zooms::open(folder.path());
    zooms
        .set("news.example.com", Zoom(125), Zoom::default())
        .unwrap();
    let mut private = zooms.in_memory();

    private
        .set("secret.example.com", Zoom(150), Zoom::default())
        .unwrap();
    private
        .set("news.example.com", Zoom::default(), Zoom::default())
        .unwrap();

    assert_eq!(private.of("secret.example.com", Zoom::default()), Zoom(150));
    let reopened = Zooms::open(folder.path());
    assert_eq!(reopened.of("news.example.com", Zoom::default()), Zoom(125));
    assert_eq!(
        reopened.of("secret.example.com", Zoom::default()),
        Zoom::default()
    );
}
