use std::fs;

use magpie::zoom::{Step, Zoom, Zooms};

const DEFAULT: Zoom = Zoom(80);

#[test]
fn stepping_walks_the_standard_ladder_and_stops_at_its_ends() {
    assert_eq!(Zoom(100).step(Step::In, DEFAULT), Zoom(110));
    assert_eq!(Zoom(100).step(Step::Out, DEFAULT), Zoom(90));
    assert_eq!(Zoom(67).step(Step::In, DEFAULT), Zoom(75));
    assert_eq!(Zoom(300).step(Step::In, DEFAULT), Zoom(300));
    assert_eq!(Zoom(25).step(Step::Out, DEFAULT), Zoom(25));
}

#[test]
fn stepping_from_an_off_ladder_level_lands_on_the_next_step() {
    assert_eq!(Zoom(105).step(Step::In, DEFAULT), Zoom(110));
    assert_eq!(Zoom(105).step(Step::Out, DEFAULT), Zoom(100));
}

#[test]
fn reset_returns_to_the_default_zoom() {
    assert_eq!(Zoom(250).step(Step::Reset, DEFAULT), DEFAULT);
    assert_eq!(Zoom(250).step(Step::Reset, Zoom(110)), Zoom(110));
}

#[test]
fn zoom_levels_persist_per_site_and_only_overrides_are_stored() {
    let folder = tempfile::tempdir().unwrap();
    let mut zooms = Zooms::open(folder.path());
    zooms.set("example.com", Zoom(150), DEFAULT).unwrap();
    zooms.set("docs.example.com", Zoom(100), DEFAULT).unwrap();
    zooms.set("docs.example.com", DEFAULT, DEFAULT).unwrap();
    zooms.set("other.org", DEFAULT, DEFAULT).unwrap();

    let reopened = Zooms::open(folder.path());
    assert_eq!(reopened.of("example.com", DEFAULT), Zoom(150));
    assert_eq!(reopened.of("docs.example.com", DEFAULT), DEFAULT);
    assert_eq!(reopened.of("other.org", Zoom(110)), Zoom(110));
    let stored = fs::read_to_string(folder.path().join("zoom.json")).unwrap();
    assert_eq!(stored, r#"{"example.com":150}"#);
}
