use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use magpie_engine::{bitwarden::CipherId, update::Channel};
use serde::Serialize;

use crate::{
    config::{
        ArchiveAfter, Choosable, ControlStyle, Engine, FavoritesColumns, FavoritesSize, Language,
        Scheme, Settings, Side, Sidebar, Startup, Switch, Theme, TrackingPrevention,
    },
    darkmode::Mode,
    import::{Profile as ImportProfile, Report},
    locale::{Locale, Text},
    reader::{Font, TextSize},
    sponsorblock::Category,
    storage::{
        permissions::{self, Kind, Origin, Rules},
        profile::{ProfileId, Profiles},
        session::{Space, SpaceId},
    },
    update::{BUILD_COMMIT, CURRENT_VERSION, Phase},
    zoom::Zoom,
};

const PAGE_TEXT: [Text; 180] = [
    Text::Settings,
    Text::Appearance,
    Text::General,
    Text::PrivacySecurity,
    Text::AddOns,
    Text::SettingsShortcuts,
    Text::Theme,
    Text::ThemeHint,
    Text::Pages,
    Text::DefaultZoom,
    Text::DefaultZoomHint,
    Text::Layout,
    Text::TopBar,
    Text::TopBarHint,
    Text::Sidebar,
    Text::SidebarHint,
    Text::SidebarSide,
    Text::FavoritesSize,
    Text::FavoritesSizeHint,
    Text::FavoritesColumns,
    Text::WindowControls,
    Text::WindowControlsHint,
    Text::WindowControlsSide,
    Text::WindowControlsSideHint,
    Text::Accessibility,
    Text::ClassicScrollbars,
    Text::ClassicScrollbarsHint,
    Text::Language,
    Text::LanguageHint,
    Text::System,
    Text::Search,
    Text::SearchEngine,
    Text::SearchEngineHint,
    Text::SearchAddress,
    Text::SearchAddressHint,
    Text::SearchAddressInvalid,
    Text::Startup,
    Text::OnStartup,
    Text::OnStartupHint,
    Text::HomePage,
    Text::HomePageHint,
    Text::Tabs,
    Text::ArchiveIdleTabs,
    Text::ArchiveIdleTabsHint,
    Text::Downloads,
    Text::DownloadFolder,
    Text::AskWhereToSave,
    Text::UseDefault,
    Text::Change,
    Text::UseThisFolder,
    Text::AdTrackerBlocker,
    Text::AdTrackerBlockerHint,
    Text::NoPausedSites,
    Text::NoPausedSitesHint,
    Text::ResumeBlocking,
    Text::BlockingPaused,
    Text::Tracking,
    Text::TrackingPrevention,
    Text::TrackingBasic,
    Text::TrackingBalanced,
    Text::TrackingStrict,
    Text::DoNotTrack,
    Text::DoNotTrackHint,
    Text::HttpsOnly,
    Text::HttpsOnlyHint,
    Text::StripTracking,
    Text::StripTrackingHint,
    Text::TrimReferrer,
    Text::TrimReferrerHint,
    Text::BlockPreloading,
    Text::BlockPreloadingHint,
    Text::BlockPings,
    Text::BlockPingsHint,
    Text::ClearOnExit,
    Text::ClearOnExitHint,
    Text::BrowsingData,
    Text::ClearBrowsingData,
    Text::ClearBrowsingDataHint,
    Text::ClearBrowsingDataConfirm,
    Text::ClearBrowsingDataWarning,
    Text::Clearing,
    Text::Cleared,
    Text::ClearFailed,
    Text::Cancel,
    Text::ClearNow,
    Text::ClearData,
    Text::SponsorBlockHint,
    Text::ShowSkipNotice,
    Text::ShowSkipNoticeHint,
    Text::SkipTheseSegments,
    Text::PausedChannels,
    Text::Remove,
    Text::NothingSkipped,
    Text::ImportFromBrowser,
    Text::NoOtherBrowsers,
    Text::NoOtherBrowsersHint,
    Text::Bookmarks,
    Text::BookmarksHint,
    Text::History,
    Text::HistoryHint,
    Text::Import,
    Text::Importing,
    Text::ImportChoose,
    Text::ImportAdds,
    Text::AdBlocker,
    Text::Off,
    Text::SitePermissions,
    Text::NoSitePermissions,
    Text::NoSitePermissionsHint,
    Text::Passwords,
    Text::SignedOut,
    Text::SignIn,
    Text::VaultLocked,
    Text::Unlock,
    Text::Unlocked,
    Text::Lock,
    Text::Working,
    Text::Unavailable,
    Text::SignInToBitwarden,
    Text::BitwardenHint,
    Text::SelfHosted,
    Text::Email,
    Text::MasterPassword,
    Text::TwoStepLogin,
    Text::VerificationCode,
    Text::SendEmailCode,
    Text::EmailCodeSent,
    Text::Continue,
    Text::UnlockWithHello,
    Text::SignOut,
    Text::Sync,
    Text::WindowsHello,
    Text::WindowsHelloHint,
    Text::SearchVault,
    Text::NoMatchingLogins,
    Text::Username,
    Text::Password,
    Text::Code,
    Text::SavedLogins,
    Text::ReaderMode,
    Text::ReaderModeHint,
    Text::ReaderFont,
    Text::ReaderTextSize,
    Text::ForceDarkMode,
    Text::ForceDarkModeHint,
    Text::DistractionBlocker,
    Text::HideYouTubeShorts,
    Text::HideYouTubeShortsHint,
    Text::HideYouTubePlayables,
    Text::HideYouTubePlayablesHint,
    Text::HideMembersOnly,
    Text::HideMembersOnlyHint,
    Text::HideYouTubeRecommendations,
    Text::HideYouTubeRecommendationsHint,
    Text::HideYouTubeComments,
    Text::HideYouTubeCommentsHint,
    Text::HideInstagramReels,
    Text::HideInstagramReelsHint,
    Text::HideFacebookReels,
    Text::HideFacebookReelsHint,
    Text::HideXForYou,
    Text::HideXForYouHint,
    Text::BlockTikTok,
    Text::BlockTikTokHint,
    Text::HideRedditPopular,
    Text::HideRedditPopularHint,
    Text::NoDarkSites,
    Text::NoDarkSitesHint,
    Text::NeverDark,
    Text::AlwaysDark,
    Text::Profiles,
    Text::ProfilesHint,
    Text::DefaultProfileHint,
    Text::NewProfileName,
    Text::Add,
    Text::Name,
    Text::SpaceProfiles,
    Text::SpaceProfilesHint,
    Text::DeleteProfile,
    Text::DeleteProfileNow,
];

const REMEMBERED: [permissions::State; 2] = [permissions::State::Allow, permissions::State::Block];

pub struct Snapshot<'a> {
    pub settings: &'a Settings,
    pub scheme: Scheme,
    pub paused_sites: BTreeSet<String>,
    pub zooms: BTreeMap<String, Zoom>,
    pub permissions: &'a BTreeMap<Origin, Rules>,
    pub default_downloads: Option<PathBuf>,
    pub data_folder: &'a Path,
    pub passwords: Passwords,
    pub profiles: &'a Profiles,
    pub spaces: &'a [Space],
    pub update: Phase,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProfileRow {
    id: ProfileId,
    name: String,
    built_in: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SpaceRow {
    id: SpaceId,
    name: String,
    profile: ProfileId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Hello {
    pub available: bool,
    pub on: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Method {
    pub value: &'static str,
    pub name: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Challenge {
    pub methods: Vec<Method>,
    pub email_sent: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Passwords {
    Working,
    Unavailable {
        reason: String,
    },
    SignedOut {
        problem: Option<String>,
    },
    Challenged {
        challenge: Challenge,
        problem: Option<String>,
    },
    Locked {
        email: Option<String>,
        hello: Hello,
        problem: Option<String>,
    },
    Unlocked {
        email: Option<String>,
        logins: usize,
        hello: Hello,
        problem: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LoginRow {
    pub id: CipherId,
    pub name: String,
    pub username: String,
    pub site: String,
    pub password: bool,
    pub code: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Choice {
    value: String,
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SiteZoom {
    site: String,
    zoom: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SitePermission {
    origin: Origin,
    kind: Kind,
    label: &'static str,
    state: permissions::State,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Segment {
    key: &'static str,
    label: &'static str,
    description: &'static str,
    color: &'static str,
    on: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SponsorBlock {
    enabled: bool,
    show_skip_notice: bool,
    categories: Vec<Segment>,
    channels: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DarkSite {
    site: String,
    mode: Mode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    version: &'static str,
    commit: Option<&'static str>,
    update: Phase,
    scheme: Scheme,
    settings: BTreeMap<&'static str, String>,
    choices: BTreeMap<&'static str, Vec<Choice>>,
    custom_search_valid: bool,
    downloads_folder: String,
    data_folder: String,
    paused_sites: BTreeSet<String>,
    zooms: Vec<SiteZoom>,
    permissions: Vec<SitePermission>,
    sponsorblock: SponsorBlock,
    dark_sites: Vec<DarkSite>,
    passwords: Passwords,
    profiles: Vec<ProfileRow>,
    spaces: Vec<SpaceRow>,
    text: BTreeMap<String, &'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ImportChoice {
    id: PathBuf,
    label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Reply {
    State(Box<State>),
    ImportProfiles {
        profiles: Vec<ImportChoice>,
    },
    Cleared {
        error: Option<String>,
    },
    Imported {
        summary: String,
        warnings: Vec<String>,
    },
    Logins {
        logins: Vec<LoginRow>,
    },
}

fn choices<T: Choosable>(locale: Locale) -> Vec<Choice> {
    T::ALL
        .iter()
        .map(|choice| Choice {
            value: choice.label().into_owned(),
            name: choice.name(locale).into_owned(),
        })
        .collect()
}

fn all_choices(locale: Locale) -> BTreeMap<&'static str, Vec<Choice>> {
    BTreeMap::from([
        ("theme", choices::<Theme>(locale)),
        ("default_zoom", choices::<Zoom>(locale)),
        ("top_bar", choices::<Switch>(locale)),
        ("sidebar", choices::<Sidebar>(locale)),
        ("sidebar_side", choices::<Side>(locale)),
        ("favorites_size", choices::<FavoritesSize>(locale)),
        ("favorites_columns", choices::<FavoritesColumns>(locale)),
        ("window_controls", choices::<Side>(locale)),
        ("window_controls_style", choices::<ControlStyle>(locale)),
        ("language", choices::<Language>(locale)),
        ("search_engine", choices::<Engine>(locale)),
        ("startup", choices::<Startup>(locale)),
        ("auto_archive", choices::<ArchiveAfter>(locale)),
        ("tracking_prevention", choices::<TrackingPrevention>(locale)),
        ("reader_font", choices::<Font>(locale)),
        ("reader_text_size", choices::<TextSize>(locale)),
        ("update_channel", choices::<Channel>(locale)),
        ("site_permission", permission_choices(locale)),
    ])
}

fn permission_choices(locale: Locale) -> Vec<Choice> {
    REMEMBERED
        .into_iter()
        .map(|state| Choice {
            value: state.name().to_owned(),
            name: locale.text(state.label()).to_owned(),
        })
        .collect()
}

fn site_permissions(sites: &BTreeMap<Origin, Rules>, locale: Locale) -> Vec<SitePermission> {
    sites
        .iter()
        .flat_map(|(origin, rules)| {
            rules.iter().map(|(kind, state)| SitePermission {
                origin: origin.clone(),
                kind: *kind,
                label: locale.text(kind.label()),
                state: *state,
            })
        })
        .collect()
}

fn sponsorblock(settings: &Settings) -> SponsorBlock {
    let config = &settings.sponsorblock;
    let categories = Category::ALL
        .into_iter()
        .map(|category| Segment {
            key: category.name(),
            label: category.label(),
            description: category.description(),
            color: category.color(),
            on: config.categories.contains(category),
        })
        .collect();
    SponsorBlock {
        enabled: config.enabled,
        show_skip_notice: config.show_skip_notice,
        categories,
        channels: config.channel_allowlist.clone(),
    }
}

fn dark_sites(settings: &Settings) -> Vec<DarkSite> {
    settings
        .dark_mode
        .exceptions()
        .iter()
        .map(|(site, mode)| DarkSite {
            site: site.clone(),
            mode: *mode,
        })
        .collect()
}

fn page_text(locale: Locale) -> BTreeMap<String, &'static str> {
    PAGE_TEXT
        .into_iter()
        .map(|text| (format!("{text:?}"), locale.text(text)))
        .collect()
}

fn site_zooms(zooms: BTreeMap<String, Zoom>) -> Vec<SiteZoom> {
    zooms
        .into_iter()
        .map(|(site, zoom)| SiteZoom {
            site,
            zoom: zoom.to_string(),
        })
        .collect()
}

fn profile_rows(profiles: &Profiles, locale: Locale) -> Vec<ProfileRow> {
    profiles
        .named(locale.text(Text::DefaultProfile))
        .into_iter()
        .map(|profile| ProfileRow {
            name: profile.name().to_owned(),
            built_in: profile.id.built_in(),
            id: profile.id,
        })
        .collect()
}

fn space_rows(spaces: &[Space]) -> Vec<SpaceRow> {
    spaces
        .iter()
        .map(|space| SpaceRow {
            id: space.id,
            name: space.name.clone(),
            profile: space.profile.clone(),
        })
        .collect()
}

fn count(amount: usize, one: &str, many: &str) -> String {
    match amount {
        1 => format!("1 {one}"),
        _ => format!("{amount} {many}"),
    }
}

impl State {
    pub fn new(snapshot: Snapshot) -> Self {
        let Snapshot {
            settings,
            scheme,
            paused_sites,
            zooms,
            permissions,
            default_downloads,
            data_folder,
            passwords,
            profiles,
            spaces,
            update,
        } = snapshot;
        let locale = settings.general.language.locale();
        let downloads_folder = settings
            .general
            .downloads
            .folder
            .clone()
            .or(default_downloads)
            .map(|folder| folder.display().to_string())
            .unwrap_or_default();
        Self {
            version: CURRENT_VERSION,
            commit: BUILD_COMMIT,
            update,
            scheme,
            settings: settings.pairs().into_iter().collect(),
            choices: all_choices(locale),
            custom_search_valid: settings.general.search.custom_valid(),
            downloads_folder,
            data_folder: data_folder.display().to_string(),
            paused_sites,
            zooms: site_zooms(zooms),
            permissions: site_permissions(permissions, locale),
            sponsorblock: sponsorblock(settings),
            dark_sites: dark_sites(settings),
            passwords,
            profiles: profile_rows(profiles, locale),
            spaces: space_rows(spaces),
            text: page_text(locale),
        }
    }
}

impl Reply {
    pub fn state(snapshot: Snapshot) -> Self {
        Self::State(Box::new(State::new(snapshot)))
    }

    pub fn import_profiles(profiles: &[ImportProfile]) -> Self {
        let profiles = profiles
            .iter()
            .map(|profile| ImportChoice {
                id: profile.folder.clone(),
                label: profile.label(),
            })
            .collect();
        Self::ImportProfiles { profiles }
    }

    pub fn cleared(error: Option<String>) -> Self {
        Self::Cleared { error }
    }

    pub fn imported(report: Report) -> Self {
        let bookmarks = count(report.bookmarks, "bookmark", "bookmarks");
        let history = count(report.visits, "history entry", "history entries");
        Self::Imported {
            summary: format!("✓ Imported {bookmarks}, {history}"),
            warnings: report.warnings,
        }
    }

    pub fn logins(logins: Vec<LoginRow>) -> Self {
        Self::Logins { logins }
    }

    pub fn json(&self) -> String {
        serde_json::to_string(self).expect("settings replies serialize to JSON")
    }
}
