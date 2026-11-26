use std::fs;

use magpie::storage::Store;
use magpie::storage::recovery::{Exit, Sentinel};
use magpie::storage::session::{Space, SpaceId};

fn space(id: u64, name: &str) -> Space {
    Space::new(SpaceId(id), name, name)
}

#[test]
fn only_a_run_that_never_lowered_its_sentinel_counts_as_unexpected() {
    let folder = tempfile::tempdir().unwrap();
    let app_folder = folder.path().join("Magpie");

    let (first, exit) = Sentinel::raise(&app_folder).unwrap();
    assert_eq!(exit, Exit::Clean);
    first.lower().unwrap();

    let (second, exit) = Sentinel::raise(&app_folder).unwrap();
    assert_eq!(exit, Exit::Clean);
    drop(second);

    let (third, exit) = Sentinel::raise(&app_folder).unwrap();
    assert_eq!(exit, Exit::Unexpected);
    third.lower().unwrap();
    assert_eq!(Sentinel::raise(&app_folder).unwrap().1, Exit::Clean);
}

#[test]
fn a_session_broken_mid_write_comes_back_from_the_last_good_copy() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    store.session.spaces.push(space(1, "Work"));
    store.save_session().unwrap();
    let last_good = Store::open(folder.path()).unwrap().session;
    fs::write(folder.path().join("session.json"), b"{\"spaces\": [{\"id\"").unwrap();

    let recovered = Store::open(folder.path()).unwrap();

    assert_eq!(recovered.session, last_good);
    assert_eq!(recovered.session.spaces[0].name, "Work");
}

#[test]
fn private_windows_leave_nothing_to_recover() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    store.session.spaces.push(space(1, "Work"));
    store.save_session().unwrap();
    let mut private = store.new_private_window();
    private.session.spaces.push(space(2, "Secret"));
    private.save_session().unwrap();

    Store::open(folder.path()).unwrap();
    fs::remove_file(folder.path().join("session.json")).unwrap();
    let recovered = Store::open(folder.path()).unwrap();

    let names: Vec<&str> = recovered
        .session
        .spaces
        .iter()
        .map(|space| space.name.as_str())
        .collect();
    assert_eq!(names, ["Work"]);
}
