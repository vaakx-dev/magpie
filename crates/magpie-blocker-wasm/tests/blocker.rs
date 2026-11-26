use magpie_blocker::{exchange, should_block};

const PAGE: &str = "https://news.example.com/";

fn blocked(url: &str, kind: &str) -> bool {
    let request = [url, PAGE, kind, "GET"].join("\n");
    let length = request.len();
    let buffer = exchange(length);
    unsafe { std::ptr::copy_nonoverlapping(request.as_ptr(), buffer, length) };
    should_block(length)
}

#[test]
fn blocks_known_ads_before_any_snapshot_loads() {
    assert!(blocked(
        "https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js",
        "script"
    ));
    assert!(blocked(
        "https://securepubads.doubleclick.net/tag/js/gpt.js",
        "script"
    ));
    assert!(!blocked("https://news.example.com/app.js", "script"));
}
