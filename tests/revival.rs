use std::time::{Duration, Instant};

use magpie::webview::Visibility;
use magpie::webview::revival::{Revival, Revive};

#[test]
fn visible_tab_reloads_at_once_and_hidden_tab_waits_until_shown() {
    let start = Instant::now();
    let mut visible = Revival::default();
    let mut hidden = Revival::default();

    assert_eq!(visible.ended(start, Visibility::Focused), Revive::Now);
    assert_eq!(hidden.ended(start, Visibility::Hidden), Revive::WhenShown);
}

#[test]
fn crash_loop_stops_reloading_until_the_tab_is_shown_again() {
    let start = Instant::now();
    let mut revival = Revival::default();

    assert_eq!(revival.ended(start, Visibility::Shown), Revive::Now);
    let again = start + Duration::from_secs(5);
    assert_eq!(revival.ended(again, Visibility::Shown), Revive::WhenShown);
    let calm = again + Duration::from_secs(60);
    assert_eq!(revival.ended(calm, Visibility::Shown), Revive::Now);
}
