use magpie::config::{Config, Settings};
use magpie_engine::focus::{Focus, Rule, Site};
use url::Url;

fn url(text: &str) -> Url {
    Url::parse(text).unwrap()
}

fn redirected(focus: &Focus, text: &str) -> Option<String> {
    focus.redirect(&url(text)).map(String::from)
}

#[test]
fn shorts_open_as_normal_videos_on_desktop_and_mobile_youtube() {
    let focus = Focus::default();

    assert_eq!(
        redirected(
            &focus,
            "https://www.youtube.com/shorts/abc123?feature=share"
        ),
        Some("https://www.youtube.com/watch?v=abc123".to_owned())
    );
    assert_eq!(
        redirected(&focus, "https://m.youtube.com/shorts/abc123/"),
        Some("https://m.youtube.com/watch?v=abc123".to_owned())
    );
    assert_eq!(redirected(&focus, "https://www.youtube.com/shorts/"), None);
    assert_eq!(
        redirected(&focus, "https://www.youtube.com/@channel/shorts"),
        None
    );
    assert_eq!(
        redirected(&focus, "https://example.com/shorts/abc123"),
        None
    );
}

#[test]
fn shorts_stay_shorts_once_the_blocker_is_off() {
    let mut focus = Focus::default();
    focus.allow(Rule::YouTubeShorts);

    assert_eq!(
        redirected(&focus, "https://www.youtube.com/shorts/abc123"),
        None
    );
}

#[test]
fn sites_match_their_desktop_and_mobile_hosts_only() {
    let cases = [
        ("youtube.com", Some(Site::YouTube)),
        ("www.youtube.com", Some(Site::YouTube)),
        ("m.youtube.com", Some(Site::YouTube)),
        ("notyoutube.com", None),
        ("youtube.com.example.org", None),
        ("www.instagram.com", Some(Site::Instagram)),
        ("m.facebook.com", Some(Site::Facebook)),
        ("mobile.twitter.com", Some(Site::X)),
        ("x.com", Some(Site::X)),
        ("www.tiktok.com", Some(Site::TikTok)),
        ("old.reddit.com", Some(Site::Reddit)),
    ];

    for (host, site) in cases {
        assert_eq!(Site::of(host), site, "{host}");
    }
}

#[test]
fn each_toggle_hides_only_its_own_site() {
    let mut focus = Focus::default();
    for rule in Rule::ALL {
        focus.allow(rule);
    }
    let empty = Site::ALL.map(|site| focus.stylesheet(site));
    assert!(empty.iter().all(String::is_empty));

    for rule in Rule::ALL {
        let mut only = focus.clone();
        only.block(rule);
        for site in Site::ALL {
            let css = only.stylesheet(site);
            assert_eq!(css.is_empty(), site != rule.site(), "{rule:?} on {site:?}");
            assert!(
                css.lines()
                    .all(|line| line.ends_with("{display:none !important}"))
            );
        }
    }
}

#[test]
fn youtube_toggles_add_to_one_shared_stylesheet() {
    let mut focus = Focus::default();
    let defaults = focus.stylesheet(Site::YouTube);
    focus.block(Rule::YouTubeComments);
    let with_comments = focus.stylesheet(Site::YouTube);
    focus.allow(Rule::YouTubeShorts);
    let without_shorts = focus.stylesheet(Site::YouTube);

    assert!(with_comments.len() > defaults.len());
    assert!(without_shorts.len() < with_comments.len());
    assert!(!without_shorts.contains("/shorts/"));
}

#[test]
fn fresh_settings_block_shorts_playables_and_members_only_videos() {
    let focus = Settings::default().focus;
    let blocked: Vec<Rule> = Rule::ALL
        .into_iter()
        .filter(|rule| focus.blocks(*rule))
        .collect();

    assert_eq!(
        blocked,
        [
            Rule::YouTubeShorts,
            Rule::YouTubePlayables,
            Rule::YouTubeMembersOnly
        ]
    );
}

#[test]
fn distraction_toggles_survive_a_save_and_load() {
    let folder = tempfile::tempdir().unwrap();
    let mut config = Config::load(folder.path());
    config.settings.focus.allow(Rule::YouTubeShorts);
    config.settings.focus.block(Rule::InstagramReels);
    config.settings.focus.block(Rule::RedditPopular);
    config.save().unwrap();

    let loaded = Config::load(folder.path()).settings.focus;

    assert_eq!(loaded, config.settings.focus);
    assert!(!loaded.blocks(Rule::YouTubeShorts));
    assert!(loaded.blocks(Rule::YouTubePlayables));
}

#[test]
fn hand_edited_distraction_keys_load_and_bad_values_keep_the_default() {
    let settings =
        Settings::parse("hide_x_for_you = On\nhide_youtube_shorts = maybe\nblock_tiktok = Off\n");

    assert!(settings.focus.blocks(Rule::XForYou));
    assert!(settings.focus.blocks(Rule::YouTubeShorts));
    assert!(!settings.focus.blocks(Rule::TikTok));
}

#[test]
fn a_focus_built_from_rules_blocks_exactly_those_rules() {
    let focus: Focus = [Rule::YouTubeComments, Rule::TikTok].into_iter().collect();

    let blocked: Vec<Rule> = Rule::ALL
        .into_iter()
        .filter(|rule| focus.blocks(*rule))
        .collect();
    assert_eq!(blocked, [Rule::YouTubeComments, Rule::TikTok]);
    assert_eq!(
        redirected(&focus, "https://m.youtube.com/shorts/abc123"),
        None
    );
}
