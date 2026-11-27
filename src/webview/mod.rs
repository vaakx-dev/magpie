mod audio;
pub mod autofill;
mod backdrop;
mod background;
mod context_menu;
mod cosmetic;
mod crash;
mod cutout;
mod darkmode;
mod devtools;
mod download;
mod filter;
mod find;
mod focus;
mod fullscreen;
mod input;
mod key;
mod link;
pub mod media;
mod memory;
mod native;
mod navigation;
mod permission;
mod preload;
mod profile;
mod reader;
pub mod revival;
mod runtime;
mod scheme;
mod script;
mod scrollbar;
mod signal;
pub mod sleep;
mod snapshot;
mod sponsorblock;
mod youtube;
mod zoom;

use std::{
    collections::BTreeSet,
    future::Future,
    mem,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::StreamExt;
use gpui::{
    Bounds, ClipboardItem, Context, Entity, EventEmitter, Pixels, SharedString, Task, canvas,
    prelude::*, px,
};
use url::Url;
use windows::{Win32::Foundation::RECT, core::Result};

pub use audio::Audio;
pub use backdrop::Backdrop;
pub use cutout::Cutouts;
pub use download::Transfer;
pub use memory::{FrameId, Memory};
pub use permission::Access;
pub use permission::Prompt as PermissionPrompt;
pub use profile::default_downloads;
pub use runtime::{Environment, Escape, Runtime, Setup};
pub use snapshot::{HEIGHT_PX as SNAPSHOT_HEIGHT_PX, WIDTH_PX as SNAPSHOT_WIDTH_PX};

use crate::{
    find::Matches,
    internal,
    reader::Config as ReaderStyle,
    storage::{
        permissions::{Kind, Origin, State},
        profile::ProfileId,
    },
    zoom::{Step, Zoom},
};
use autofill::{Fill, Login, Submission};
use background::BackgroundPlay;
use crash::Failure;
use darkmode::DarkMode;
use download::Prompt;
use filter::Filter;
use focus::Focus;
use magpie_engine::{
    blocker::{site, web_site},
    sponsorblock::Request,
};
use media::{Control, Track};
use native::{Native, Placement, Rounded, report};
use preload::Preloading;
use revival::{Revival, Revive};
use runtime::{Created, Generation};
use scrollbar::Scrollbars;
use signal::{CurrentPage, History, Signal, Signals};
use sleep::Activity;

const PAINT_SETTLE: Duration = Duration::from_millis(1500);
const NEW_TAB_TITLE: &str = "New Tab";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Visit {
    pub url: Url,
    pub title: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    Foreground,
    Background,
    Split,
}

pub struct Opening {
    pub profile: ProfileId,
    pub url: Option<Url>,
    pub backdrop: Backdrop,
}

pub enum Event {
    Visited(Visit),
    Updated(Visit),
    Opened(Url, Disposition),
    Favicon { page: Url, png: Vec<u8> },
    Snapshot { page: Url, jpeg: Vec<u8> },
    Focused,
    Download(Transfer),
    DownloadVideo(Url),
    Zoomed,
    Fullscreen(bool),
    Internal(internal::Message),
    ReaderStyle(ReaderStyle),
    Submitted(Submission),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Hidden,
    Shown,
    Focused,
}

#[derive(Default)]
struct PageScripts {
    scrollbars: Scrollbars,
    dark_mode: DarkMode,
    focus: Focus,
    preloading: Preloading,
    background_play: BackgroundPlay,
}

pub struct WebView {
    runtime: Rc<Runtime>,
    profile: ProfileId,
    native: Option<Native>,
    url: Option<Url>,
    title: String,
    history: History,
    loading: bool,
    backdrop: Backdrop,
    matches: Option<Matches>,
    placement: Option<Placement>,
    visibility: Visibility,
    shown_or_hidden: Instant,
    audio: Audio,
    track: Option<Track>,
    asleep: bool,
    settling: Task<()>,
    creating: Task<()>,
    generation: Generation,
    scripts: PageScripts,
    crashed: bool,
    revival: Revival,
    prompts: Vec<PermissionPrompt>,
    asked: BTreeSet<Kind>,
    login: Login,
    pump: Task<()>,
}

fn frame(bounds: Bounds<Pixels>, scale: f32, corner_px: f32) -> Rounded {
    let physical = |logical: Pixels| cutout::physical(logical, scale);
    Rounded {
        x: physical(bounds.origin.x),
        y: physical(bounds.origin.y),
        width: physical(bounds.size.width),
        height: physical(bounds.size.height),
        corner: physical(px(2.0 * corner_px)),
    }
}

fn visible(bounds: Bounds<Pixels>, mask: Bounds<Pixels>, scale: f32) -> RECT {
    let physical = |logical: Pixels| cutout::physical(logical, scale);
    let shown = bounds.intersect(&mask);
    let start = shown.origin - bounds.origin;
    RECT {
        left: physical(start.x),
        top: physical(start.y),
        right: physical(start.x + shown.size.width),
        bottom: physical(start.y + shown.size.height),
    }
}

fn wire(created: &Created, runtime: &Rc<Runtime>, signals: &Signals) -> Result<()> {
    let Created {
        native,
        environment,
        ..
    } = created;
    let page = CurrentPage::default();
    navigation::wire(native, Rc::clone(runtime), Rc::clone(&page), signals)?;
    input::wire(&native.controller, Rc::clone(runtime), signals)?;
    audio::wire(&native.webview, signals)?;
    link::wire(&native.webview, signals.clone())?;
    context_menu::wire(native, environment, signals.clone())?;
    devtools::wire(&native.webview)?;
    zoom::wire(native, signals)?;
    find::wire(&native.webview, signals)?;
    media::wire(&native.webview, signals.clone())?;
    fullscreen::wire(&native.webview, signals.clone())?;
    crash::wire(&native.webview, signals.clone())?;
    permission::wire(&native.webview, Rc::clone(runtime), signals.clone())?;
    scheme::wire(&native.webview, environment.clone(), signals.clone())?;
    download::wire(&native.webview, Rc::clone(runtime), signals.clone())?;
    cosmetic::wire(&native.webview, Arc::clone(runtime.shield()))?;
    sponsorblock::wire(&native.webview, signals.clone())?;
    reader::wire(&native.webview, signals.clone())?;
    autofill::wire(&native.webview, signals.clone())?;
    youtube::wire(&native.webview, Rc::clone(runtime))?;
    filter::wire(
        &native.webview,
        Filter {
            environment: environment.clone(),
            runtime: Rc::clone(runtime),
            page,
        },
    )
}

impl PageScripts {
    fn apply(&mut self, native: &Native, runtime: &Runtime) {
        let webview = &native.webview;
        report(self.scrollbars.apply(webview, runtime.classic_scrollbars()));
        report(self.dark_mode.apply(webview, &runtime.dark_mode()));
        report(self.focus.apply(webview, &runtime.focus()));
        report(
            self.preloading
                .apply(webview, runtime.privacy().block_preloading),
        );
        report(
            self.background_play
                .apply(webview, runtime.background_play()),
        );
    }
}

fn release_focus(native: &Native, runtime: &Runtime) {
    if native.focused() {
        runtime.focus_shell();
    }
}

fn reveal(native: &Native, runtime: &Runtime, visibility: Visibility) {
    match visibility {
        Visibility::Hidden => {
            release_focus(native, runtime);
            native.hide();
        }
        Visibility::Shown => {
            release_focus(native, runtime);
            native.show();
        }
        Visibility::Focused => {
            native.show();
            native.focus();
        }
    }
}

fn access(state: State) -> Option<Access> {
    match state {
        State::Ask => None,
        State::Allow => Some(Access::Granted),
        State::Block => Some(Access::Denied),
    }
}

fn ask_where_to_save(prompt: Prompt, cx: &mut Context<WebView>) {
    let suggested = prompt.suggested();
    let folder = suggested.parent().unwrap_or(&suggested);
    let name = suggested.file_name().and_then(|name| name.to_str());
    let chosen = cx.prompt_for_new_path(folder, name);
    cx.spawn(async move |view, cx| {
        let path = chosen.await.ok().and_then(|picked| picked.ok()).flatten();
        if let Some(transfer) = prompt.finish(path) {
            view.update(cx, |_, cx| cx.emit(Event::Download(transfer)))
                .ok();
        }
    })
    .detach();
}

impl WebView {
    pub fn new(runtime: Rc<Runtime>, opening: Opening, cx: &mut Context<Self>) -> Self {
        let Opening {
            profile,
            url,
            backdrop,
        } = opening;
        let mut view = Self {
            runtime,
            profile,
            native: None,
            url,
            title: String::new(),
            history: History::default(),
            loading: false,
            backdrop,
            matches: None,
            placement: None,
            visibility: Visibility::Hidden,
            shown_or_hidden: Instant::now(),
            audio: Audio::default(),
            track: None,
            asleep: false,
            settling: Task::ready(()),
            creating: Task::ready(()),
            generation: Generation::default(),
            scripts: PageScripts::default(),
            crashed: false,
            revival: Revival::default(),
            prompts: Vec::new(),
            asked: BTreeSet::new(),
            login: Login::default(),
            pump: Task::ready(()),
        };
        view.create(cx);
        view
    }

    fn create(&mut self, cx: &mut Context<Self>) {
        let runtime = Rc::clone(&self.runtime);
        let profile = self.profile.clone();
        self.creating = cx.spawn(async move |view, cx| match runtime.native(&profile).await {
            Ok(created) => {
                view.update(cx, |view, cx| view.attach(created, cx)).ok();
            }
            Err(error) => eprintln!("magpie: could not create a WebView2: {error}"),
        });
    }

    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    pub fn profile(&self) -> &ProfileId {
        &self.profile
    }

    pub fn title(&self) -> SharedString {
        let host = || self.url.as_ref().and_then(site);
        match self.title.is_empty() {
            false => self.title.clone().into(),
            true => host().unwrap_or_else(|| NEW_TAB_TITLE.to_owned()).into(),
        }
    }

    pub fn loading(&self) -> bool {
        self.loading
    }

    pub fn can_go_back(&self) -> bool {
        self.history.back
    }

    pub fn can_go_forward(&self) -> bool {
        self.history.forward
    }

    pub fn open(&mut self, url: Url, cx: &mut Context<Self>) {
        match &self.native {
            Some(native) => native.navigate(&url),
            None => self.url = Some(url),
        }
        cx.notify();
    }

    pub fn back(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::back);
    }

    pub fn forward(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::forward);
    }

    pub fn reload(&mut self, cx: &mut Context<Self>) {
        match &self.native {
            Some(native) => native.reload(),
            None if self.crashed => self.create(cx),
            None => {}
        }
    }

    pub fn print(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::print);
    }

    pub fn save_page(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::save_page);
    }

    pub fn open_dev_tools(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::open_dev_tools);
    }

    pub fn toggle_reader(&mut self, _: &mut Context<Self>) {
        let Some(native) = &self.native else {
            return;
        };
        if self.url.as_ref().and_then(web_site).is_none() {
            return;
        }
        reader::toggle(
            &native.webview,
            &self.runtime.reader(),
            self.runtime.language().locale(),
        );
    }

    pub fn audio(&self) -> Audio {
        self.audio
    }

    pub fn track(&self) -> Option<&Track> {
        self.track.as_ref()
    }

    pub fn control(&self, control: Control) {
        if let Some(native) = &self.native {
            media::control(&native.webview, control);
        }
    }

    pub fn asleep(&self) -> bool {
        self.asleep
    }

    pub fn mute(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::mute);
    }

    pub fn unmute(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::unmute);
    }

    pub fn stop(&mut self, cx: &mut Context<Self>) {
        self.native.iter().for_each(Native::stop);
        self.loading = false;
        cx.notify();
    }

    pub fn matches(&self) -> Option<Matches> {
        self.matches
    }

    pub fn find(&mut self, query: &str, cx: &mut Context<Self>) {
        self.matches = None;
        if let Some(native) = &self.native {
            match query.is_empty() {
                true => native.stop_find(),
                false => native.find(query),
            }
        }
        cx.notify();
    }

    pub fn find_next(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::find_next);
    }

    pub fn find_previous(&mut self, _: &mut Context<Self>) {
        self.native.iter().for_each(Native::find_previous);
    }

    pub fn stop_find(&mut self, cx: &mut Context<Self>) {
        self.native.iter().for_each(Native::stop_find);
        self.matches = None;
        cx.notify();
    }

    pub fn zoom(&self) -> Zoom {
        self.url
            .as_ref()
            .map_or_else(|| self.runtime.default_zoom(), |url| self.runtime.zoom(url))
    }

    pub fn zoomed(&self) -> bool {
        self.zoom() != self.runtime.default_zoom()
    }

    pub fn step_zoom(&mut self, step: Step, cx: &mut Context<Self>) {
        let Some(url) = &self.url else {
            return;
        };
        let zoom = self.zoom().step(step, self.runtime.default_zoom());
        self.runtime.set_zoom(url, zoom);
        self.apply_zoom();
        cx.emit(Event::Zoomed);
        cx.notify();
    }

    pub fn apply_zoom(&self) {
        if let Some(native) = &self.native {
            native.zoom_to(self.zoom().factor());
        }
    }

    pub fn apply_settings(&mut self) {
        self.apply_zoom();
        if let Some(native) = &self.native {
            self.scripts.apply(native, &self.runtime);
        }
    }

    pub fn set_backdrop(&mut self, backdrop: Backdrop) {
        if self.backdrop == backdrop {
            return;
        }
        self.backdrop = backdrop;
        if let Some(native) = &self.native {
            native.paint(backdrop);
        }
    }

    pub fn login_field(&self) -> Option<autofill::Field> {
        self.login.field(self.zoom().factor() as f32)
    }

    pub fn login_form(&self) -> bool {
        self.login.form()
    }

    pub fn forget_login_field(&mut self, cx: &mut Context<Self>) {
        if self.login.focused() {
            self.login.see(autofill::Sighting::Left);
            cx.notify();
        }
    }

    pub fn autofill(&mut self, fill: &Fill, cx: &mut Context<Self>) {
        if let (Some(native), Some(page)) = (&self.native, &self.url) {
            autofill::fill(&native.webview, page, fill);
            self.login.filled();
            cx.notify();
        }
    }

    pub fn exit_fullscreen(&self) {
        if let Some(native) = &self.native {
            fullscreen::exit(&native.webview);
        }
    }

    pub fn blur(&self) {
        if let Some(native) = &self.native {
            release_focus(native, &self.runtime);
        }
    }

    pub fn frame(&self) -> Option<FrameId> {
        memory::main_frame(&self.native.as_ref()?.webview)
    }

    pub fn capture(&mut self, cx: &mut Context<Self>) {
        let Some(native) = self
            .native
            .as_ref()
            .filter(|_| self.visibility != Visibility::Hidden)
        else {
            return;
        };
        let Some(page) = self.url.clone() else {
            return;
        };
        let captured = snapshot::capture(&native.webview);
        cx.spawn(async move |view, cx| {
            let page_jpeg = match captured.await {
                Ok(jpeg) => jpeg,
                Err(error) => {
                    return eprintln!("magpie: could not capture a page preview: {error}");
                }
            };
            match cx
                .background_spawn(async move { snapshot::shrink(&page_jpeg) })
                .await
            {
                Ok(Some(jpeg)) => view
                    .update(cx, |view, cx| view.keep(page, jpeg, cx))
                    .unwrap_or_default(),
                Ok(None) => {}
                Err(error) => eprintln!("magpie: could not shrink a page preview: {error}"),
            }
        })
        .detach();
    }

    fn keep(&self, page: Url, jpeg: Vec<u8>, cx: &mut Context<Self>) {
        if self.url.as_ref() == Some(&page) {
            cx.emit(Event::Snapshot { page, jpeg });
        }
    }

    fn capture_once_painted(&mut self, cx: &mut Context<Self>) {
        self.settling = cx.spawn(async move |view, cx| {
            cx.background_executor().timer(PAINT_SETTLE).await;
            view.update(cx, Self::capture).ok();
        });
    }

    pub fn show(&mut self, visibility: Visibility, cx: &mut Context<Self>) {
        if self.visibility == visibility {
            return;
        }
        match visibility {
            Visibility::Hidden => self.capture(cx),
            Visibility::Shown | Visibility::Focused => self.capture_once_painted(cx),
        }
        self.visibility = visibility;
        self.shown_or_hidden = Instant::now();
        if visibility != Visibility::Hidden && self.crashed {
            self.reload(cx);
        }
        let Some(native) = &self.native else {
            return;
        };
        if visibility != Visibility::Hidden && mem::take(&mut self.asleep) {
            native.resume();
        }
        reveal(native, &self.runtime, self.presented());
    }

    fn presented(&self) -> Visibility {
        match self.crashed {
            true => Visibility::Hidden,
            false => self.visibility,
        }
    }

    fn crash(&mut self, failure: Failure, cx: &mut Context<Self>) {
        match failure {
            Failure::Page => {}
            Failure::Browser => self.lose_browser(),
        }
        self.crashed = true;
        self.loading = false;
        self.present();
        match self.revival.ended(Instant::now(), self.visibility) {
            Revive::Now => self.reload(cx),
            Revive::WhenShown => {}
        }
    }

    fn lose_browser(&mut self) {
        self.runtime.restart(self.generation);
        self.native = None;
    }

    fn revive(&mut self) {
        if mem::take(&mut self.crashed) {
            self.present();
        }
    }

    fn present(&self) {
        if let Some(native) = &self.native {
            reveal(native, &self.runtime, self.presented());
        }
    }

    fn activity(&self) -> Activity {
        Activity {
            visibility: self.visibility,
            audio: self.audio,
            unchanged_for: self.shown_or_hidden.elapsed(),
        }
    }

    pub fn rest(&mut self, idle: Duration, cx: &mut Context<Self>) {
        let Some(native) = self.native.as_ref().filter(|_| !self.asleep) else {
            return;
        };
        if !self.activity().drowsy(idle) {
            return;
        }
        let suspended = native.suspend();
        cx.spawn(async move |view, cx| match suspended.await {
            Ok(true) => view.update(cx, Self::fall_asleep).unwrap_or_default(),
            Ok(false) => {}
            Err(error) => eprintln!("magpie: could not suspend a WebView2: {error}"),
        })
        .detach();
    }

    fn fall_asleep(&mut self, cx: &mut Context<Self>) {
        if self.visibility == Visibility::Hidden {
            self.asleep = true;
            cx.notify();
        }
    }

    fn place(&mut self, placement: Placement) {
        if self.placement.as_ref() == Some(&placement) {
            return;
        }
        if let Some(native) = &self.native {
            native.place(&placement);
        }
        self.placement = Some(placement);
    }

    fn attach(&mut self, created: Created, cx: &mut Context<Self>) {
        let (signals, mut receiver) = signal::channel();
        if let Err(error) = wire(&created, &self.runtime, &signals) {
            eprintln!("magpie: could not wire a WebView2: {error}");
            return;
        }
        let Created {
            native, generation, ..
        } = created;
        native.paint(self.backdrop);
        self.scripts.apply(&native, &self.runtime);
        if let Some(placement) = &self.placement {
            native.place(placement);
        }
        if let Some(url) = &self.url {
            native.navigate(url);
        }
        self.native = Some(native);
        self.generation = generation;
        self.crashed = false;
        self.present();
        self.pump = cx.spawn(async move |view, cx| {
            while let Some(signal) = receiver.next().await {
                if view
                    .update(cx, |view, cx| view.receive(signal, cx))
                    .is_err()
                {
                    return;
                }
            }
        });
    }

    fn visit(&self) -> Option<Visit> {
        Some(Visit {
            url: self.url.clone()?,
            title: self.title().to_string(),
        })
    }

    fn receive(&mut self, signal: Signal, cx: &mut Context<Self>) {
        match signal {
            Signal::Rewritten(url) => self.open(url, cx),
            Signal::Prompt(prompt) => ask_where_to_save(prompt, cx),
            Signal::Download(transfer) => cx.emit(Event::Download(transfer)),
            Signal::Zoom(step) => self.step_zoom(step, cx),
            Signal::ZoomDrifted => self.apply_zoom(),
            Signal::Matches(matches) => self.matches = Some(matches),
            Signal::Started => {
                self.revive();
                self.loading = true;
                self.track = None;
                self.login = Login::default();
            }
            Signal::Source(url) => {
                self.leave_origin(&url, cx);
                self.url = Some(url);
                self.apply_zoom();
                self.emit_visit(Event::Updated, cx);
            }
            Signal::Title(title) => {
                self.title = title;
                self.emit_visit(Event::Updated, cx);
            }
            Signal::History(history) => self.history = history,
            Signal::Finished => {
                self.loading = false;
                self.capture_once_painted(cx);
                self.emit_visit(Event::Visited, cx);
            }
            Signal::Favicon(png) => {
                if let Some(page) = self.url.clone() {
                    cx.emit(Event::Favicon { page, png });
                }
            }
            Signal::Opened(url, disposition) => cx.emit(Event::Opened(url, disposition)),
            Signal::Copied(text) => cx.write_to_clipboard(ClipboardItem::new_string(text)),
            Signal::DownloadVideo(url) => cx.emit(Event::DownloadVideo(url)),
            Signal::Audio(audio) => self.audio = audio,
            Signal::Track(track) => self.track = Some(track),
            Signal::Shortcut(keystroke) => self.runtime.press(keystroke, cx),
            Signal::Focused => cx.emit(Event::Focused),
            Signal::SponsorBlock(request) => self.answer(request, cx),
            Signal::Reader(style) => cx.emit(Event::ReaderStyle(style)),
            Signal::Fullscreen(contains) => cx.emit(Event::Fullscreen(contains)),
            Signal::Internal(message) => cx.emit(Event::Internal(message)),
            Signal::Permission(prompt) => self.ask(prompt),
            Signal::Login(sighting) => self.login.see(sighting),
            Signal::Submitted(submission) => cx.emit(Event::Submitted(submission)),
            Signal::Crashed(failure) => self.crash(failure, cx),
        }
        cx.notify();
    }

    pub fn origin(&self) -> Option<Origin> {
        self.url.as_ref().and_then(Origin::of)
    }

    pub fn prompt(&self) -> Option<&PermissionPrompt> {
        self.prompts.first()
    }

    pub fn asked(&self) -> &BTreeSet<Kind> {
        &self.asked
    }

    fn ask(&mut self, prompt: PermissionPrompt) {
        if self.origin().as_ref() == Some(&prompt.origin) {
            self.asked.insert(prompt.kind);
        }
        self.prompts.push(prompt);
    }

    fn settle_where(
        &mut self,
        decide: impl Fn(&PermissionPrompt) -> Option<Access>,
        cx: &mut Context<Self>,
    ) {
        for prompt in mem::take(&mut self.prompts) {
            match decide(&prompt) {
                Some(access) => {
                    if let Some(url) = prompt.settle(access) {
                        cx.emit(Event::Opened(url, Disposition::Foreground));
                    }
                }
                None => self.prompts.push(prompt),
            }
        }
    }

    fn leave_origin(&mut self, next: &Url, cx: &mut Context<Self>) {
        if Origin::of(next) == self.origin() {
            return;
        }
        self.asked.clear();
        self.settle_where(|_| Some(Access::Denied), cx);
    }

    pub fn answer_once(&mut self, access: Access, cx: &mut Context<Self>) {
        let Some((origin, kind)) = self
            .prompt()
            .map(|first| (first.origin.clone(), first.kind))
        else {
            return;
        };
        let same = move |prompt: &PermissionPrompt| prompt.origin == origin && prompt.kind == kind;
        self.settle_where(|prompt| same(prompt).then_some(access), cx);
        cx.notify();
    }

    pub fn apply_permissions(&mut self, cx: &mut Context<Self>) {
        let runtime = Rc::clone(&self.runtime);
        self.settle_where(
            |prompt| access(runtime.permission(&prompt.origin, prompt.kind)),
            cx,
        );
        cx.notify();
    }

    pub fn clear_site_data(&self) -> Option<impl Future<Output = Result<()>> + use<>> {
        let origin = self.origin()?;
        Some(self.native.as_ref()?.clear_site_data(&origin))
    }

    fn answer(&self, request: Request, cx: &mut Context<Self>) {
        let answer = self.runtime.sponsorblock_answer(request);
        let reply = cx.background_spawn(async move { answer().json() });
        cx.spawn(async move |view, cx| {
            let json = reply.await;
            view.update(cx, |view, _| view.post(&json)).ok();
        })
        .detach();
    }

    pub fn post(&self, json: &str) {
        if let Some(native) = &self.native {
            native.post(json);
        }
    }

    fn emit_visit(&self, event: fn(Visit) -> Event, cx: &mut Context<Self>) {
        if let Some(visit) = self.visit() {
            cx.emit(event(visit));
        }
    }
}

impl EventEmitter<Event> for WebView {}

pub fn surface(view: Entity<WebView>, cutouts: Cutouts, corner_px: f32) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, (), window, cx| {
            let scale = window.scale_factor();
            let placement = Placement {
                frame: frame(bounds, scale, corner_px),
                visible: visible(bounds, window.content_mask().bounds, scale),
                cutouts: cutouts.within(bounds, scale),
            };
            view.update(cx, |view, _| view.place(placement));
        },
    )
    .size_full()
}
