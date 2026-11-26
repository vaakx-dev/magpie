use std::fs;

use magpie::storage::{
    Store,
    profile::{ProfileId, Profiles},
    session::{Session, Space, SpaceId},
};

fn profile_id(json: &str) -> Option<ProfileId> {
    serde_json::from_str(json).ok()
}

#[test]
fn an_older_session_opens_every_space_in_the_default_profile() {
    let folder = tempfile::tempdir().unwrap();
    let old_session = r#"{"spaces": [{"id": 1, "name": "Work", "theme": "Work",
        "favorites": [], "pinned": [], "today": [], "active": null}], "active": 1}"#;
    fs::write(folder.path().join("session.json"), old_session).unwrap();

    let store = Store::open(folder.path()).unwrap();

    assert_eq!(store.session.spaces[0].profile, ProfileId::default());
    assert_eq!(store.session.spaces[0].profile.webview_name(), None);
    assert_eq!(store.profiles().ids(), [ProfileId::default()]);
}

#[test]
fn profiles_and_their_spaces_survive_a_reopen() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    let work = store.profiles_mut().create("  Work \t accounts ").unwrap();
    store.save_profiles().unwrap();
    let mut space = Space::new(SpaceId(1), "Office", "Work");
    space.profile = work.clone();
    store.session.spaces.push(space);
    store.save_session().unwrap();

    let reopened = Store::open(folder.path()).unwrap();

    assert_eq!(reopened.session, store.session);
    assert_eq!(
        reopened.profiles().custom_name(&work),
        Some("Work accounts")
    );
}

#[test]
fn a_session_pointing_at_a_missing_profile_falls_back_to_default() {
    let folder = tempfile::tempdir().unwrap();
    let mut store = Store::open(folder.path()).unwrap();
    let mut space = Space::new(SpaceId(1), "Office", "Work");
    space.profile = Profiles::default().create("Gone").unwrap();
    store.session.spaces.push(space);
    store.save_session().unwrap();

    let reopened = Store::open(folder.path()).unwrap();

    assert_eq!(reopened.session.spaces[0].profile, ProfileId::default());
}

#[test]
fn renaming_a_profile_keeps_its_webview_profile() {
    let mut profiles = Profiles::default();
    let id = profiles.create("Work").unwrap();
    let webview_name = id.webview_name().map(str::to_owned);

    assert!(profiles.rename(&id, "Client"));
    assert!(!profiles.rename(&id, "   "));
    assert!(!profiles.rename(&id, "Client"));

    let renamed = &profiles.custom()[0];
    assert_eq!(renamed.name(), "Client");
    assert_eq!(renamed.id, id);
    assert_eq!(renamed.id.webview_name().map(str::to_owned), webview_name);
}

#[test]
fn deleting_a_profile_moves_its_spaces_back_to_default() {
    let mut profiles = Profiles::default();
    let work = profiles.create("Work").unwrap();
    let play = profiles.create("Play").unwrap();
    let mut session = Session::default();
    for (id, profile) in [(1, &work), (2, &play), (3, &work)] {
        let mut space = Space::new(SpaceId(id), "Space", "Work");
        space.profile = profile.clone();
        session.spaces.push(space);
    }

    assert!(!profiles.delete(&ProfileId::default()));
    assert!(profiles.delete(&work));
    assert!(!profiles.delete(&work));
    assert!(session.keep_profiles(&profiles));

    let assigned: Vec<&ProfileId> = session.spaces.iter().map(|space| &space.profile).collect();
    assert_eq!(
        assigned,
        [&ProfileId::default(), &play, &ProfileId::default()]
    );
    assert!(profiles.contains(&ProfileId::default()));
    assert!(!profiles.contains(&work));
}

#[test]
fn new_profiles_get_distinct_webview_safe_ids_and_tidy_names() {
    let mut profiles = Profiles::default();
    let first = profiles.create("Work").unwrap();
    let second = profiles.create("Work").unwrap();
    let long = profiles.create(&"x".repeat(100)).unwrap();

    assert_ne!(first, second);
    for id in [&first, &second] {
        let name = id.webview_name().unwrap();
        let reparsed = profile_id(&serde_json::to_string(id).unwrap());
        assert_eq!(reparsed.as_ref(), Some(id));
        assert!(!name.eq_ignore_ascii_case("default"));
    }
    assert_eq!(profiles.custom_name(&long).map(str::len), Some(40));
    assert_eq!(profiles.create(" \n\t "), None);
    let control = profiles.create("Side\u{0}project").unwrap();
    assert_eq!(profiles.custom_name(&control), Some("Side project"));
}

#[test]
fn stored_profile_ids_must_be_valid_webview_profile_names() {
    assert_eq!(profile_id(r#""default""#), Some(ProfileId::default()));
    assert!(profile_id(r#""magpie-work (2)""#).is_some());
    for invalid in [
        r#""""#,
        r#""Default""#,
        r#""../escape""#,
        r#""trailing.""#,
        r#""trailing ""#,
        r#""名前""#,
    ] {
        assert_eq!(profile_id(invalid), None, "{invalid}");
    }
    let too_long = format!("\"{}\"", "a".repeat(65));
    assert_eq!(profile_id(&too_long), None);
}

#[test]
fn a_profiles_file_with_a_repeated_or_built_in_id_is_rejected() {
    let repeated = r#"[{"id": "magpie-a", "name": "A"}, {"id": "magpie-a", "name": "B"}]"#;
    let built_in = r#"[{"id": "default", "name": "Mine"}]"#;
    let unnamed = r#"[{"id": "magpie-a", "name": "  "}]"#;
    for invalid in [repeated, built_in, unnamed] {
        assert!(
            serde_json::from_str::<Profiles>(invalid).is_err(),
            "{invalid}"
        );
    }
}
