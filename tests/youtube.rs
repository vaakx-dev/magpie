use std::{path::Path, process::Command};

use serde_json::{Value, json};

const AD_FIELDS: [&str; 4] = [
    "adPlacements",
    "playerAds",
    "adSlots",
    "adBreakHeartbeatParams",
];

fn responses_seen_on(hostname: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("node")
        .arg(root.join("tests/youtube/harness.cjs"))
        .arg(root.join("crates/magpie-engine/src/blocker/youtube.js"))
        .arg(hostname)
        .output()
        .expect("node runs the YouTube script harness");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn has_ads(response: &Value) -> bool {
    AD_FIELDS.iter().any(|field| response.get(field).is_some())
}

#[test]
fn strips_ads_from_youtube_player_responses_before_the_player_reads_them() {
    let seen = responses_seen_on("www.youtube.com");
    let video = json!({ "videoId": "abc" });

    assert!(!has_ads(&seen["initial"]));
    assert_eq!(seen["initial"]["videoDetails"], video);
    assert!(!has_ads(&seen["fetched"]));
    assert_eq!(seen["fetched"]["videoDetails"], video);
    assert!(!has_ads(&seen["watched"]["playerResponse"]));
    assert_eq!(seen["watched"]["playerResponse"]["videoDetails"], video);
    let navigated = &seen["navigated"];
    assert!(!has_ads(&navigated[0]["playerResponse"]));
    assert!(!has_ads(&navigated[1]["response"]["playerResponse"]));
    assert_eq!(navigated[0]["playerResponse"]["videoDetails"], video);
    assert!(has_ads(&seen["browsed"]));
}

#[test]
fn leaves_responses_alone_off_youtube() {
    let seen = responses_seen_on("www.notyoutube.com");

    assert!(has_ads(&seen["initial"]));
    assert!(has_ads(&seen["fetched"]));
    assert!(has_ads(&seen["watched"]["playerResponse"]));
    assert!(has_ads(&seen["navigated"][0]["playerResponse"]));
}
