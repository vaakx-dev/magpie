mod general;
mod language;
mod layout;
mod privacy;
mod theme;
mod update;

use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};

pub use general::{ArchiveAfter, Downloads, Engine, General, Search, Startup};
pub use language::Language;
pub use layout::{
    ControlStyle, Controls, FavoritesColumns, FavoritesSize, Layout, Side, Sidebar, SidebarWidth,
};
pub use privacy::{Privacy, TrackingPrevention};
pub use theme::{Scheme, Theme};
pub use update::Updates;

use crate::{
    darkmode::{self, Mode},
    locale::Locale,
    reader, sponsorblock,
    zoom::Zoom,
};
use magpie_engine::{
    file,
    focus::{Focus, Rule},
    update::Repo,
};

const FOLDER: &str = "Magpie";
const FILE: &str = "settings.conf";
const SPONSORBLOCK_FILE: &str = "sponsorblock.conf";
const LEGACY_FILE: &str = "layout.conf";

pub trait Choosable: Copy + PartialEq + 'static {
    const ALL: &'static [Self];

    fn label(self) -> Cow<'static, str>;

    fn name(self, _: Locale) -> Cow<'static, str> {
        self.label()
    }

    fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|choice| choice.label().eq_ignore_ascii_case(text))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Switch {
    #[default]
    Off,
    On,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    pub theme: Theme,
    pub classic_scrollbars: Switch,
    pub default_zoom: Zoom,
    pub layout: Layout,
    pub general: General,
    pub privacy: Privacy,
    pub sponsorblock: sponsorblock::Config,
    pub dark_mode: darkmode::Config,
    pub reader: reader::Config,
    pub focus: Focus,
    pub updates: Updates,
}

pub struct Config {
    folder: PathBuf,
    pub settings: Settings,
}

impl Choosable for Switch {
    const ALL: &'static [Self] = &[Self::Off, Self::On];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Off => "Off",
            Self::On => "On",
        })
    }
}

impl From<bool> for Switch {
    fn from(on: bool) -> Self {
        match on {
            true => Self::On,
            false => Self::Off,
        }
    }
}

impl Switch {
    pub fn on(self) -> bool {
        self == Self::On
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Off => Self::On,
            Self::On => Self::Off,
        }
    }
}

fn choose<T: Choosable>(choice: &mut T, text: &str) {
    if let Some(parsed) = T::parse(text) {
        *choice = parsed;
    }
}

fn choose_focus(focus: &mut Focus, key: &str, value: &str) {
    let (Some(rule), Some(switch)) = (Rule::parse(key), Switch::parse(value)) else {
        return;
    };
    match switch {
        Switch::On => focus.block(rule),
        Switch::Off => focus.allow(rule),
    }
}

fn focus_pairs(focus: &Focus) -> impl Iterator<Item = (&'static str, String)> + '_ {
    Rule::ALL.into_iter().map(|rule| {
        let switch = Switch::from(focus.blocks(rule));
        (rule.key(), switch.label().into_owned())
    })
}

fn folder_text(folder: Option<&Path>) -> String {
    folder
        .map(|path| path.display().to_string())
        .unwrap_or_default()
}

impl Settings {
    pub fn parse(text: &str) -> Self {
        text.lines().filter_map(|line| line.split_once('=')).fold(
            Self::default(),
            |mut settings, (key, value)| {
                settings.set(key.trim(), value.trim());
                settings
            },
        )
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let Self {
            theme,
            classic_scrollbars,
            default_zoom,
            layout,
            general,
            privacy,
            sponsorblock: _,
            dark_mode,
            reader,
            focus,
            updates,
        } = self;
        match key {
            "theme" => choose(theme, value),
            "classic_scrollbars" => choose(classic_scrollbars, value),
            "default_zoom" => choose(default_zoom, value),
            "top_bar" => choose(&mut layout.top_bar, value),
            "sidebar" => choose(&mut layout.sidebar, value),
            "sidebar_side" => choose(&mut layout.sidebar_side, value),
            "sidebar_width" => {
                layout.sidebar_width = SidebarWidth::parse(value).unwrap_or(layout.sidebar_width)
            }
            "favorites_size" => choose(&mut layout.favorites_size, value),
            "favorites_columns" => choose(&mut layout.favorites_columns, value),
            "window_controls" => choose(&mut layout.controls.side, value),
            "window_controls_style" => choose(&mut layout.controls.style, value),
            "language" => choose(&mut general.language, value),
            "search_engine" => choose(&mut general.search.engine, value),
            "custom_search" => general.search.custom = value.to_owned(),
            "startup" => choose(&mut general.startup, value),
            "home_page" => general.home_page = value.to_owned(),
            "auto_archive" => choose(&mut general.archive_after, value),
            "download_folder" => {
                general.downloads.folder = (!value.is_empty()).then(|| PathBuf::from(value))
            }
            "ask_where_to_save" => choose(&mut general.downloads.ask, value),
            "shield" => choose(&mut privacy.shield, value),
            "tracking_prevention" => choose(&mut privacy.tracking, value),
            "do_not_track" => choose(&mut privacy.do_not_track, value),
            "https_only" => choose(&mut privacy.https_only, value),
            "strip_tracking_params" => choose(&mut privacy.strip_tracking, value),
            "trim_referrer" => choose(&mut privacy.trim_referrer, value),
            "block_preloading" => choose(&mut privacy.block_preloading, value),
            "block_link_pings" => choose(&mut privacy.block_pings, value),
            "clear_on_exit" => choose(&mut privacy.clear_on_exit, value),
            "force_dark_mode" => choose(&mut dark_mode.enabled, value),
            "force_dark_excluded" => dark_mode.set_sites(Mode::Off, value),
            "force_dark_forced" => dark_mode.set_sites(Mode::Forced, value),
            "reader_font" => choose(&mut reader.font, value),
            "reader_text_size" => choose(&mut reader.size, value),
            "check_for_updates" => choose(&mut updates.automatic, value),
            "update_channel" => choose(&mut updates.channel, value),
            "update_repo" => updates.repo = Repo::parse(value),
            _ => choose_focus(focus, key, value),
        }
    }

    pub fn text(&self) -> String {
        self.pairs()
            .iter()
            .map(|(key, value)| format!("{key} = {value}\n"))
            .collect()
    }

    pub fn pairs(&self) -> Vec<(&'static str, String)> {
        let Self {
            theme,
            classic_scrollbars,
            default_zoom,
            layout,
            general,
            privacy,
            sponsorblock: _,
            dark_mode,
            reader,
            focus,
            updates,
        } = self;
        [
            ("theme", theme.label().into_owned()),
            (
                "classic_scrollbars",
                classic_scrollbars.label().into_owned(),
            ),
            ("default_zoom", default_zoom.label().into_owned()),
            ("top_bar", layout.top_bar.label().into_owned()),
            ("sidebar", layout.sidebar.label().into_owned()),
            ("sidebar_side", layout.sidebar_side.label().into_owned()),
            ("sidebar_width", layout.sidebar_width.text()),
            ("favorites_size", layout.favorites_size.label().into_owned()),
            (
                "favorites_columns",
                layout.favorites_columns.label().into_owned(),
            ),
            ("window_controls", layout.controls.side.label().into_owned()),
            (
                "window_controls_style",
                layout.controls.style.label().into_owned(),
            ),
            ("language", general.language.label().into_owned()),
            ("search_engine", general.search.engine.label().into_owned()),
            ("custom_search", general.search.custom.clone()),
            ("startup", general.startup.label().into_owned()),
            ("home_page", general.home_page.clone()),
            ("auto_archive", general.archive_after.label().into_owned()),
            (
                "download_folder",
                folder_text(general.downloads.folder.as_deref()),
            ),
            (
                "ask_where_to_save",
                general.downloads.ask.label().into_owned(),
            ),
            ("shield", privacy.shield.label().into_owned()),
            ("tracking_prevention", privacy.tracking.label().into_owned()),
            ("do_not_track", privacy.do_not_track.label().into_owned()),
            ("https_only", privacy.https_only.label().into_owned()),
            (
                "strip_tracking_params",
                privacy.strip_tracking.label().into_owned(),
            ),
            ("trim_referrer", privacy.trim_referrer.label().into_owned()),
            (
                "block_preloading",
                privacy.block_preloading.label().into_owned(),
            ),
            ("block_link_pings", privacy.block_pings.label().into_owned()),
            ("clear_on_exit", privacy.clear_on_exit.label().into_owned()),
            ("force_dark_mode", dark_mode.enabled.label().into_owned()),
            ("force_dark_excluded", dark_mode.sites_text(Mode::Off)),
            ("force_dark_forced", dark_mode.sites_text(Mode::Forced)),
            ("reader_font", reader.font.label().into_owned()),
            ("reader_text_size", reader.size.label().into_owned()),
            ("check_for_updates", updates.automatic.label().into_owned()),
            ("update_channel", updates.channel.label().to_owned()),
            (
                "update_repo",
                updates
                    .repo
                    .as_ref()
                    .map(Repo::to_string)
                    .unwrap_or_default(),
            ),
        ]
        .into_iter()
        .chain(focus_pairs(focus))
        .collect()
    }
}

pub fn default_folder() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")?;
    Some(PathBuf::from(base).join(FOLDER))
}

fn remove_legacy(folder: &Path) -> io::Result<()> {
    match fs::remove_file(folder.join(LEGACY_FILE)) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        removed => removed,
    }
}

impl Config {
    pub fn load(folder: &Path) -> Self {
        let read = |name: &&str| fs::read_to_string(folder.join(name)).ok();
        let settings = [FILE, LEGACY_FILE]
            .iter()
            .find_map(read)
            .map_or_else(Settings::default, |text| Settings::parse(&text));
        let sponsorblock = read(&SPONSORBLOCK_FILE)
            .map_or_else(sponsorblock::Config::default, |text| {
                sponsorblock::Config::parse(&text)
            });
        Self {
            folder: folder.to_owned(),
            settings: Settings {
                sponsorblock,
                ..settings
            },
        }
    }

    pub fn save(&self) -> io::Result<()> {
        fs::create_dir_all(&self.folder)?;
        file::replace(&self.folder.join(FILE), self.settings.text().as_bytes())?;
        file::replace(
            &self.folder.join(SPONSORBLOCK_FILE),
            self.settings.sponsorblock.text().as_bytes(),
        )?;
        remove_legacy(&self.folder)
    }
}
