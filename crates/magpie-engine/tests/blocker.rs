use std::collections::HashSet;

use magpie_engine::blocker::{Resource, Shield, filter_list};
use url::Url;

fn url(text: &str) -> Url {
    Url::parse(text).unwrap()
}

#[test]
fn captive_portal_pages_are_not_mistaken_for_filter_lists() {
    let portals = [
        "<!DOCTYPE html><html><body>Sign in to Hotel Wi-Fi</body></html>",
        "\n  <html lang=\"en\"><head><title>Airport Wi-Fi</title></head></html>",
        "\u{feff}<!doctype html>\n<title>Accept terms</title>",
        "<?xml version=\"1.0\"?><WISPAccessGatewayParam/>",
        "",
        "  \n",
    ];
    for portal in portals {
        assert!(!filter_list(portal), "{portal:?}");
    }
    assert!(filter_list(
        "[Adblock Plus 2.0]\n! Title: EasyList\n||ads.example^"
    ));
    assert!(filter_list("! Title: uBlock filters\nexample.com##.ad"));
}

#[test]
fn blocks_third_party_ad_script() {
    let shield = Shield::default();
    let page = url("https://news.example/article");
    let ad = url("https://securepubads.g.doubleclick.net/tag/js/gpt.js");
    assert!(shield.blocks(&ad, &page, Resource::Script));
}

#[test]
fn allows_first_party_content() {
    let shield = Shield::default();
    let page = url("https://news.example/article");
    let image = url("https://news.example/images/photo.jpg");
    assert!(!shield.blocks(&image, &page, Resource::Image));
}

#[test]
fn honours_exception_rules() {
    let shield = Shield::new(["||ads.example^\n@@||ads.example/allowed^".to_owned()]);
    let page = url("https://site.example/");
    assert!(shield.blocks(
        &url("https://ads.example/banner.png"),
        &page,
        Resource::Image
    ));
    assert!(!shield.blocks(
        &url("https://ads.example/allowed/x.png"),
        &page,
        Resource::Image
    ));
}

#[test]
fn pausing_a_site_allows_its_requests_and_cosmetics_until_resumed() {
    let shield = Shield::default();
    let page = url("https://www.news.example/article");
    let other = url("https://other.example/");
    let ad = url("https://securepubads.g.doubleclick.net/tag/js/gpt.js");
    let classes = HashSet::from(["adsbygoogle".to_owned()]);
    shield.pause("news.example");
    assert!(!shield.blocks(&ad, &page, Resource::Script));
    assert!(
        shield
            .class_id_selectors(&page, &classes, &HashSet::new())
            .is_empty()
    );
    assert!(shield.blocks(&ad, &other, Resource::Script));
    shield.resume("news.example");
    assert!(shield.blocks(&ad, &page, Resource::Script));
}

#[test]
fn reports_cosmetic_hide_selectors_for_present_classes() {
    let shield = Shield::default();
    let page = url("https://site.example/");
    let classes = HashSet::from(["adsbygoogle".to_owned()]);
    let selectors = shield.class_id_selectors(&page, &classes, &HashSet::new());
    assert!(selectors.iter().any(|selector| selector == ".adsbygoogle"));
}

#[test]
fn reports_site_specific_hide_selectors_only_on_that_site() {
    let shield = Shield::new(["site.example##.promo-rail".to_owned()]);
    let selectors = shield.page_selectors(&url("https://site.example/news"));
    assert_eq!(selectors, [".promo-rail"]);
    assert!(
        shield
            .page_selectors(&url("https://other.example/"))
            .is_empty()
    );
}

#[test]
fn blocks_tracker_fetches_from_the_page() {
    let shield = Shield::new(["||telemetry.example^$xmlhttprequest".to_owned()]);
    let page = url("https://site.example/");
    let beacon = url("https://telemetry.example/collect");
    assert!(shield.blocks(&beacon, &page, Resource::Fetch));
    assert!(!shield.blocks(&beacon, &page, Resource::Image));
}

#[test]
fn shield_is_active_on_youtube_unless_disabled_or_paused_there() {
    let shield = Shield::default();
    let video = url("https://www.youtube.com/watch?v=abc");
    let other = url("https://news.example/article");
    assert!(shield.active_on(&video));

    shield.pause("youtube.com");
    assert!(!shield.active_on(&video));
    assert!(shield.active_on(&other));
    shield.resume("youtube.com");

    shield.disable();
    assert!(!shield.active_on(&video));
    assert!(!shield.active_on(&other));
}

#[test]
fn disabled_shield_blocks_nothing_until_enabled_again() {
    let shield = Shield::default();
    let page = url("https://news.example/article");
    let ad = url("https://securepubads.g.doubleclick.net/tag/js/gpt.js");

    shield.disable();
    assert!(!shield.blocks(&ad, &page, Resource::Script));
    shield.enable();
    assert!(shield.blocks(&ad, &page, Resource::Script));
}
