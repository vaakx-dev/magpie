use std::time::Duration;

use magpie::storage::arrange::{Item, Spot};
use magpie::storage::clock::Epoch;
use magpie::storage::closed::{CAPACITY, Closed, Stack};
use magpie::storage::session::{
    Direction, Folder, FolderId, Link, Pane, Pinned, Position, Section, Session, Space, SpaceId,
    Tab, TabId,
};
use magpie::webview::sleep::{Activity, IDLE_BEFORE_SLEEP};
use magpie::webview::{Audio, Visibility};

const START: Epoch = Epoch(1_800_000_000);
const MINUTE: Duration = Duration::from_secs(60);

fn tab(id: u64) -> Tab {
    let pane = Pane {
        url: format!("https://site{id}.example.com/"),
        title: format!("Site {id}"),
    };
    Tab::new(TabId(id), pane, START)
}

fn today(space: &Space) -> Vec<u64> {
    space.today.iter().map(|tab| tab.id.0).collect()
}

fn outline(items: &[Pinned]) -> String {
    let names: Vec<String> = items
        .iter()
        .map(|item| match item {
            Pinned::Tab(tab) => tab.id.0.to_string(),
            Pinned::Folder(folder) => format!("[{}]", outline(&folder.items)),
        })
        .collect();
    names.join(" ")
}

fn space() -> Space {
    let mut space = Space::new(SpaceId(1), "Work", "Work");
    let mut docs = Folder::new(FolderId(10), "Docs");
    let mut specs = Folder::new(FolderId(11), "Specs");
    specs.items.push(Pinned::Tab(tab(3)));
    docs.items.push(Pinned::Tab(tab(2)));
    docs.items.push(Pinned::Folder(specs));
    space.pinned.push(Pinned::Tab(tab(1)));
    space.pinned.push(Pinned::Folder(docs));
    space.today.extend([tab(4), tab(5), tab(6)]);
    space.active = Some(TabId(4));
    space
}

fn closed(id: u64) -> Closed {
    Closed {
        tab: tab(id),
        space: SpaceId(1),
    }
}

#[test]
fn closed_tabs_reopen_newest_first_and_only_the_most_recent_are_kept() {
    let mut stack = Stack::default();
    let pushed = CAPACITY as u64 + 2;
    (1..=pushed).for_each(|id| stack.push(closed(id)));

    let reopened: Vec<u64> = std::iter::from_fn(|| stack.take(0))
        .map(|entry| entry.tab.id.0)
        .collect();

    let expected: Vec<u64> = (3..=pushed).rev().collect();
    assert_eq!(reopened, expected);
    assert_eq!(stack.take(0), None);
}

#[test]
fn a_closed_tab_reopens_from_anywhere_in_the_stack() {
    let mut stack = Stack::default();
    (1..=4).for_each(|id| stack.push(closed(id)));

    let picked = stack.take(1).map(|entry| entry.tab.id.0);
    let left: Vec<u64> = stack.newest_first().map(|entry| entry.tab.id.0).collect();

    assert_eq!(picked, Some(3));
    assert_eq!(left, [4, 2, 1]);
    assert_eq!(stack.take(3), None);
}

#[test]
fn today_tabs_reorder_before_and_after_other_tabs() {
    let mut space = space();

    assert!(space.arrange(Item::Tab(TabId(6)), Spot::Before(Item::Tab(TabId(4)))));
    assert_eq!(today(&space), [6, 4, 5]);
    assert!(space.arrange(Item::Tab(TabId(6)), Spot::After(Item::Tab(TabId(5)))));
    assert_eq!(today(&space), [4, 5, 6]);
    assert!(space.arrange(Item::Tab(TabId(5)), Spot::Top(Section::Today)));
    assert_eq!(today(&space), [5, 4, 6]);
    assert_eq!(space.active, Some(TabId(4)));
}

#[test]
fn tabs_move_between_today_pinned_and_folders() {
    let mut space = space();

    assert!(space.arrange(Item::Tab(TabId(5)), Spot::Into(FolderId(11))));
    assert!(space.arrange(Item::Tab(TabId(1)), Spot::After(Item::Tab(TabId(2)))));
    assert!(space.arrange(Item::Tab(TabId(3)), Spot::Before(Item::Tab(TabId(6)))));

    assert_eq!(outline(&space.pinned), "[2 1 [5]]");
    assert_eq!(today(&space), [4, 3, 6]);
}

#[test]
fn folders_reorder_but_never_land_in_today_or_inside_themselves() {
    let mut space = space();
    let original = space.clone();

    assert!(!space.arrange(Item::Folder(FolderId(10)), Spot::Into(FolderId(11))));
    assert!(!space.arrange(Item::Folder(FolderId(11)), Spot::Top(Section::Today)));
    assert!(!space.arrange(
        Item::Folder(FolderId(11)),
        Spot::Before(Item::Tab(TabId(4)))
    ));
    assert!(!space.arrange(Item::Tab(TabId(4)), Spot::Before(Item::Tab(TabId(4)))));
    assert_eq!(space, original);

    assert!(space.arrange(Item::Folder(FolderId(11)), Spot::Top(Section::Pinned)));
    assert_eq!(outline(&space.pinned), "[3] 1 [2]");
}

#[test]
fn tabs_and_folders_move_to_another_space_keeping_their_section() {
    let mut session = Session {
        spaces: vec![space(), Space::new(SpaceId(2), "Home", "Personal")],
        active: Some(SpaceId(1)),
    };

    assert!(session.move_to_space(Item::Tab(TabId(4)), SpaceId(2)));
    assert!(session.move_to_space(Item::Folder(FolderId(11)), SpaceId(2)));
    assert!(!session.move_to_space(Item::Tab(TabId(4)), SpaceId(2)));
    assert!(!session.move_to_space(Item::Tab(TabId(5)), SpaceId(9)));

    let (work, home) = (&session.spaces[0], &session.spaces[1]);
    assert_eq!(today(work), [5, 6]);
    assert_eq!(work.active, Some(TabId(5)));
    assert_eq!(outline(&work.pinned), "1 [2]");
    assert_eq!(today(home), [4]);
    assert_eq!(outline(&home.pinned), "[3]");
}

const SILENT: Audio = Audio {
    playing: false,
    muted: false,
};
const PLAYING_MUTED: Audio = Audio {
    playing: true,
    muted: true,
};

fn activity(visibility: Visibility, audio: Audio, idle_minutes: u32) -> Activity {
    Activity {
        visibility,
        audio,
        unchanged_for: idle_minutes * MINUTE,
    }
}

#[test]
fn only_hidden_silent_tabs_idle_past_the_threshold_fall_asleep() {
    let half_hour = 30 * MINUTE;
    let long = 120;

    assert!(!activity(Visibility::Hidden, SILENT, 29).drowsy(half_hour));
    assert!(activity(Visibility::Hidden, SILENT, 30).drowsy(half_hour));
    assert!(activity(Visibility::Hidden, SILENT, long).drowsy(IDLE_BEFORE_SLEEP));
    assert!(!activity(Visibility::Focused, SILENT, long).drowsy(IDLE_BEFORE_SLEEP));
    assert!(!activity(Visibility::Shown, SILENT, long).drowsy(IDLE_BEFORE_SLEEP));
    assert!(!activity(Visibility::Hidden, PLAYING_MUTED, long).drowsy(IDLE_BEFORE_SLEEP));
}

fn favorite_titles(space: &Space) -> String {
    let titles: Vec<&str> = space
        .favorites
        .iter()
        .map(|link| link.title.as_str())
        .collect();
    titles.join(" ")
}

#[test]
fn favorites_move_to_the_slot_they_are_dropped_on() {
    let mut space = Space::new(SpaceId(1), "Work", "Work");
    space.favorites = ["A", "B", "C", "D"]
        .map(|title| Link {
            url: format!("https://{title}.example.com/"),
            title: title.to_owned(),
        })
        .to_vec();

    assert!(space.move_favorite(0, 3));
    assert_eq!(favorite_titles(&space), "B C A D");
    assert!(space.move_favorite(3, 0));
    assert_eq!(favorite_titles(&space), "D B C A");
    assert!(space.move_favorite(1, 4));
    assert_eq!(favorite_titles(&space), "D C A B");
    assert!(!space.move_favorite(2, 2));
    assert!(!space.move_favorite(2, 3));
    assert!(!space.move_favorite(4, 0));
    assert!(!space.move_favorite(0, 5));
    assert_eq!(favorite_titles(&space), "D C A B");
}

#[test]
fn tab_cycling_walks_pinned_then_today_tabs_and_wraps_at_both_ends() {
    let mut space = space();

    assert_eq!(space.cycled(Direction::Forward), Some(TabId(5)));
    assert_eq!(space.cycled(Direction::Backward), Some(TabId(3)));
    space.active = Some(TabId(6));
    assert_eq!(space.cycled(Direction::Forward), Some(TabId(1)));
    space.active = Some(TabId(1));
    assert_eq!(space.cycled(Direction::Backward), Some(TabId(6)));
    space.active = None;
    assert_eq!(space.cycled(Direction::Forward), Some(TabId(1)));
    assert_eq!(space.cycled(Direction::Backward), Some(TabId(6)));
}

#[test]
fn numbered_tabs_count_from_the_top_and_the_last_one_is_always_reachable() {
    let space = space();

    assert_eq!(space.tab_at(Position::Index(0)), Some(TabId(1)));
    assert_eq!(space.tab_at(Position::Index(2)), Some(TabId(3)));
    assert_eq!(space.tab_at(Position::Index(5)), Some(TabId(6)));
    assert_eq!(space.tab_at(Position::Index(7)), None);
    assert_eq!(space.tab_at(Position::Last), Some(TabId(6)));
}

#[test]
fn an_empty_space_has_no_tab_to_cycle_or_jump_to() {
    let space = Space::new(SpaceId(1), "Work", "Work");

    assert_eq!(space.cycled(Direction::Forward), None);
    assert_eq!(space.cycled(Direction::Backward), None);
    assert_eq!(space.tab_at(Position::Index(0)), None);
    assert_eq!(space.tab_at(Position::Last), None);
}
