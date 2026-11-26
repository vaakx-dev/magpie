use std::time::{Duration, Instant};

use magpie::sponsorblock::{
    Action, Cache, Categories, Category, Channel, Config, Reply, Request, Segment, Skip, VideoId,
    api, skips,
};
use url::Url;

fn video(id: &str) -> VideoId {
    VideoId::parse(id).unwrap()
}

fn id_from(address: &str) -> Option<String> {
    VideoId::from_url(&Url::parse(address).unwrap()).map(|video| video.to_string())
}

fn segment(start_secs: f64, end_secs: f64, category: Category, action: Action) -> Segment {
    Segment {
        start_secs,
        end_secs,
        category,
        action,
        uuid: format!("{category:?}-{start_secs}"),
        votes: 0,
    }
}

#[test]
fn hash_prefix_is_the_first_four_hex_digits_of_the_sha256_of_the_id() {
    assert_eq!(video("dQw4w9WgXcQ").hash_prefix(), "5f6b");
}

#[test]
fn address_sends_only_the_hash_prefix_and_chosen_categories() {
    let categories: Categories = [Category::Sponsor, Category::MusicOfftopic]
        .into_iter()
        .collect();
    let address = api::address(&video("dQw4w9WgXcQ"), categories);
    assert_eq!(
        address,
        "https://sponsor.ajay.app/api/skipSegments/5f6b?categories=%5B%22sponsor%22%2C%22music_offtopic%22%5D"
    );
    assert!(!address.contains("dQw4w9WgXcQ"));
}

#[test]
fn finds_video_ids_in_every_youtube_url_shape() {
    let id = Some("dQw4w9WgXcQ".to_owned());
    for address in [
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        "https://m.youtube.com/watch?feature=share&v=dQw4w9WgXcQ&t=42",
        "https://music.youtube.com/watch?v=dQw4w9WgXcQ&list=RD",
        "https://www.youtube.com/shorts/dQw4w9WgXcQ",
        "https://www.youtube.com/embed/dQw4w9WgXcQ?start=10",
        "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ",
        "https://www.youtube.com/live/dQw4w9WgXcQ",
        "https://youtu.be/dQw4w9WgXcQ?si=abc",
    ] {
        assert_eq!(id_from(address), id, "{address}");
    }
    for address in [
        "https://www.youtube.com/",
        "https://www.youtube.com/@channel",
        "https://www.youtube.com/watch?v=short",
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ%3Cscript",
        "https://example.com/watch?v=dQw4w9WgXcQ",
        "https://youtu.be/",
    ] {
        assert_eq!(id_from(address), None, "{address}");
    }
}

#[test]
fn parse_keeps_only_known_segments_of_the_requested_video() {
    let body = r#"[
        {"videoID": "otherVideo1", "hash": "1917aa", "segments": [
            {"category": "sponsor", "actionType": "skip", "segment": [1, 2], "UUID": "x", "votes": 1}
        ]},
        {"videoID": "dQw4w9WgXcQ", "hash": "1917bb", "segments": [
            {"category": "sponsor", "actionType": "skip", "segment": [10.5, 30], "UUID": "a", "votes": 4, "locked": 1},
            {"category": "filler", "actionType": "mute", "segment": [40, 50], "UUID": "b"},
            {"category": "chapter", "actionType": "chapter", "segment": [0, 5], "UUID": "c", "votes": 0},
            {"category": "intro", "actionType": "skip", "segment": [9, 3], "UUID": "d", "votes": 0}
        ]}
    ]"#;
    let segments = api::parse(body, &video("dQw4w9WgXcQ")).unwrap();
    assert_eq!(
        segments,
        vec![
            Segment {
                start_secs: 10.5,
                end_secs: 30.0,
                category: Category::Sponsor,
                action: Action::Skip,
                uuid: "a".to_owned(),
                votes: 4,
            },
            Segment {
                start_secs: 40.0,
                end_secs: 50.0,
                category: Category::Filler,
                action: Action::Mute,
                uuid: "b".to_owned(),
                votes: 0,
            },
        ]
    );
    assert!(api::parse("<html>", &video("dQw4w9WgXcQ")).is_err());
}

#[test]
fn skips_merge_overlapping_skip_segments_and_ignore_other_actions() {
    let segments = [
        segment(100.0, 120.0, Category::Outro, Action::Skip),
        segment(10.0, 20.0, Category::Sponsor, Action::Skip),
        segment(15.0, 25.0, Category::SelfPromo, Action::Skip),
        segment(25.0, 30.0, Category::Interaction, Action::Skip),
        segment(50.0, 60.0, Category::Filler, Action::Mute),
        segment(70.0, 70.0, Category::Sponsor, Action::Poi),
    ];
    assert_eq!(
        skips(&segments),
        vec![
            Skip {
                start_secs: 10.0,
                end_secs: 30.0,
                category: Category::Sponsor,
            },
            Skip {
                start_secs: 100.0,
                end_secs: 120.0,
                category: Category::Outro,
            },
        ]
    );
}

#[test]
fn cache_expires_entries_and_evicts_the_least_recently_used() {
    let lifetime = Duration::from_secs(60);
    let mut cache = Cache::new(2, lifetime);
    let start = Instant::now();
    let categories = Categories::default();
    let key = |id: &str| (video(id), categories);
    let found = vec![segment(1.0, 2.0, Category::Sponsor, Action::Skip)];

    cache.insert(key("aaaaaaaaaaa"), found.clone(), start);
    cache.insert(key("bbbbbbbbbbb"), Vec::new(), start);
    assert_eq!(cache.get(&key("aaaaaaaaaaa"), start), Some(found.clone()));
    assert_eq!(
        cache.get(&(video("aaaaaaaaaaa"), Categories::NONE), start),
        None
    );

    cache.insert(key("ccccccccccc"), Vec::new(), start);
    assert_eq!(cache.get(&key("bbbbbbbbbbb"), start), None);
    assert_eq!(cache.get(&key("aaaaaaaaaaa"), start), Some(found));
    assert_eq!(cache.get(&key("ccccccccccc"), start + lifetime), None);
}

#[test]
fn config_survives_a_save_and_load() {
    let mut categories = Categories::default();
    categories.remove(Category::Interaction);
    categories.insert(Category::Filler);
    let config = Config {
        enabled: false,
        categories,
        show_skip_notice: false,
        channel_allowlist: vec!["UCabc".to_owned(), "Some Channel, Inc.".to_owned()],
    };
    assert_eq!(Config::parse(&config.text()), config);
}

#[test]
fn hand_edited_config_keeps_valid_lines_and_defaults_the_rest() {
    let config = Config::parse(
        "intro = ON
sponsor = maybe
enabled=off
unknown = On
not a setting
",
    );
    let expected: Categories = [
        Category::Sponsor,
        Category::SelfPromo,
        Category::Interaction,
        Category::Intro,
    ]
    .into_iter()
    .collect();
    assert_eq!(
        config,
        Config {
            enabled: false,
            categories: expected,
            ..Config::default()
        }
    );
}

#[test]
fn allowlisted_channels_are_exempt_by_id_or_name() {
    let config = Config {
        channel_allowlist: vec!["UCabc".to_owned(), "Friendly Channel".to_owned()],
        ..Config::default()
    };
    let channel = |id: &str, name: &str| Channel {
        id: Some(id.to_owned()),
        name: Some(name.to_owned()),
    };
    assert!(config.exempts(&channel("UCabc", "Anyone")));
    assert!(config.exempts(&channel("UCxyz", "friendly channel")));
    assert!(!config.exempts(&channel("UCxyz", "Stranger")));
    assert!(!config.exempts(&Channel::default()));
}

#[test]
fn bridge_reads_page_requests_and_writes_replies_the_script_understands() {
    let request = Request::parse(
        r#"{"kind":"magpie_sponsorblock_request","request":3,"address":"https://www.youtube.com/watch?v=dQw4w9WgXcQ","channel_id":"UCabc","channel_name":null}"#,
    )
    .unwrap();
    assert_eq!(request.number, 3);
    assert_eq!(request.video, video("dQw4w9WgXcQ"));
    assert_eq!(request.channel.id.as_deref(), Some("UCabc"));
    assert!(Request::parse(r#"{"kind":"page"}"#).is_none());
    assert!(
        Request::parse(r#"{"kind":"magpie_sponsorblock_request","request":4,"address":"https://www.youtube.com/","channel_id":null,"channel_name":null}"#)
            .is_none()
    );

    let reply = Reply {
        request: 3,
        notice: true,
        skips: vec![Skip {
            start_secs: 10.0,
            end_secs: 30.5,
            category: Category::Sponsor,
        }],
    };
    let json: serde_json::Value = serde_json::from_str(&reply.json()).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "kind": "magpie_sponsorblock_segments",
            "request": 3,
            "notice": true,
            "skips": [{"start": 10.0, "end": 30.5, "label": "Sponsor", "color": "#00d400"}],
        })
    );
}
