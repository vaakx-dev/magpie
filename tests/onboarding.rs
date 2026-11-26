use std::fs;

use magpie::config::{Config, Switch, Theme};

#[test]
fn a_fresh_profile_onboards_until_it_finishes() {
    let folder = tempfile::tempdir().unwrap();
    let mut config = Config::load(folder.path());
    assert_eq!(config.settings.onboarding_done, Switch::Off);

    config.settings.theme = Theme::Dark;
    config.save().unwrap();
    let mut config = Config::load(folder.path());
    assert_eq!(config.settings.onboarding_done, Switch::Off);

    config.settings.onboarding_done = Switch::On;
    config.save().unwrap();
    assert_eq!(
        Config::load(folder.path()).settings.onboarding_done,
        Switch::On
    );
}

#[test]
fn profiles_saved_before_onboarding_existed_skip_it() {
    for file in ["settings.conf", "layout.conf"] {
        let folder = tempfile::tempdir().unwrap();
        fs::write(folder.path().join(file), "theme = Light\n").unwrap();
        let settings = Config::load(folder.path()).settings;
        assert_eq!(settings.onboarding_done, Switch::On, "{file}");
        assert_eq!(settings.theme, Theme::Light, "{file}");
    }
}
