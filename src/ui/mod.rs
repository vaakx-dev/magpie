mod audio;
mod backup;
mod badge;
mod banner;
mod browser;
mod command;
mod context_menu;
mod crash;
mod downloads;
mod drag;
mod exit;
mod favicon;
mod favorite;
pub mod field;
mod find;
mod frame;
pub mod fullscreen;
mod handoff;
mod icon;
mod internal;
mod marquee;
mod menu;
pub mod mini_player;
mod onboarding;
mod password;
mod peek;
mod permission;
mod pill;
mod profile;
mod recovery;
mod resize;
mod screen;
mod session;
mod sidebar;
mod site;
mod slide;
mod space;
mod surface;
mod switch;
mod tab;
mod thumbnail;
mod toast;
mod toolbar;
mod top_bar;
mod update;
mod window;
mod zoom;

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use gpui::{Action, App, AppContext, Context, Entity, KeyBinding, Window, actions};

use crate::{
    config::{self, Config},
    instance::Instance,
    storage::{
        Store,
        permissions::Permissions,
        recovery::{Exit, Sentinel},
        session::Position,
    },
    webview::Environment,
    zoom::Zooms,
};
use browser::Browser;
pub use icon::Assets;
use magpie_engine::blocker::{Shield, subscriptions};
use password::Keyring;
use update::Updater;

actions!(
    magpie,
    [
        ToggleSidebar,
        NewTab,
        NewWindow,
        NewPrivateWindow,
        EditAddress,
        Back,
        Forward,
        Reload,
        Print,
        SavePage,
        OpenDevTools,
        CloseTab,
        ReopenClosedTab,
        SplitView,
        OpenSettings,
        ClosePopover,
        Find,
        ZoomIn,
        ZoomOut,
        ResetZoom,
        ShowDownloads,
        ShowHistory,
        ToggleFullscreen,
        NextTab,
        PreviousTab,
        ToggleFavorite,
        ToggleReader
    ]
);

#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = magpie, no_json)]
pub struct SelectTab(Position);

const WEBVIEW_FOLDER: &str = "WebView2";
const BITWARDEN_FOLDER: &str = "bitwarden";
const FILTERS_FOLDER: &str = "filters";
const NUMBERED_TABS: usize = 8;

fn tab_bindings() -> impl Iterator<Item = KeyBinding> {
    let numbered = (1..=NUMBERED_TABS).map(|number| {
        let select = SelectTab(Position::Index(number - 1));
        KeyBinding::new(&format!("ctrl-{number}"), select, None)
    });
    let last = KeyBinding::new("ctrl-9", SelectTab(Position::Last), None);
    numbered.chain([last])
}

fn page_bindings() -> Vec<KeyBinding> {
    [
        KeyBinding::new("ctrl-b", ToggleSidebar, None),
        KeyBinding::new("ctrl-shift-s", ToggleSidebar, None),
        KeyBinding::new("ctrl-,", OpenSettings, None),
        KeyBinding::new("ctrl-t", NewTab, None),
        KeyBinding::new("ctrl-n", NewWindow, None),
        KeyBinding::new("ctrl-shift-n", NewPrivateWindow, None),
        KeyBinding::new("ctrl-l", EditAddress, None),
        KeyBinding::new("alt-left", Back, None),
        KeyBinding::new("alt-right", Forward, None),
        KeyBinding::new("ctrl-r", Reload, None),
        KeyBinding::new("f5", Reload, None),
        KeyBinding::new("ctrl-p", Print, None),
        KeyBinding::new("ctrl-s", SavePage, None),
        KeyBinding::new("f12", OpenDevTools, None),
        KeyBinding::new("ctrl-shift-i", OpenDevTools, None),
        KeyBinding::new("ctrl-w", CloseTab, None),
        KeyBinding::new("ctrl-shift-t", ReopenClosedTab, None),
        KeyBinding::new("ctrl-shift-\\", SplitView, None),
        KeyBinding::new("ctrl-f", Find, None),
        KeyBinding::new("ctrl-=", ZoomIn, None),
        KeyBinding::new("ctrl-shift-=", ZoomIn, None),
        KeyBinding::new("ctrl--", ZoomOut, None),
        KeyBinding::new("ctrl-0", ResetZoom, None),
        KeyBinding::new("ctrl-j", ShowDownloads, None),
        KeyBinding::new("ctrl-h", ShowHistory, None),
        KeyBinding::new("f11", ToggleFullscreen, None),
        KeyBinding::new("ctrl-tab", NextTab, None),
        KeyBinding::new("ctrl-shift-tab", PreviousTab, None),
        KeyBinding::new("ctrl-d", ToggleFavorite, None),
        KeyBinding::new("alt-r", ToggleReader, None),
    ]
    .into_iter()
    .chain(tab_bindings())
    .collect()
}

fn bindings() -> Vec<KeyBinding> {
    page_bindings()
        .into_iter()
        .chain([KeyBinding::new("escape", ClosePopover, Some("Browser"))])
        .chain(field::bindings())
        .chain(command::bindings())
        .chain(find::bindings())
        .chain(favorite::bindings())
        .chain(toolbar::bindings())
        .chain(password::bindings())
        .collect()
}

fn load_filter_lists(shield: Arc<Shield>, folder: PathBuf, cx: &mut App) {
    cx.background_spawn(async move {
        let lists = subscriptions(&folder);
        if !lists.is_empty() {
            shield.replace(lists);
        }
    })
    .detach();
}

fn keyring(config_folder: &Path, cx: &mut App) -> Entity<Keyring> {
    let keyring = cx.new(|cx| Keyring::open(&config_folder.join(BITWARDEN_FOLDER), cx));
    cx.observe(&keyring, |_, cx| internal::passwords_changed(cx))
        .detach();
    keyring
}

fn updater(config_folder: &Path, cx: &mut App) -> Entity<Updater> {
    let updater = cx.new(|_| Updater::new(config_folder));
    cx.observe(&updater, |_, cx| internal::updates_changed(cx))
        .detach();
    updater
}

fn shield(store: &Store) -> Shield {
    let shield = Shield::default();
    match store.paused_sites() {
        Ok(sites) => sites.iter().for_each(|site| shield.pause(site)),
        Err(error) => eprintln!("magpie: could not read paused sites: {error}"),
    }
    shield
}

fn watch_for_crashes(folder: &Path, cx: &mut App) -> Exit {
    let (sentinel, exit) = match Sentinel::raise(folder) {
        Ok(raised) => raised,
        Err(error) => {
            eprintln!("magpie: could not mark Magpie as running: {error}");
            return Exit::Clean;
        }
    };
    cx.on_app_quit(move |_| {
        if let Err(error) = sentinel.lower() {
            eprintln!("magpie: could not mark a clean exit: {error}");
        }
        async {}
    })
    .detach();
    exit
}

pub fn open(addresses: &[String], instance: Option<Instance>, cx: &mut App) {
    let store = match Store::open_default() {
        Ok(store) => store,
        Err(error) => {
            eprintln!("magpie: could not open the session store: {error}");
            cx.quit();
            return;
        }
    };
    cx.bind_keys(bindings());
    cx.on_window_closed(|cx| {
        if cx.windows().is_empty() {
            cx.quit();
        }
    })
    .detach();
    let shield = Arc::new(shield(&store));
    load_filter_lists(Arc::clone(&shield), store.folder().join(FILTERS_FOLDER), cx);
    let config_folder = config::default_folder().unwrap_or_else(|| store.folder().to_owned());
    let previous_exit = watch_for_crashes(&config_folder, cx);
    let settings = Config::load(&config_folder).settings;
    let keyring = keyring(&config_folder, cx);
    let updater = updater(&config_folder, cx);
    updater.update(cx, |updater, cx| {
        updater.check_if_due(&settings.updates, cx)
    });
    cx.set_global(window::Shared {
        environment: Environment::new(
            &store.folder().join(WEBVIEW_FOLDER),
            settings.general.language,
        ),
        shield,
        zooms: Rc::new(RefCell::new(Zooms::open(store.folder()))),
        permissions: Rc::new(RefCell::new(Permissions::open(&config_folder))),
        keyring,
        updater,
        config_folder,
    });
    let addresses = addresses.to_vec();
    let start = move |browser: &mut Browser, window: &mut Window, cx: &mut Context<Browser>| {
        browser.start(&addresses, previous_exit, window, cx)
    };
    if let Err(error) = window::open(store, start, cx) {
        eprintln!("magpie: could not open a window: {error}");
        cx.quit();
        return;
    }
    if let Some(instance) = instance {
        handoff::receive(instance, cx);
    }
    cx.activate(true);
}
