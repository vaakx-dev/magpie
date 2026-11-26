use gpui::EntityId;
use magpie::storage::session::TabId;
use magpie::ui::fullscreen::{Fullscreen, Page};

fn page(tab: u64, view: u64) -> Page {
    Page {
        tab: TabId(tab),
        view: EntityId::from(view),
    }
}

#[test]
fn video_fullscreen_inside_browser_fullscreen_leaves_browser_fullscreen_on() {
    let mut fullscreen = Fullscreen::default();
    let video = page(1, 10);

    fullscreen.toggle_window();
    fullscreen.enter_page(video);
    fullscreen.exit_page(video.view);
    assert!(fullscreen.active());

    fullscreen.toggle_window();
    assert!(!fullscreen.active());
}

#[test]
fn browser_fullscreen_outlives_a_video_that_entered_first() {
    let mut fullscreen = Fullscreen::default();
    let video = page(1, 10);

    fullscreen.enter_page(video);
    fullscreen.toggle_window();
    fullscreen.exit_page(video.view);
    assert!(fullscreen.active());

    fullscreen.toggle_window();
    assert!(!fullscreen.active());
}

#[test]
fn only_the_fullscreen_pane_of_a_split_shows_until_it_exits() {
    let mut fullscreen = Fullscreen::default();
    let (left, right) = (page(1, 10), page(1, 11));

    fullscreen.enter_page(right);
    assert!(fullscreen.shows(right.view));
    assert!(!fullscreen.shows(left.view));

    fullscreen.exit_page(right.view);
    assert!(fullscreen.shows(left.view));
    assert!(!fullscreen.active());
}

#[test]
fn switching_away_from_the_fullscreen_tab_exits_video_fullscreen_once() {
    let mut fullscreen = Fullscreen::default();
    let video = page(1, 10);
    fullscreen.enter_page(video);

    assert_eq!(fullscreen.follow(Some(TabId(1))), None);
    assert!(fullscreen.active());

    assert_eq!(fullscreen.follow(Some(TabId(2))), Some(video));
    assert!(!fullscreen.active());
    assert_eq!(fullscreen.follow(Some(TabId(1))), None);
    assert!(!fullscreen.active());
}

#[test]
fn closing_the_fullscreen_tab_exits_video_but_keeps_browser_fullscreen() {
    let mut fullscreen = Fullscreen::default();
    let video = page(1, 10);
    fullscreen.toggle_window();
    fullscreen.enter_page(video);

    assert_eq!(fullscreen.follow(None), Some(video));
    assert!(fullscreen.shows(page(2, 20).view));
    assert!(fullscreen.active());
}

#[test]
fn a_stale_exit_from_another_page_keeps_the_current_video_fullscreen() {
    let mut fullscreen = Fullscreen::default();
    let (old, new) = (page(1, 10), page(2, 20));
    fullscreen.enter_page(old);
    fullscreen.follow(Some(TabId(2)));

    assert_eq!(fullscreen.enter_page(new), None);
    fullscreen.exit_page(old.view);
    assert!(fullscreen.active());
    assert!(!fullscreen.shows(old.view));
}
