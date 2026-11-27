mod common;

use magpie::{
    config::Switch,
    darkmode::{Config, Mode, Overrides},
};
use url::Url;

use common::node_accepts;

fn url(text: &str) -> Url {
    Url::parse(text).unwrap()
}

fn enabled() -> Config {
    let mut config = Config::default();
    config.enabled = Switch::On;
    config
}

#[test]
fn excluding_a_site_covers_its_www_address_and_every_page() {
    let mut config = enabled();
    config.toggle(&url("https://www.news.example/story"));

    assert_eq!(config.mode(&url("https://news.example/other")), Mode::Off);
    assert_eq!(config.mode(&url("http://www.news.example/")), Mode::Off);
    assert_eq!(
        config.mode(&url("https://sports.news.example/")),
        Mode::Auto
    );
    assert_eq!(config.mode(&url("https://shop.example/")), Mode::Auto);
}

#[test]
fn never_darkens_magpie_or_local_pages() {
    let config = enabled();

    assert_eq!(config.mode(&url("magpie://settings")), Mode::Off);
    assert_eq!(config.mode(&url("file:///C:/notes.html")), Mode::Off);
}

#[test]
fn toggling_a_site_twice_leaves_no_exception() {
    let page = url("https://docs.example/guide");
    for mut config in [enabled(), Config::default()] {
        config.toggle(&page);
        config.toggle(&page);

        assert!(config.exceptions().is_empty());
    }
}

#[test]
fn turning_a_site_on_while_switched_off_forces_it_dark() {
    let mut config = Config::default();
    config.toggle(&url("https://bright.example/"));

    assert_eq!(
        config.mode(&url("https://bright.example/page")),
        Mode::Forced
    );
    assert_eq!(config.mode(&url("https://other.example/")), Mode::Off);
}

#[test]
fn overrides_flip_a_site_on_top_of_the_saved_sites() {
    let mut saved = enabled();
    saved.toggle(&url("https://excluded.example/"));
    let mut overrides = Overrides::default();

    overrides.toggle(&saved, &url("https://excluded.example/page"));
    overrides.toggle(&saved, &url("https://private.example/"));
    let shown = saved.overridden(&overrides);

    assert_eq!(shown.mode(&url("https://excluded.example/")), Mode::Auto);
    assert_eq!(shown.mode(&url("https://private.example/")), Mode::Off);

    overrides.toggle(&saved, &url("https://private.example/"));
    let shown = saved.overridden(&overrides);
    assert_eq!(shown.mode(&url("https://private.example/")), Mode::Auto);
}

#[test]
fn the_injected_script_is_valid_javascript() {
    let mut config = enabled();
    config.toggle(&url("https://excluded.example/"));

    assert!(node_accepts(config.script().as_bytes()));
}
