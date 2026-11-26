use magpie::ui::field::buffer::{Buffer, Motion};

const ADDRESS: &str = "https://example.com/path";

#[test]
fn starts_with_everything_selected_so_typing_replaces_it() {
    let mut buffer = Buffer::new(ADDRESS);
    assert_eq!(buffer.selected(), ADDRESS);
    buffer.insert("news");
    assert_eq!(buffer.text(), "news");
    assert_eq!(buffer.cursor(), 4);
    assert!(buffer.selection().is_empty());
}

#[test]
fn shift_motions_extend_from_the_anchor_in_either_direction() {
    let mut buffer = Buffer::new("hello world");
    buffer.move_to(5);
    buffer.select_by(Motion::Right);
    buffer.select_by(Motion::Right);
    assert_eq!(buffer.selected(), " w");
    buffer.select_by(Motion::Home);
    assert_eq!(buffer.selected(), "hello");
    assert!(buffer.reversed());
    buffer.select_by(Motion::End);
    assert_eq!(buffer.selected(), " world");
    assert!(!buffer.reversed());
}

#[test]
fn plain_motions_collapse_a_selection_to_its_edge() {
    let mut buffer = Buffer::new("hello world");
    buffer.move_to(2);
    buffer.select_to(8);
    buffer.move_by(Motion::Left);
    assert_eq!(buffer.selection(), 2..2);
    buffer.select_to(8);
    buffer.move_by(Motion::Right);
    assert_eq!(buffer.selection(), 8..8);
}

#[test]
fn backspace_and_delete_remove_the_selection_or_one_character() {
    let mut buffer = Buffer::new("héllo");
    buffer.move_to(3);
    buffer.backspace();
    assert_eq!(buffer.text(), "hllo");
    buffer.delete();
    assert_eq!(buffer.text(), "hlo");
    buffer.select_to(3);
    buffer.backspace();
    assert_eq!(buffer.text(), "h");
    buffer.select_all();
    buffer.delete();
    assert_eq!(buffer.text(), "");
}

#[test]
fn a_backwards_drag_selection_is_replaced_by_typing() {
    let mut buffer = Buffer::new("hello world");
    buffer.move_to(11);
    buffer.select_to(6);
    buffer.insert("there");
    assert_eq!(buffer.text(), "hello there");
}

#[test]
fn word_selection_takes_the_run_of_letters_around_the_click() {
    let mut buffer = Buffer::new(ADDRESS);
    buffer.select_word_at(10);
    assert_eq!(buffer.selected(), "example");
    buffer.select_word_at(ADDRESS.len());
    assert_eq!(buffer.selected(), "path");
    buffer.select_word_at(0);
    assert_eq!(buffer.selected(), "https");
}
