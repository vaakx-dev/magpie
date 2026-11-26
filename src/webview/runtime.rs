use std::{
    cell::{Ref, RefCell},
    collections::{BTreeMap, HashMap, HashSet},
    future::Future,
    path::Path,
    rc::Rc,
    sync::{Arc, Once},
};

use futures::{
    FutureExt,
    channel::oneshot,
    future::{self, LocalBoxFuture, Shared},
};
use gpui::{AnyWindowHandle, App, KeyBinding, Keystroke, Window};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use url::Url;
use webview2_com::{
    CoreWebView2EnvironmentOptions, CreateCoreWebView2ControllerCompletedHandler,
    CreateCoreWebView2EnvironmentCompletedHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows::{
    Win32::{
        Foundation::{E_ABORT, E_POINTER, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::SetFocus,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, RegisterClassW, WINDOW_EX_STYLE, WM_ERASEBKGND,
                WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN,
            },
        },
    },
    core::{Error, HSTRING, Interface, PCWSTR, Result, w},
};

use super::{
    backdrop,
    memory::{self, FrameId, Memory},
    native::{Container, Native, report},
    profile::Profile,
    scheme,
    signal::Requested,
};
use crate::{
    config::{Language, Privacy, Settings, Switch},
    darkmode, reader,
    role::Role,
    storage::{
        permissions::{Kind, Origin, Permissions, State},
        profile::ProfileId,
    },
    zoom::{Zoom, Zooms},
};
use magpie_engine::{
    blocker::{Shield, site},
    focus::Focus,
    sponsorblock::{self, Reply, Request},
};

const CONTAINER_CLASS: PCWSTR = w!("MagpieViewport");

type SharedProfile = Shared<LocalBoxFuture<'static, Result<Rc<Profile>>>>;

#[derive(Clone, PartialEq, Eq, Hash)]
enum Partition {
    Private,
    Profile(ProfileId),
}

#[derive(Clone)]
pub struct Environment(Shared<LocalBoxFuture<'static, Result<ICoreWebView2Environment10>>>);

pub struct Setup {
    pub environment: Environment,
    pub shield: Arc<Shield>,
    pub shortcuts: Vec<KeyBinding>,
    pub zooms: Rc<RefCell<Zooms>>,
    pub permissions: Rc<RefCell<Permissions>>,
    pub role: Role,
}

pub struct Runtime {
    parent: HWND,
    window: AnyWindowHandle,
    environment: Environment,
    role: Role,
    profiles: RefCell<HashMap<Partition, SharedProfile>>,
    settings: Rc<RefCell<Settings>>,
    shield: Arc<Shield>,
    sponsorblock: Arc<sponsorblock::Client>,
    shortcuts: Vec<KeyBinding>,
    zooms: Rc<RefCell<Zooms>>,
    permissions: Rc<RefCell<Permissions>>,
}

unsafe extern "system" fn container_procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_ERASEBKGND => backdrop::erase(window, wparam),
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn instance() -> Result<HINSTANCE> {
    let module = unsafe { GetModuleHandleW(None) }?;
    Ok(module.into())
}

fn register_container_class() {
    static REGISTERED: Once = Once::new();
    REGISTERED.call_once(|| {
        let class = WNDCLASSW {
            lpfnWndProc: Some(container_procedure),
            hInstance: instance().unwrap_or_default(),
            lpszClassName: CONTAINER_CLASS,
            ..WNDCLASSW::default()
        };
        unsafe { RegisterClassW(&class) };
    });
}

pub fn window_handle(window: &Window) -> Result<HWND> {
    let handle = HasWindowHandle::window_handle(window).map_err(|_| Error::from(E_POINTER))?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => Ok(HWND(handle.hwnd.get() as _)),
        _ => Err(E_POINTER.into()),
    }
}

impl Partition {
    fn profile_name(&self) -> Option<&str> {
        match self {
            Self::Private => None,
            Self::Profile(profile) => profile.webview_name(),
        }
    }
}

impl Environment {
    pub fn new(folder: &Path, language: Language) -> Self {
        let (sender, receiver) = oneshot::channel();
        let completed = CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(
            move |result, environment| {
                let environment = result.and_then(|()| environment.ok_or_else(|| E_POINTER.into()));
                sender.send(environment).ok();
                Ok(())
            },
        ));
        let options = CoreWebView2EnvironmentOptions::default();
        if let Some(locale) = language.chosen() {
            unsafe { options.set_language(locale.tag().to_owned()) };
        }
        unsafe { options.set_scheme_registrations(vec![Some(scheme::registration())]) };
        let options: ICoreWebView2EnvironmentOptions = options.into();
        let started = unsafe {
            CreateCoreWebView2EnvironmentWithOptions(
                PCWSTR::null(),
                &HSTRING::from(folder.as_os_str()),
                &options,
                &completed,
            )
        };
        let created = async move {
            started?;
            let environment: ICoreWebView2Environment =
                receiver.await.map_err(|_| Error::from(E_ABORT))??;
            environment.cast::<ICoreWebView2Environment10>()
        };
        Self(created.boxed_local().shared())
    }
}

fn controller_options(
    environment: &ICoreWebView2Environment10,
    partition: &Partition,
) -> Result<ICoreWebView2ControllerOptions> {
    let options = unsafe { environment.CreateCoreWebView2ControllerOptions() }?;
    unsafe { options.SetIsInPrivateModeEnabled(*partition == Partition::Private) }?;
    if let Some(name) = partition.profile_name() {
        unsafe { options.SetProfileName(&HSTRING::from(name)) }?;
    }
    Ok(options)
}

fn container(parent: HWND) -> Result<Container> {
    register_container_class();
    let window = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CONTAINER_CLASS,
            PCWSTR::null(),
            WS_CHILD | WS_CLIPCHILDREN,
            0,
            0,
            0,
            0,
            Some(parent),
            None,
            Some(instance()?),
            None,
        )
    }?;
    Ok(Container(window))
}

async fn controller(
    environment: &ICoreWebView2Environment10,
    container: &Container,
    partition: &Partition,
) -> Result<ICoreWebView2Controller> {
    let options = controller_options(environment, partition)?;
    let (sender, receiver) = oneshot::channel();
    let completed = CreateCoreWebView2ControllerCompletedHandler::create(Box::new(
        move |result, controller| {
            let controller = result.and_then(|()| controller.ok_or_else(|| E_POINTER.into()));
            sender.send(controller).ok();
            Ok(())
        },
    ));
    unsafe {
        environment.CreateCoreWebView2ControllerWithOptions(container.0, &options, &completed)
    }?;
    receiver.await.map_err(|_| Error::from(E_ABORT))?
}

async fn native(
    parent: HWND,
    environment: Environment,
    partition: Partition,
) -> Result<(Native, ICoreWebView2Environment)> {
    let environment = environment.0.await?;
    let container = container(parent)?;
    let controller = controller(&environment, &container, &partition).await?;
    let webview = unsafe { controller.CoreWebView2() }?;
    let native = Native {
        controller,
        webview,
        container,
        requested: Requested::default(),
    };
    Ok((native, environment.into()))
}

impl Runtime {
    pub fn new(window: &Window, setup: Setup, settings: &Settings) -> Result<Rc<Self>> {
        let runtime = Self {
            parent: window_handle(window)?,
            window: window.window_handle(),
            profiles: RefCell::default(),
            environment: setup.environment,
            role: setup.role,
            settings: Rc::new(RefCell::new(settings.clone())),
            shield: setup.shield,
            sponsorblock: Arc::default(),
            shortcuts: setup.shortcuts,
            zooms: setup.zooms,
            permissions: setup.permissions,
        };
        runtime.configure_shield();
        Ok(Rc::new(runtime))
    }

    pub fn shield(&self) -> &Arc<Shield> {
        &self.shield
    }

    fn configure_shield(&self) {
        match self.settings.borrow().privacy.shield {
            Switch::On => self.shield.enable(),
            Switch::Off => self.shield.disable(),
        }
    }

    pub fn configure(&self, settings: &Settings) {
        *self.settings.borrow_mut() = settings.clone();
        self.configure_shield();
        let profiles = self.profiles.borrow();
        let ready = profiles.values().filter_map(|profile| profile.peek());
        for profile in ready.flatten() {
            report(profile.apply(settings));
        }
    }

    fn partition(&self, profile: &ProfileId) -> Partition {
        match self.role.private() {
            true => Partition::Private,
            false => Partition::Profile(profile.clone()),
        }
    }

    fn anchor(&self, partition: Partition) -> SharedProfile {
        let page = native(self.parent, self.environment.clone(), partition);
        let settings = Rc::clone(&self.settings);
        async move {
            let (anchor, _) = page.await?;
            let profile = Profile::new(anchor)?;
            report(profile.apply(&settings.borrow()));
            Ok(Rc::new(profile))
        }
        .boxed_local()
        .shared()
    }

    fn profile(&self, partition: &Partition) -> SharedProfile {
        self.profiles
            .borrow_mut()
            .entry(partition.clone())
            .or_insert_with(|| self.anchor(partition.clone()))
            .clone()
    }

    pub fn forget_profile(&self, profile: &ProfileId) {
        self.profiles
            .borrow_mut()
            .remove(&Partition::Profile(profile.clone()));
    }

    pub fn delete_profile(&self, profile: &ProfileId) -> impl Future<Output = Result<()>> + use<> {
        let partition = Partition::Profile(profile.clone());
        let removed = self.profiles.borrow_mut().remove(&partition);
        let anchor = removed.unwrap_or_else(|| self.anchor(partition));
        async move { anchor.await?.delete() }
    }

    pub(super) fn privacy(&self) -> Privacy {
        self.settings.borrow().privacy
    }

    pub(super) fn language(&self) -> Language {
        self.settings.borrow().general.language
    }

    pub(super) fn classic_scrollbars(&self) -> Switch {
        self.settings.borrow().classic_scrollbars
    }

    pub(super) fn dark_mode(&self) -> Ref<'_, darkmode::Config> {
        Ref::map(self.settings.borrow(), |settings| &settings.dark_mode)
    }

    pub(super) fn focus(&self) -> Ref<'_, Focus> {
        Ref::map(self.settings.borrow(), |settings| &settings.focus)
    }

    pub(super) fn reader(&self) -> reader::Config {
        self.settings.borrow().reader
    }

    pub(super) fn sponsorblock_answer(
        &self,
        request: Request,
    ) -> impl FnOnce() -> Reply + Send + 'static {
        let client = Arc::clone(&self.sponsorblock);
        let config = self.settings.borrow().sponsorblock.clone();
        move || sponsorblock::answer(&client, &config, &request)
    }

    pub(super) fn asks_where_to_save(&self) -> bool {
        self.settings.borrow().general.downloads.ask.on()
    }

    pub(super) fn default_zoom(&self) -> Zoom {
        self.settings.borrow().default_zoom
    }

    pub(super) fn zoom(&self, url: &Url) -> Zoom {
        let default = self.default_zoom();
        site(url).map_or(default, |site| self.zooms.borrow().of(&site, default))
    }

    pub(super) fn set_zoom(&self, url: &Url, zoom: Zoom) {
        if let Some(site) = site(url) {
            self.set_site_zoom(&site, zoom);
        }
    }

    fn set_site_zoom(&self, site: &str, zoom: Zoom) {
        let saved = self.zooms.borrow_mut().set(site, zoom, self.default_zoom());
        if let Err(error) = saved {
            eprintln!("magpie: could not save the zoom levels: {error}");
        }
    }

    pub fn reset_zoom(&self, site: &str) {
        self.set_site_zoom(site, self.default_zoom());
    }

    pub fn site_zooms(&self) -> BTreeMap<String, Zoom> {
        self.zooms.borrow().sites().clone()
    }

    pub fn permissions(&self) -> Ref<'_, Permissions> {
        self.permissions.borrow()
    }

    pub(super) fn permission(&self, origin: &Origin, kind: Kind) -> State {
        self.permissions.borrow().state(origin, kind)
    }

    pub fn set_permission(&self, origin: &Origin, kind: Kind, state: State) {
        let saved = self.permissions.borrow_mut().set(origin, kind, state);
        if let Err(error) = saved {
            eprintln!("magpie: could not save the site permissions: {error}");
        }
    }

    pub fn clear_browsing_data(
        &self,
        profiles: &[ProfileId],
    ) -> impl Future<Output = Result<()>> + use<> {
        let partitions: HashSet<Partition> = profiles
            .iter()
            .map(|profile| self.partition(profile))
            .collect();
        let anchors: Vec<SharedProfile> = partitions
            .iter()
            .map(|partition| self.profile(partition))
            .collect();
        async move {
            let clearing = anchors
                .into_iter()
                .map(|anchor| async move { anchor.await?.clear().await });
            future::try_join_all(clearing).await.map(drop)
        }
    }

    pub fn memory(&self, frames: Vec<FrameId>) -> impl Future<Output = Result<Memory>> + 'static {
        let environment = self.environment.clone();
        async move {
            let environment = environment.0.await?.cast::<ICoreWebView2Environment13>()?;
            memory::measure(&environment, frames).await
        }
    }

    pub(super) async fn native(
        &self,
        profile: &ProfileId,
    ) -> Result<(Native, ICoreWebView2Environment)> {
        let partition = self.partition(profile);
        let anchor = self.profile(&partition);
        let page = native(self.parent, self.environment.clone(), partition);
        let (created, profile) = futures::join!(page, anchor);
        report(profile.map(drop));
        created
    }

    pub(super) fn focus_shell(&self) {
        unsafe { SetFocus(Some(self.parent)) }.ok();
    }

    pub(super) fn handles(&self, keystroke: &Keystroke) -> bool {
        self.shortcuts
            .iter()
            .any(|binding| binding.match_keystrokes(std::slice::from_ref(keystroke)) == Some(false))
    }

    pub fn window(&self) -> AnyWindowHandle {
        self.window
    }

    pub(super) fn press(&self, keystroke: Keystroke, cx: &mut App) {
        let window = self.window;
        cx.defer(move |cx| {
            window
                .update(cx, |_, window, cx| window.dispatch_keystroke(keystroke, cx))
                .ok();
        });
    }
}
