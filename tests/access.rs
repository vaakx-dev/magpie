use magpie::internal::access::{Initiator, allowed, requested};
use url::Url;

fn url(address: &str) -> Url {
    Url::parse(address).unwrap()
}

fn initiator(source: &str) -> Initiator {
    Initiator::page(Some(&url(source)))
}

#[test]
fn web_pages_cannot_reach_internal_pages() {
    let web = initiator("https://example.com/attack");
    for target in [
        "magpie://settings",
        "MAGPIE://Settings#import",
        "magpie://newtab/",
    ] {
        assert!(!allowed(web, &url(target)), "{target}");
    }
    let local = initiator("file:///C:/page.html");
    assert!(!allowed(local, &url("magpie://history")));
    assert!(!allowed(Initiator::page(None), &url("magpie://downloads")));
}

#[test]
fn web_pages_reach_other_web_addresses() {
    let web = initiator("https://example.com/");
    assert!(allowed(web, &url("https://other.example/")));
    assert!(allowed(web, &url("about:blank")));
}

#[test]
fn internal_pages_and_the_browser_reach_internal_pages() {
    let bookmarks = initiator("magpie://bookmarks");
    assert!(allowed(bookmarks, &url("magpie://settings#import")));
    assert!(allowed(Initiator::Browser, &url("magpie://settings")));
    assert!(allowed(
        initiator("about:blank"),
        &url("https://example.com/")
    ));
}

#[test]
fn requested_address_matches_its_reported_form() {
    let settings = url("magpie://settings#import");
    assert!(requested(&settings, &url("magpie://Settings/#import")));
    assert!(!requested(&settings, &url("magpie://history/")));
    assert!(!requested(&settings, &url("magpie://settings/#appearance")));
}
