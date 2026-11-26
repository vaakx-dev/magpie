use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

use gpui::{
    App, AppContext, Bounds, Context, Entity, Focusable, Global, Result, TitlebarOptions, Window,
    WindowBounds, WindowHandle, WindowOptions, point, px, size,
};

use super::{
    NewPrivateWindow, NewTab, NewWindow, browser::Browser, page_bindings, password::Keyring,
    update::Updater,
};
use crate::{
    config::{Config, Settings},
    role::Role,
    storage::{Store, permissions::Permissions},
    webview::{Environment, Runtime, Setup},
    zoom::Zooms,
};
use magpie_engine::blocker::Shield;

const WINDOW_WIDTH_PX: f32 = 1360.0;
const WINDOW_HEIGHT_PX: f32 = 860.0;
const CASCADE_PX: f32 = 28.0;

pub struct Shared {
    pub environment: Environment,
    pub shield: Arc<Shield>,
    pub zooms: Rc<RefCell<Zooms>>,
    pub permissions: Rc<RefCell<Permissions>>,
    pub keyring: Entity<Keyring>,
    pub updater: Entity<Updater>,
    pub config_folder: PathBuf,
}

impl Global for Shared {}

pub fn keyring(cx: &App) -> Entity<Keyring> {
    cx.global::<Shared>().keyring.clone()
}

fn options(cx: &App) -> WindowOptions {
    let centered = Bounds::centered(None, size(px(WINDOW_WIDTH_PX), px(WINDOW_HEIGHT_PX)), cx);
    let cascade = px(CASCADE_PX) * cx.windows().len() as f32;
    let bounds = Bounds {
        origin: centered.origin + point(cascade, cascade),
        ..centered
    };
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some("Magpie".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn zooms(store: &Store, shared: &Shared) -> Rc<RefCell<Zooms>> {
    match store.role() {
        Role::Private => Rc::new(RefCell::new(shared.zooms.borrow().in_memory())),
        Role::Primary | Role::Secondary => Rc::clone(&shared.zooms),
    }
}

fn permissions(store: &Store, shared: &Shared) -> Rc<RefCell<Permissions>> {
    match store.role() {
        Role::Private => Rc::new(RefCell::new(shared.permissions.borrow().in_memory())),
        Role::Primary | Role::Secondary => Rc::clone(&shared.permissions),
    }
}

fn shield(store: &Store, shared: &Shared) -> Arc<Shield> {
    match store.role() {
        Role::Private => Arc::new(shared.shield.private()),
        Role::Primary | Role::Secondary => Arc::clone(&shared.shield),
    }
}

fn setup(store: &Store, shared: &Shared) -> Setup {
    Setup {
        environment: shared.environment.clone(),
        shield: shield(store, shared),
        shortcuts: page_bindings(),
        zooms: zooms(store, shared),
        permissions: permissions(store, shared),
        role: store.role(),
    }
}

pub fn open(
    store: Store,
    start: impl FnOnce(&mut Browser, &mut Window, &mut Context<Browser>) + 'static,
    cx: &mut App,
) -> Result<WindowHandle<Browser>> {
    let shared = cx.global::<Shared>();
    let setup = setup(&store, shared);
    let config = Config::load(&shared.config_folder);
    cx.open_window(options(cx), |window, cx| {
        let runtime = Runtime::new(window, setup, &config.settings)
            .expect("a GPUI window on Windows has a Win32 handle");
        let browser = cx.new(|cx| Browser::new(runtime, store, config, cx));
        browser.focus_handle(cx).focus(window);
        let closing = browser.downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            closing
                .update(cx, |browser, cx| browser.may_close(window, cx))
                .unwrap_or(true)
        });
        browser.update(cx, |browser, cx| start(browser, window, cx));
        browser
    })
}

pub fn each_browser(cx: &mut App, update: impl Fn(&mut Browser, &mut Context<Browser>) + 'static) {
    cx.defer(move |cx| {
        let browsers = cx
            .windows()
            .into_iter()
            .filter_map(|handle| handle.downcast());
        for browser in browsers {
            browser
                .update(cx, |browser: &mut Browser, _, cx| update(browser, cx))
                .ok();
        }
    });
}

pub fn share_settings(settings: &Settings, cx: &mut App) {
    let settings = settings.clone();
    each_browser(cx, move |browser, cx| browser.adopt_settings(&settings, cx));
}

fn start_blank(browser: &mut Browser, window: &mut Window, cx: &mut Context<Browser>) {
    browser.new_tab(&NewTab, window, cx);
}

impl Browser {
    fn open_window(&self, store: Store, cx: &mut Context<Self>) {
        if let Err(error) = open(store, start_blank, cx) {
            eprintln!("magpie: could not open a window: {error}");
        }
    }

    pub(super) fn new_window(&mut self, _: &NewWindow, _: &mut Window, cx: &mut Context<Self>) {
        self.open_window(self.store.new_window(), cx);
    }

    pub(super) fn new_private_window(
        &mut self,
        _: &NewPrivateWindow,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open_window(self.store.new_private_window(), cx);
    }
}
