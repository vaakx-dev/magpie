use std::{fs, path::PathBuf};

use magpie::{
    reader::{self, Font, TextSize},
    zoom::Zoom,
};
use magpie_engine::{
    sponsorblock::{self, Category},
    update::{Channel, Repo},
};
use url::Url;

use magpie::config::{
    ArchiveAfter, Choosable, Config, ControlStyle, Controls, Engine, FavoritesColumns,
    FavoritesSize, Language, Scheme, Search, Settings, Side, Sidebar, SidebarWidth, Startup,
    Switch, Theme, TrackingPrevention, Updates,
};

fn custom_settings() -> Settings {
    let mut settings = Settings {
        theme: Theme::Dark,
        classic_scrollbars: Switch::On,
        background_play: Switch::On,
        default_zoom: Zoom(110),
        ..Settings::default()
    };
    settings.layout.top_bar = Switch::On;
    settings.layout.sidebar = Sidebar::Hidden;
    settings.layout.sidebar_side = Side::Right;
    settings.layout.sidebar_width = SidebarWidth::new(300.0);
    settings.layout.favorites_size = FavoritesSize::Large;
    settings.layout.favorites_columns = FavoritesColumns::Custom(7);
    settings.layout.controls = Controls {
        side: Side::Left,
        style: ControlStyle::Mac,
    };
    settings.general.language = Language::ChineseSimplified;
    settings.general.search = Search {
        engine: Engine::Custom,
        custom: "https://search.example/find?lang=en&term=%s".to_owned(),
    };
    settings.general.startup = Startup::HomePage;
    settings.general.home_page = "https://home.example/start?from=magpie".to_owned();
    settings.general.archive_after = ArchiveAfter::OneWeek;
    settings.general.downloads.folder = Some(PathBuf::from(r"D:\Saved Files"));
    settings.general.downloads.ask = Switch::On;
    settings.privacy.shield = Switch::Off;
    settings.privacy.tracking = TrackingPrevention::Strict;
    settings.privacy.do_not_track = Switch::Off;
    settings.privacy.https_only = Switch::On;
    settings.privacy.strip_tracking = Switch::Off;
    settings.privacy.trim_referrer = Switch::On;
    settings.privacy.block_preloading = Switch::On;
    settings.privacy.block_pings = Switch::On;
    settings.privacy.clear_on_exit = Switch::On;
    settings.sponsorblock = sponsorblock::Config {
        enabled: false,
        categories: [Category::Intro, Category::Filler].into_iter().collect(),
        show_skip_notice: false,
        channel_allowlist: vec!["UCexampleChannel".to_owned(), "Some Creator".to_owned()],
    };
    settings.dark_mode.toggle(&url("https://forced.example/"));
    settings.dark_mode.enabled = Switch::On;
    settings.dark_mode.toggle(&url("https://excluded.example/"));
    settings
        .dark_mode
        .toggle(&url("https://www.second.example/"));
    settings.reader = reader::Config {
        font: Font::Serif,
        size: TextSize(24),
    };
    settings.updates = Updates {
        automatic: Switch::Off,
        channel: Channel::Nightly,
        repo: Repo::parse("example/magpie"),
    };
    settings
}

fn url(text: &str) -> Url {
    Url::parse(text).unwrap()
}

#[test]
fn settings_survive_a_save_and_load() {
    let folder = tempfile::tempdir().unwrap();
    let mut config = Config::load(folder.path());
    config.settings = custom_settings();
    config.save().unwrap();

    assert_eq!(Config::load(folder.path()).settings, custom_settings());
}

#[test]
fn fresh_settings_use_the_defaults() {
    let folder = tempfile::tempdir().unwrap();
    let settings = Config::load(folder.path()).settings;

    assert_eq!(settings.theme, Theme::System);
    assert_eq!(settings.classic_scrollbars, Switch::Off);
    assert_eq!(settings.general.language, Language::System);
    assert_eq!(settings.default_zoom, Zoom(80));
    assert_eq!(
        settings.layout.sidebar_side,
        if cfg!(target_os = "macos") {
            Side::Left
        } else {
            Side::Right
        }
    );
    assert_eq!(
        settings.layout.controls,
        if cfg!(target_os = "macos") {
            Controls {
                side: Side::Left,
                style: ControlStyle::Mac,
            }
        } else {
            Controls {
                side: Side::Right,
                style: ControlStyle::Windows,
            }
        }
    );
    assert_eq!(settings.general.search.engine, Engine::DuckDuckGo);
    assert_eq!(settings.general.startup, Startup::Restore);
    assert_eq!(settings.general.archive_after, ArchiveAfter::TwelveHours);
    assert_eq!(settings.general.downloads.folder, None);
    assert_eq!(settings.privacy.shield, Switch::On);
    assert_eq!(settings.privacy.tracking, TrackingPrevention::Balanced);
    assert_eq!(settings.privacy.https_only, Switch::Off);
    assert_eq!(settings.dark_mode.enabled, Switch::Off);
    assert_eq!(settings.updates.automatic, Switch::On);
    assert_eq!(settings.updates.channel, Channel::Stable);
    assert_eq!(settings.updates.source(), Repo::default());
}

#[test]
fn only_valid_release_repos_are_read() {
    for invalid in [
        "",
        "magpie",
        "example/",
        "/magpie",
        "a/b/c",
        "exa mple/magpie",
    ] {
        let text = format!(
            "update_repo = {invalid}
"
        );
        assert_eq!(Settings::parse(&text).updates.repo, None, "{invalid}");
    }
}

#[test]
fn layout_conf_carries_over_and_later_saves_win() {
    let folder = tempfile::tempdir().unwrap();
    let legacy = "top_bar = On\nsidebar = Rail\nsidebar_side = Right\nwindow_controls = Left\n";
    fs::write(folder.path().join("layout.conf"), legacy).unwrap();

    let mut config = Config::load(folder.path());
    let migrated = config.settings.clone();
    config.settings.theme = Theme::Light;
    config.save().unwrap();
    let reloaded = Config::load(folder.path()).settings;

    assert_eq!(migrated.layout.top_bar, Switch::On);
    assert_eq!(migrated.layout.sidebar, Sidebar::Shown);
    assert_eq!(migrated.layout.sidebar_side, Side::Right);
    assert_eq!(migrated.layout.controls.side, Side::Left);
    assert_eq!(migrated.layout.controls.style, ControlStyle::Windows);
    assert_eq!(migrated.layout.sidebar_width, SidebarWidth::default());
    assert_eq!(migrated.general, Settings::default().general);
    assert_eq!(migrated.privacy, Settings::default().privacy);
    assert_eq!(reloaded.theme, Theme::Light);
    assert_eq!(reloaded.layout, migrated.layout);
}

#[test]
fn hand_edited_settings_keep_valid_lines_and_default_the_rest() {
    let text = "top_bar = ON\nsidebar = sideways\nwindow_controls=Right\ntheme = dark\naccent = teal\nlanguage = japanese\nclassic_scrollbars = sometimes\nnot a setting\n";
    let settings = Settings::parse(text);

    assert_eq!(settings.theme, Theme::Dark);
    assert_eq!(Settings::parse("accent = teal\n"), Settings::default());
    assert_eq!(settings.layout.top_bar, Switch::On);
    assert_eq!(settings.layout.sidebar, Sidebar::Shown);
    assert_eq!(settings.layout.sidebar_side, Side::Right);
    assert_eq!(settings.layout.controls.side, Side::Right);
    assert_eq!(settings.general.language, Language::Japanese);
    assert_eq!(
        Settings::parse("language = Klingon\n").general.language,
        Language::System
    );
    assert_eq!(settings.classic_scrollbars, Switch::Off);
}

#[test]
fn default_zoom_accepts_only_the_offered_steps() {
    let zoom = |text: &str| Settings::parse(text).default_zoom;

    assert_eq!(zoom("default_zoom = 125%\n"), Zoom(125));
    assert_eq!(zoom("default_zoom = 67%\n"), Zoom(67));
    assert_eq!(zoom("default_zoom = 81%\n"), Zoom(80));
    assert_eq!(zoom("default_zoom = huge\n"), Zoom(80));
}

#[test]
fn sidebar_width_stays_within_bounds() {
    let width = |text: &str| Settings::parse(text).layout.sidebar_width;

    assert_eq!(width("sidebar_width = 287\n").px(), 287.0);
    assert_eq!(width("sidebar_width = 9000\n").px(), SidebarWidth::MAX_PX);
    assert_eq!(width("sidebar_width = 12\n").px(), SidebarWidth::MIN_PX);
    assert_eq!(width("sidebar_width = wide\n"), SidebarWidth::default());
    assert_eq!(width("sidebar_width = -40\n"), SidebarWidth::default());
    assert_eq!(SidebarWidth::new(f32::INFINITY).px(), SidebarWidth::MAX_PX);
}

#[test]
fn sidebar_toggles_between_shown_and_hidden() {
    let toggles: Vec<Sidebar> =
        std::iter::successors(Some(Sidebar::Shown), |state| Some(state.toggled()))
            .take(3)
            .collect();
    assert_eq!(toggles, [Sidebar::Shown, Sidebar::Hidden, Sidebar::Shown]);
}

#[test]
fn older_sidebar_states_load_as_show_or_hide() {
    let sidebar = |text: &str| Settings::parse(text).layout.sidebar;

    assert_eq!(
        sidebar(
            "sidebar = Full
"
        ),
        Sidebar::Shown
    );
    assert_eq!(
        sidebar(
            "sidebar = rail
"
        ),
        Sidebar::Shown
    );
    assert_eq!(
        sidebar(
            "sidebar = Hidden
"
        ),
        Sidebar::Hidden
    );
    assert_eq!(
        sidebar(
            "sidebar = hide
"
        ),
        Sidebar::Hidden
    );
}

#[test]
fn favorites_columns_load_presets_and_clamped_custom_counts() {
    let columns = |text: &str| Settings::parse(text).layout.favorites_columns;

    assert_eq!(
        Settings::default().layout.favorites_columns,
        FavoritesColumns::Auto
    );
    assert_eq!(
        columns("favorites_columns = Auto\n"),
        FavoritesColumns::Auto
    );
    assert_eq!(
        columns("favorites_columns = auto\n"),
        FavoritesColumns::Auto
    );
    assert_eq!(columns("favorites_columns = 4\n"), FavoritesColumns::Four);
    assert_eq!(columns("favorites_columns = 5\n"), FavoritesColumns::Five);
    assert_eq!(columns("favorites_columns = 6\n"), FavoritesColumns::Six);
    assert_eq!(
        columns("favorites_columns = 3\n"),
        FavoritesColumns::Custom(3)
    );
    assert_eq!(
        columns("favorites_columns = 7\n"),
        FavoritesColumns::Custom(7)
    );
    assert_eq!(
        columns("favorites_columns = 0\n"),
        FavoritesColumns::Custom(1)
    );
    assert_eq!(
        columns("favorites_columns = 12\n"),
        FavoritesColumns::Custom(8)
    );
    assert_eq!(
        columns("favorites_columns = wide\n"),
        FavoritesColumns::Auto
    );
    assert_eq!(columns("favorites_columns = -2\n"), FavoritesColumns::Auto);
}

#[test]
fn favorites_columns_shrink_only_when_tiles_would_get_too_narrow() {
    let fitting = |columns: FavoritesColumns| {
        [24.0, 100.0, 188.0, 228.0].map(|row_px| columns.fitting(row_px))
    };

    assert_eq!(fitting(FavoritesColumns::Four), [1, 3, 4, 4]);
    assert_eq!(fitting(FavoritesColumns::Six), [1, 3, 5, 6]);
    assert_eq!(fitting(FavoritesColumns::Custom(8)), [1, 3, 5, 6]);
}

#[test]
fn auto_favorites_columns_fit_comfortable_tiles_between_two_and_six() {
    let auto = |row_px: f32| FavoritesColumns::Auto.fitting(row_px);

    assert_eq!(
        [24.0, 80.0, 160.0, 228.0, 280.0, 400.0].map(auto),
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(FavoritesColumns::Auto.label(), "Auto");
    assert_eq!(FavoritesColumns::ALL[0], FavoritesColumns::Auto);
}

#[test]
fn system_theme_follows_windows_and_fixed_themes_ignore_it() {
    let resolved: Vec<Scheme> = [Theme::System, Theme::Light, Theme::Dark]
        .iter()
        .flat_map(|theme| [theme.resolve(Scheme::Light), theme.resolve(Scheme::Dark)])
        .collect();
    assert_eq!(
        resolved,
        [
            Scheme::Light,
            Scheme::Dark,
            Scheme::Light,
            Scheme::Light,
            Scheme::Dark,
            Scheme::Dark
        ]
    );
}

#[test]
fn search_engines_fill_in_encoded_terms() {
    let engine = |engine| Search {
        engine,
        custom: String::new(),
    };
    let custom = |template: &str| Search {
        engine: Engine::Custom,
        custom: template.to_owned(),
    };

    assert_eq!(
        engine(Engine::Google).url("rust & gpui").as_str(),
        "https://www.google.com/search?q=rust+%26+gpui"
    );
    assert_eq!(
        engine(Engine::Kagi).url("magpie").as_str(),
        "https://kagi.com/search?q=magpie"
    );
    assert_eq!(
        custom("https://search.example/find?lang=en&term=%s")
            .url("magpie birds")
            .as_str(),
        "https://search.example/find?lang=en&term=magpie+birds"
    );
    assert_eq!(
        custom("https://search.example/").url("magpie").as_str(),
        "https://duckduckgo.com/?q=magpie"
    );
}
