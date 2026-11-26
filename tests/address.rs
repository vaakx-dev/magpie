use magpie::{
    address::{Destination, address, destination, upgraded},
    config::{Engine, Search},
};
use url::Url;

fn upgrade(address: &str) -> Option<String> {
    upgraded(&Url::parse(address).unwrap()).map(String::from)
}

#[test]
fn turns_typed_input_into_addresses_or_searches() {
    let search = Search::default();
    assert!(
        matches!(destination("rust-lang.org", &search), Destination::Address(url) if url.as_str() == "https://rust-lang.org/")
    );
    assert!(
        matches!(destination("localhost:8080", &search), Destination::Address(url) if url.scheme() == "http")
    );
    assert!(matches!(
        destination("magpie birds", &search),
        Destination::Search(url) if url.as_str() == "https://duckduckgo.com/?q=magpie+birds"
    ));
}

#[test]
fn typed_bookmark_addresses_gain_a_scheme_and_reject_plain_words() {
    let typed = |input: &str| address(input).map(String::from);

    assert_eq!(
        typed(" example.com "),
        Some("https://example.com/".to_owned())
    );
    assert_eq!(typed("http://x.org/a"), Some("http://x.org/a".to_owned()));
    assert_eq!(
        typed("localhost:3000"),
        Some("http://localhost:3000/".to_owned())
    );
    assert_eq!(typed("magpie birds"), None);
    assert_eq!(typed(""), None);
}

#[test]
fn keeps_local_file_addresses() {
    let search = Search::default();
    let address = "file:///C:/Users/someone/Documents/notes.html";
    assert!(matches!(
        destination(address, &search),
        Destination::Address(url) if url.as_str() == address
    ));
}

#[test]
fn opens_known_magpie_pages_and_searches_for_unknown_ones() {
    let search = Search::default();
    let settings = "magpie://settings#privacy";
    assert!(matches!(
        destination(settings, &search),
        Destination::Address(url) if url.as_str() == settings
    ));
    assert!(matches!(
        destination("magpie://nowhere", &search),
        Destination::Search(_)
    ));
}

#[test]
fn searches_use_the_chosen_engine() {
    let brave = Search {
        engine: Engine::Brave,
        custom: String::new(),
    };
    assert!(matches!(
        destination("magpie birds", &brave),
        Destination::Search(url) if url.as_str() == "https://search.brave.com/search?q=magpie+birds"
    ));
}

#[test]
fn https_only_upgrades_remote_http_and_leaves_local_addresses() {
    assert_eq!(
        upgrade("http://example.com/path?x=1").as_deref(),
        Some("https://example.com/path?x=1")
    );
    assert_eq!(
        upgrade("http://example.com:8080/").as_deref(),
        Some("https://example.com:8080/")
    );
    let untouched = [
        "https://example.com/",
        "http://localhost:3000/",
        "http://dev.localhost/",
        "http://127.0.0.1/",
        "http://192.168.1.1/",
        "http://[::1]/",
        "data:text/plain,hi",
    ];
    for address in untouched {
        assert_eq!(upgrade(address), None, "{address}");
    }
}
