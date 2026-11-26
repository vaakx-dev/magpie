use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use gpui::{
    App, AppContext, Bounds, Context, DisplayId, Entity, Focusable, Global, Pixels, Result,
    TitlebarOptions, Window, WindowBounds, WindowHandle, WindowOptions, point, px, size,
};

use super::{
    NewPrivateWindow, NewTab, NewWindow, browser::Browser, page_bindings, password::Keyring,
    screen::screens, update::Updater,
};
use crate::{
    config::{Config, Settings},
    placement::{self, Placement, Rect, State},
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

fn rect(bounds: Bounds<Pixels>) -> Rect {
    Rect {
        x: f32::from(bounds.origin.x),
        y: f32::from(bounds.origin.y),
        width: f32::from(bounds.size.width),
        height: f32::from(bounds.size.height),
    }
}

fn bounds(rect: Rect) -> Bounds<Pixels> {
    Bounds {
        origin: point(px(rect.x), px(rect.y)),
        size: size(px(rect.width), px(rect.height)),
    }
}

fn placement(window_bounds: WindowBounds) -> Placement {
    let (restored, state) = match window_bounds {
        WindowBounds::Windowed(restored) | WindowBounds::Fullscreen(restored) => {
            (restored, State::Windowed)
        }
        WindowBounds::Maximized(restored) => (restored, State::Maximized),
    };
    Placement {
        bounds: rect(restored),
        state,
    }
}

fn window_bounds(placement: Placement) -> WindowBounds {
    let restored = bounds(placement.bounds);
    match placement.state {
        State::Windowed => WindowBounds::Windowed(restored),
        State::Maximized => WindowBounds::Maximized(restored),
    }
}

fn cascaded(cx: &App) -> WindowBounds {
    let centered = Bounds::centered(None, size(px(WINDOW_WIDTH_PX), px(WINDOW_HEIGHT_PX)), cx);
    let cascade = px(CASCADE_PX) * cx.windows().len() as f32;
    WindowBounds::Windowed(Bounds {
        origin: centered.origin + point(cascade, cascade),
        ..centered
    })
}

fn remembered(folder: &Path, cx: &App) -> Option<(DisplayId, WindowBounds)> {
    let saved = match placement::load(folder) {
        Ok(saved) => saved?,
        Err(error) => {
            eprintln!("magpie: could not read the window placement: {error}");
            return None;
        }
    };
    let screens = screens(cx);
    let available: Vec<_> = screens.iter().map(|(_, screen)| *screen).collect();
    let (index, fitted) = saved.fit(&available)?;
    Some((screens[index].0, window_bounds(fitted)))
}

fn remember(window: &Window, folder: &Path) {
    if let Err(error) = placement::save(folder, &placement(window.window_bounds())) {
        eprintln!("magpie: could not save the window placement: {error}");
    }
}

fn options(role: Role, folder: &Path, cx: &App) -> WindowOptions {
    let restored = match role {
        Role::Primary => remembered(folder, cx),
        Role::Secondary | Role::Private => None,
    };
    let (display_id, window_bounds) = match restored {
        Some((display, window_bounds)) => (Some(display), window_bounds),
        None => (None, cascaded(cx)),
    };
    WindowOptions {
        window_bounds: Some(window_bounds),
        display_id,
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
    let config_folder = shared.config_folder.clone();
    let config = Config::load(&config_folder);
    let role = store.role();
    cx.open_window(options(role, &config_folder, cx), move |window, cx| {
        let runtime = Runtime::new(window, setup, &config.settings)
            .expect("a GPUI window on Windows has a Win32 handle");
        let browser = cx.new(|cx| Browser::new(runtime, store, config, cx));
        browser.focus_handle(cx).focus(window);
        let closing = browser.downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            if role == Role::Primary {
                remember(window, &config_folder);
            }
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
