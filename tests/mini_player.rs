use magpie::storage::session::TabId;
use magpie::ui::mini_player::{Candidate, Player};
use magpie::webview::media::{Playback, Track, track};

const VIDEO: TabId = TabId(1);
const MUSIC: TabId = TabId(2);
const ARTICLE: TabId = TabId(3);

fn playing(tab: TabId) -> Candidate {
    Candidate { tab, playing: true }
}

fn paused(tab: TabId) -> Candidate {
    Candidate {
        tab,
        playing: false,
    }
}

#[test]
fn shows_the_background_tab_that_is_playing() {
    let player = Player::default();
    let tabs = [playing(VIDEO), paused(ARTICLE), playing(MUSIC)];

    assert_eq!(player.shown(&tabs, Some(VIDEO)), Some(MUSIC));
    assert_eq!(player.shown(&tabs, Some(ARTICLE)), Some(VIDEO));
    assert_eq!(player.shown(&[paused(MUSIC)], Some(ARTICLE)), None);
}

#[test]
fn hides_while_the_playing_tab_is_active() {
    let player = Player::default();
    let tabs = [playing(MUSIC), paused(ARTICLE)];

    assert_eq!(player.shown(&tabs, Some(ARTICLE)), Some(MUSIC));
    assert_eq!(player.shown(&tabs, Some(MUSIC)), None);
}

#[test]
fn keeps_a_tab_paused_from_the_mini_player_until_released() {
    let mut player = Player::default();
    player.hold(MUSIC);

    assert_eq!(player.shown(&[paused(MUSIC)], Some(ARTICLE)), Some(MUSIC));
    assert_eq!(
        player.shown(&[paused(MUSIC), playing(VIDEO)], Some(ARTICLE)),
        Some(VIDEO),
        "another tab starting to play takes over from a paused one",
    );
    assert_eq!(
        player.shown(&[playing(VIDEO), playing(MUSIC)], Some(ARTICLE)),
        Some(MUSIC),
        "the tab being controlled stays shown once it plays again",
    );

    player.release();
    assert_eq!(player.shown(&[paused(MUSIC)], Some(ARTICLE)), None);
}

#[test]
fn opening_the_held_tab_releases_it() {
    let mut player = Player::default();
    player.hold(MUSIC);
    player.activated(ARTICLE);
    assert_eq!(player.shown(&[paused(MUSIC)], Some(ARTICLE)), Some(MUSIC));

    player.activated(MUSIC);
    assert_eq!(player.shown(&[paused(MUSIC)], Some(ARTICLE)), None);
}

#[test]
fn clears_when_the_tab_closes() {
    let mut player = Player::default();
    player.hold(MUSIC);

    assert_eq!(player.shown(&[paused(ARTICLE)], Some(ARTICLE)), None);
}

#[test]
fn reads_media_messages_and_ignores_other_page_messages() {
    let message =
        r#"{"kind":"media","title":"Night Drive","artist":"Example Band","state":"playing"}"#;
    let expected = Track {
        title: "Night Drive".to_owned(),
        artist: "Example Band".to_owned(),
        state: Playback::Playing,
    };

    assert_eq!(track(message), Some(expected));
    assert_eq!(
        track(r#"{"kind":"media","title":"Clip","artist":"","state":"paused"}"#)
            .map(|track| track.state),
        Some(Playback::Paused),
    );
    assert_eq!(track(r#"{"kind":"page"}"#), None);
    assert_eq!(
        track(r#"{"kind":"background","url":"https://example.com/"}"#),
        None
    );
    assert_eq!(
        track(r#"{"kind":"media","title":"Clip","state":"stopped"}"#),
        None
    );
    assert_eq!(track("\"hidden{display:none}\""), None);
}
