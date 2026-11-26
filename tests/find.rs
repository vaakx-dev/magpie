use magpie::find::Matches;

#[test]
fn counter_shows_the_active_match_out_of_the_total() {
    assert_eq!(Matches::new(3, 12).label(), "3 / 12");
    assert_eq!(Matches::new(1, 1).label(), "1 / 1");
}

#[test]
fn counter_reports_no_matches_and_no_active_match() {
    assert_eq!(Matches::new(-1, 0).label(), "No matches");
    assert_eq!(Matches::new(-1, 5).label(), "0 / 5");
    assert_eq!(Matches::new(0, 5).label(), "0 / 5");
}
