use magpie_engine::tracking::{stripped, trimmed_referrer};
use url::Url;

fn strip(address: &str) -> Option<String> {
    stripped(&Url::parse(address).unwrap()).map(String::from)
}

fn referrer(from: &str, to: &str) -> Option<String> {
    trimmed_referrer(&Url::parse(from).unwrap(), &Url::parse(to).unwrap())
}

#[test]
fn strips_tracking_tags_and_keeps_the_rest_of_the_address() {
    assert_eq!(
        strip("https://shop.example/item?id=42&utm_source=mail&utm_campaign=spring&color=red%20blue#reviews")
            .as_deref(),
        Some("https://shop.example/item?id=42&color=red%20blue#reviews")
    );
    assert_eq!(
        strip("https://news.example/story?fbclid=abc&gclid=def&msclkid=ghi").as_deref(),
        Some("https://news.example/story")
    );
    assert_eq!(
        strip("https://www.youtube.com/watch?v=dQw4w9WgXcQ&si=share123").as_deref(),
        Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
    );
    let untouched = [
        "https://search.example/find?q=utm_source&page=2",
        "https://docs.example/page?si=units",
        "https://plain.example/path",
        "https://youtube.com.evil.example/watch?si=kept",
        "data:text/plain,utm_source=x",
    ];
    for address in untouched {
        assert_eq!(strip(address), None, "{address}");
    }
}

#[test]
fn cross_origin_referrers_shrink_to_the_origin() {
    assert_eq!(
        referrer(
            "https://mail.example/inbox/message?id=7",
            "https://cdn.other.example/pixel.gif"
        )
        .as_deref(),
        Some("https://mail.example/")
    );
    assert_eq!(
        referrer(
            "https://mail.example/inbox?id=7",
            "https://mail.example/api/next"
        ),
        None
    );
    assert_eq!(
        referrer("https://mail.example/", "https://other.example/"),
        None
    );
    assert_eq!(
        referrer(
            "https://mail.example:8443/inbox",
            "https://mail.example/inbox"
        )
        .as_deref(),
        Some("https://mail.example:8443/")
    );
}
