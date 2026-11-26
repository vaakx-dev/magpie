mod common;

use magpie::{
    locale::Locale,
    reader::{Config, Font, TextSize},
};
use serde_json::json;

use common::node_accepts;

fn message(kind: &str, font: &str, size: u8) -> String {
    json!({ "kind": kind, "font": font, "size": size }).to_string()
}

#[test]
fn reads_the_style_a_reader_page_reports() {
    let parsed = Config::parse(&message("magpie_reader", "Serif", 22));

    assert_eq!(
        parsed,
        Some(Config {
            font: Font::Serif,
            size: TextSize(22),
        })
    );
}

#[test]
fn ignores_other_messages_and_sizes_it_does_not_offer() {
    assert_eq!(Config::parse(&message("magpie_reader", "Serif", 90)), None);
    assert_eq!(Config::parse(&message("magpie_reader", "Comic", 18)), None);
    assert_eq!(Config::parse(&message("other", "Sans", 18)), None);
}

#[test]
fn the_injected_script_is_valid_javascript() {
    let script = Config::default().script(Locale::Japanese);

    assert!(node_accepts(script.as_bytes()));
}
