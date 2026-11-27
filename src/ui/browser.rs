use std::{collections::HashMap, rc::Rc, time::Duration};

use futures::StreamExt;
use gpui::{
    AnyElement, App, Context, Div, Entity, EntityId, FocusHandle, Focusable, Hsla, MouseButton,
    MouseDownEvent, NavigationDirection, Subscription, Task, Window, div, prelude::*, px, rgba,
    transparent_black,
};
use url::Url;

use super::{
    Back, CloseTab, EditAddress, Find, Forward, NewTab, OpenDevTools, OpenSettings, Print, Reload,
    SavePage, ShowDownloads, ShowHistory, SplitView, ToggleReader, ToggleSidebar,
    command::{Choice, Command, CommandBar, Lookup, Mode, Options, Outcome, Sources, Target},
    context_menu,
    downloads::{Downloads, open_file, show_in_folder},
    drag::{FavoriteSpot, Hint},
    favicon::Favicons,
    favorite::Tiles,
    find,
    frame::INSET_PX,
    fullscreen::Fullscreen,
    menu::Layer,
    mini_player::Player,
    onboarding::Tour,
    password::{AfterUnlock, Look, Offer, Options as SheetOptions, Outcome as SheetOutcome, Sheet},
    peek::Peek,
    recovery::Notice as RecoveryNotice,
    resize::Handle,
    session::Pane,
    sidebar::Density,
    site,
    slide::Slide,
    space,
    surface::{Colors, PAGE_CORNER_PX, PANEL_CORNER_PX, colors},
    thumbnail::Thumbnails,
    toast::Toast,
    toolbar,
    top_bar::TOP_BAR_HEIGHT_PX,
    window,
};
use crate::{
    address::destination,
    config::{Config, Layout, Scheme, Settings, Side, Sidebar, Startup, Switch},
    internal::{PAGES, Page, settings::Section as SettingsSection},
    locale::{Locale, Text},
    reader,
    storage::{
        Store, closed,
        recovery::Exit,
        session::{Section, Space, TabId},
    },
    system_theme,
    webview::{self, Backdrop, Cutouts, Runtime, Transfer, Visibility, WebView},
};

const PEEK_ZONE_PX: f32 = 10.0;
const COMMAND_BAR_TOP_PX: f32 = 96.0;
const POPOVER_SPACING_PX: f32 = 8.0;
const POPOVER_OFFSET_PX: f32 = TOP_BAR_HEIGHT_PX + POPOVER_SPACING_PX;
const SWEEP_INTERVAL: Duration = Duration::from_secs(10 * 60);
const REST_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Reveal {
    Sidebar,
    Pill,
}

pub struct Passwords {
    sheet: Entity<Sheet>,
    _outcome: Subscription,
}

pub enum Popover {
    Menu(Vec<Layer>),
    Context(context_menu::Menu),
    SiteInfo(site::Info),
    Passwords(Passwords),
}

impl Popover {
    fn back(&mut self) -> bool {
        match self {
            Popover::Menu(layers) => layers.pop().is_some(),
            Popover::SiteInfo(info) => info.back(),
            Popover::Context(_) | Popover::Passwords(_) => false,
        }
    }
}

struct Palette {
    bar: Entity<CommandBar>,
    _outcome: Subscription,
}

pub struct Browser {
    pub(super) runtime: Rc<Runtime>,
    pub(super) store: Store,
    pub(super) live: HashMap<TabId, Vec<Pane>>,
    config: Config,
    system_scheme: Scheme,
    pub(super) reveal: Option<Reveal>,
    pub(super) popover: Option<Popover>,
    pub(super) offer: Option<Offer>,
    pub(super) after_unlock: Option<AfterUnlock>,
    pub(super) peek: Peek,
    pub(super) thumbnails: Thumbnails,
    palette: Option<Palette>,
    pub(super) address_edit: Option<toolbar::Edit>,
    pub(super) site_anchor: site::Anchor,
    pub(super) find: Option<find::Card>,
    pub(super) toast: Option<Toast>,
    pub(super) cutouts: Cutouts,
    pub(super) downloads: Entity<Downloads>,
    pub(super) favicons: Entity<Favicons>,
    pub(super) closed: closed::Stack,
    pub(super) drop_hint: Option<Hint>,
    pub(super) favorite_hint: Option<FavoriteSpot>,
    pub(super) tiles: Tiles,
    pub(super) resize: Handle,
    slide: Slide,
    pub(super) player: Player,
    pub(super) fullscreen: Fullscreen,
    pub(super) recovery: Option<RecoveryNotice>,
    pub(super) onboarding: Option<Tour>,
    pub(super) focus: FocusHandle,
    pub(super) leaving: Option<Task<()>>,
    _sweeper: Task<()>,
    _rester: Task<()>,
    _favicon_updates: Subscription,
    _download_updates: Subscription,
    _system_theme: Task<()>,
}

fn every(
    interval: Duration,
    chore: fn(&mut Browser, &mut Context<Browser>),
    cx: &mut Context<Browser>,
) -> Task<()> {
    cx.spawn(async move |browser, cx| {
        loop {
            cx.background_executor().timer(interval).await;
            if browser.update(cx, chore).is_err() {
                break;
            }
        }
    })
}

fn system_theme_watch(cx: &mut Context<Browser>) -> Task<()> {
    let mut changes = system_theme::changes();
    cx.spawn(async move |browser, cx| {
        while let Some(scheme) = changes.next().await {
            let followed = browser.update(cx, |browser, cx| browser.follow_system(scheme, cx));
            if followed.is_err() {
                break;
            }
        }
    })
}

impl Browser {
    pub fn new(
        runtime: Rc<Runtime>,
        mut store: Store,
        config: Config,
        cx: &mut Context<Self>,
    ) -> Self {
        space::prepare(&mut store.session);
        let favicon_cache = store.favicons();
        let thumbnails = Thumbnails::new(store.thumbnails());
        let slide = Slide::new(config.settings.layout.sidebar);
        let favicons = cx.new(|_| Favicons::new(favicon_cache));
        let favicon_updates = cx.observe(&favicons, |_, _, cx| cx.notify());
        let downloads = cx.new(Downloads::new);
        let download_updates = cx.observe(&downloads, |browser, _, cx| {
            browser.refresh_pages(&[Page::Downloads], cx);
            browser.settle_toast(cx);
            cx.notify();
        });
        let mut browser = Self {
            runtime,
            store,
            live: HashMap::new(),
            config,
            system_scheme: system_theme::scheme(),
            reveal: None,
            popover: None,
            offer: None,
            after_unlock: None,
            peek: Peek::default(),
            thumbnails,
            palette: None,
            address_edit: None,
            site_anchor: site::Anchor::default(),
            find: None,
            toast: None,
            cutouts: Cutouts::default(),
            downloads,
            favicons,
            closed: closed::Stack::default(),
            drop_hint: None,
            favorite_hint: None,
            tiles: Tiles::default(),
            resize: Handle::default(),
            slide,
            player: Player::default(),
            fullscreen: Fullscreen::default(),
            recovery: None,
            onboarding: None,
            focus: cx.focus_handle(),
            leaving: None,
            _sweeper: every(SWEEP_INTERVAL, Browser::sweep, cx),
            _rester: every(REST_INTERVAL, Browser::rest, cx),
            _favicon_updates: favicon_updates,
            _download_updates: download_updates,
            _system_theme: system_theme_watch(cx),
        };
        browser.sweep(cx);
        browser
    }

    pub fn start(
        &mut self,
        addresses: &[String],
        previous_exit: Exit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let general = self.settings().general.clone();
        match (general.startup, previous_exit) {
            (Startup::Restore, Exit::Clean) => self.resume(cx),
            (Startup::Restore, Exit::Unexpected) => self.recover(cx),
            (Startup::HomePage, _) => {
                self.archive_today(cx);
                let home = destination(&general.home_page, &general.search).url();
                self.open_in_new_tab(home, cx);
            }
            (Startup::NewTab, _) if addresses.is_empty() => {
                self.archive_today(cx);
                self.show_palette(Mode::Everything(Target::NewTab), String::new(), window, cx);
            }
            (Startup::NewTab, _) => self.archive_today(cx),
        }
        self.open_addresses(addresses, cx);
        self.offer_onboarding(cx);
    }

    pub(super) fn open_addresses(&mut self, addresses: &[String], cx: &mut Context<Self>) {
        let search = self.settings().general.search.clone();
        for address in addresses.iter().rev() {
            self.open_in_new_tab(destination(address, &search).url(), cx);
        }
    }

    pub(super) fn settings(&self) -> &Settings {
        &self.config.settings
    }

    pub(super) fn layout(&self) -> Layout {
        self.config.settings.layout
    }

    pub(super) fn scheme(&self) -> Scheme {
        self.settings().theme.resolve(self.system_scheme)
    }

    pub(super) fn locale(&self) -> Locale {
        self.settings().general.language.locale()
    }

    pub(super) fn text(&self, text: Text) -> &'static str {
        self.locale().text(text)
    }

    pub(super) fn colors(&self) -> &'static Colors {
        colors(self.scheme())
    }

    pub(super) fn look(&self) -> Look {
        Look {
            colors: self.colors(),
            locale: self.locale(),
        }
    }

    pub(super) fn backdrop(&self) -> Backdrop {
        Backdrop::from_rgba(self.colors().surface)
    }

    fn paint_backdrops(&self, cx: &mut Context<Self>) {
        let backdrop = self.backdrop();
        for pane in self.live.values().flatten() {
            pane.view.update(cx, |view, _| view.set_backdrop(backdrop));
        }
    }

    fn follow_system(&mut self, scheme: Scheme, cx: &mut Context<Self>) {
        self.system_scheme = scheme;
        self.paint_backdrops(cx);
        self.refresh_pages(&PAGES, cx);
        cx.notify();
    }

    pub(super) fn current(&self) -> &Space {
        self.store
            .session
            .active_space()
            .expect("the session always has an active space")
    }

    pub(super) fn current_mut(&mut self) -> &mut Space {
        self.store
            .session
            .active_space_mut()
            .expect("the session always has an active space")
    }

    pub(super) fn shown_views(&self) -> impl Iterator<Item = &Entity<WebView>> {
        self.current()
            .active_tab()
            .and_then(|tab| self.live.get(&tab.id))
            .into_iter()
            .flatten()
            .map(|pane| &pane.view)
    }

    pub(super) fn active_view(&self) -> Option<&Entity<WebView>> {
        let tab = self.current().active_tab()?;
        let panes = self.live.get(&tab.id)?;
        panes.get(tab.focused_index()).map(|pane| &pane.view)
    }

    pub(super) fn open(&mut self, url: Url, target: Target, cx: &mut Context<Self>) {
        match (target, self.active_view().cloned()) {
            (Target::Current, Some(view)) => view.update(cx, |view, cx| view.open(url, cx)),
            _ => self.open_in_new_tab(url, cx),
        }
    }

    fn overlaid(&self) -> bool {
        self.palette.is_some() || self.reveal.is_some() || self.onboarding.is_some()
    }

    fn floating(&self) -> bool {
        self.find.is_some() || self.address_edit.is_some() || self.popover.is_some()
    }

    fn visibility(&self, tab: TabId, index: usize, view: EntityId) -> Visibility {
        let shown = !self.overlaid() && self.fullscreen.shows(view);
        let Some(active) = self.current().active_tab().filter(|_| shown) else {
            return Visibility::Hidden;
        };
        match (tab == active.id, index == active.focused_index()) {
            (false, _) => Visibility::Hidden,
            (true, true) if self.floating() => Visibility::Shown,
            (true, true) => Visibility::Focused,
            (true, false) => Visibility::Shown,
        }
    }

    fn show_views(&self, cx: &mut Context<Self>) {
        for (tab, panes) in &self.live {
            for (index, pane) in panes.iter().enumerate() {
                let visibility = self.visibility(*tab, index, pane.view.entity_id());
                pane.view.update(cx, |view, cx| view.show(visibility, cx));
            }
        }
    }

    pub(super) fn split_active(&self) -> bool {
        self.current()
            .active_tab()
            .is_some_and(|tab| tab.panes().len() > 1)
    }

    pub(super) fn toggle_split(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.split_active() {
            true => self.unsplit(cx),
            false => self.split(window, cx),
        }
    }

    pub(super) fn with_view(
        &mut self,
        cx: &mut Context<Self>,
        action: impl FnOnce(&mut WebView, &mut Context<WebView>),
    ) {
        if let Some(view) = self.active_view().cloned() {
            view.update(cx, action);
        }
    }

    fn run(&mut self, command: Command, window: &mut Window, cx: &mut Context<Self>) {
        match command {
            Command::ToggleSidebar => self.toggle_sidebar(&ToggleSidebar, window, cx),
            Command::Settings => self.show_page(Page::Settings, cx),
            Command::SplitView => self.split(window, cx),
            Command::Unsplit => self.unsplit(cx),
            Command::NewSpace => self.new_space(cx),
            Command::PinTab => self.move_active(Section::Pinned, cx),
            Command::UnpinTab => self.move_active(Section::Today, cx),
            Command::PinInNewFolder => self.pin_in_new_folder(cx),
            Command::ToggleFavorite => self.toggle_favorite(cx),
            Command::CloseTab => self.close_tab(&CloseTab, window, cx),
            Command::Reload => self.reload(&Reload, window, cx),
            Command::FindInPage => self.open_find(&Find, window, cx),
            Command::Print => self.print(&Print, window, cx),
            Command::SavePage => self.save_page(&SavePage, window, cx),
            Command::DevTools => self.open_dev_tools(&OpenDevTools, window, cx),
            Command::ReaderMode => self.toggle_reader(&ToggleReader, window, cx),
            Command::DarkMode => self.toggle_dark_mode(cx),
            Command::Downloads => self.toggle_downloads(window, cx),
        }
    }

    pub(super) fn show_palette(
        &mut self,
        mode: Mode,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let browser = cx.weak_entity();
        let lookup: Lookup = Box::new(move |query, cx| {
            browser
                .read_with(cx, |browser, _| browser.results(query))
                .unwrap_or_default()
        });
        let options = Options {
            mode,
            text,
            colors: self.colors(),
            search: self.settings().general.search.clone(),
            locale: self.locale(),
        };
        let sources = Sources {
            lookup,
            favicons: self.favicons.clone(),
            downloads: self.downloads.clone(),
        };
        let bar = cx.new(|cx| CommandBar::new(options, sources, cx));
        let outcome = cx.subscribe_in(&bar, window, |browser, _, outcome: &Outcome, window, cx| {
            browser.hide_palette(window, cx);
            if let Outcome::Chosen(choice) = outcome {
                browser.choose(choice.clone(), window, cx);
            }
        });
        bar.focus_handle(cx).focus(window);
        self.palette = Some(Palette {
            bar,
            _outcome: outcome,
        });
        cx.notify();
    }

    pub(super) fn toggle_downloads(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mode = self
            .palette
            .as_ref()
            .map(|palette| palette.bar.read(cx).mode());
        match mode {
            Some(Mode::Downloads) => self.hide_palette(window, cx),
            _ => self.show_palette(Mode::Downloads, String::new(), window, cx),
        }
    }

    pub(super) fn download_video(&mut self, video: &Url, cx: &mut Context<Self>) {
        let folder = self.settings().general.downloads.folder.clone();
        let Some(target) = folder.or_else(webview::default_downloads) else {
            return;
        };
        let id = self.downloads.update(cx, |downloads, cx| {
            downloads.stream(video.as_str(), &target, cx)
        });
        self.announce(id, cx);
    }

    pub(super) fn track_download(&mut self, transfer: &Transfer, cx: &mut Context<Self>) {
        let id = self
            .downloads
            .update(cx, |downloads, cx| downloads.track(transfer.clone(), cx));
        self.announce(id, cx);
    }

    fn hide_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.palette = None;
        self.focus.focus(window);
        cx.notify();
    }

    fn choose(&mut self, choice: Choice, window: &mut Window, cx: &mut Context<Self>) {
        match choice {
            Choice::Open(url, target) => self.open(url, target, cx),
            Choice::Activate(id) => self.activate(id, cx),
            Choice::Restore(id) => self.restore(id, cx),
            Choice::Run(command) => self.run(command, window, cx),
            Choice::OpenFile(path) => open_file(&path),
            Choice::ShowFile(path) => show_in_folder(&path),
        }
    }

    pub(super) fn change_settings(
        &mut self,
        change: impl FnOnce(&mut Settings),
        cx: &mut Context<Self>,
    ) {
        change(&mut self.config.settings);
        self.reveal = None;
        if let Err(error) = self.config.save() {
            eprintln!("magpie: could not save the settings: {error}");
        }
        self.apply_settings(cx);
        window::share_settings(self.settings(), cx);
    }

    pub(super) fn adopt_settings(&mut self, settings: &Settings, cx: &mut Context<Self>) {
        if self.config.settings == *settings {
            return;
        }
        self.config.settings = settings.clone();
        self.apply_settings(cx);
    }

    fn apply_settings(&mut self, cx: &mut Context<Self>) {
        self.runtime.configure(self.settings());
        for pane in self.live.values().flatten() {
            pane.view.update(cx, |view, _| view.apply_settings());
        }
        self.paint_backdrops(cx);
        self.refresh_pages(&PAGES, cx);
        cx.notify();
    }

    pub(super) fn toggle_sidebar(
        &mut self,
        _: &ToggleSidebar,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.change_settings(
            |settings| settings.layout.sidebar = settings.layout.sidebar.toggled(),
            cx,
        );
    }

    pub(super) fn reveal(&mut self, reveal: Reveal, showing: bool, cx: &mut Context<Self>) {
        match showing {
            true => self.reveal = Some(reveal),
            false if self.reveal == Some(reveal) => self.reveal = None,
            false => return,
        }
        cx.notify();
    }

    pub(super) fn toggle_settings(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.show_page(Page::Settings, cx);
    }

    fn open_settings(&mut self, _: &OpenSettings, _: &mut Window, cx: &mut Context<Self>) {
        self.show_page(Page::Settings, cx);
    }

    fn show_downloads(&mut self, _: &ShowDownloads, _: &mut Window, cx: &mut Context<Self>) {
        self.show_page(Page::Downloads, cx);
    }

    fn show_history(&mut self, _: &ShowHistory, _: &mut Window, cx: &mut Context<Self>) {
        self.show_page(Page::History, cx);
    }

    pub(super) fn show_passwords(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let options = SheetOptions {
            keyring: window::keyring(cx),
            page: self.active_view().cloned(),
            look: self.look(),
        };
        let sheet = cx.new(|cx| Sheet::new(options, window, cx));
        let outcome = cx.subscribe_in(
            &sheet,
            window,
            |browser, _, outcome: &SheetOutcome, window, cx| {
                browser.dismiss_popover(window, cx);
                if let SheetOutcome::Manage = outcome {
                    browser.show_settings(SettingsSection::Passwords, cx);
                }
            },
        );
        self.popover = Some(Popover::Passwords(Passwords {
            sheet,
            _outcome: outcome,
        }));
        cx.notify();
    }

    pub(super) fn toggle_passwords(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.popover {
            Some(Popover::Passwords(_)) => self.dismiss_popover(window, cx),
            _ => self.show_passwords(window, cx),
        }
    }

    pub(super) fn passwords_shown(&self) -> bool {
        matches!(self.popover, Some(Popover::Passwords(_)))
    }

    pub(super) fn dismiss_popover(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.popover = None;
        self.focus.focus(window);
        cx.notify();
    }

    pub(super) fn step_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.popover.as_mut().is_some_and(Popover::back) {
            true => cx.notify(),
            false => self.dismiss_popover(window, cx),
        }
    }

    fn release_page_focus(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        for view in self.shown_views() {
            view.read(cx).blur();
        }
    }

    fn close_login_dropdown(&mut self, _: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.after_unlock = None;
        self.with_view(cx, WebView::forget_login_field);
    }

    fn focus_unclaimed(&mut self, _: &MouseDownEvent, window: &mut Window, _: &mut Context<Self>) {
        if !window.default_prevented() {
            self.focus.focus(window);
        }
    }

    pub(super) fn new_tab(&mut self, _: &NewTab, window: &mut Window, cx: &mut Context<Self>) {
        self.show_palette(Mode::Everything(Target::NewTab), String::new(), window, cx);
    }

    fn edit_address_action(
        &mut self,
        _: &EditAddress,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.edit_address(window, cx);
    }

    pub(super) fn back(&mut self, _: &Back, _: &mut Window, cx: &mut Context<Self>) {
        self.with_view(cx, WebView::back);
    }

    pub(super) fn forward(&mut self, _: &Forward, _: &mut Window, cx: &mut Context<Self>) {
        self.with_view(cx, WebView::forward);
    }

    pub(super) fn reload(&mut self, _: &Reload, _: &mut Window, cx: &mut Context<Self>) {
        self.with_view(cx, WebView::reload);
    }

    pub(super) fn print(&mut self, _: &Print, _: &mut Window, cx: &mut Context<Self>) {
        self.with_view(cx, WebView::print);
    }

    pub(super) fn save_page(&mut self, _: &SavePage, _: &mut Window, cx: &mut Context<Self>) {
        self.with_view(cx, WebView::save_page);
    }

    pub(super) fn open_dev_tools(
        &mut self,
        _: &OpenDevTools,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.with_view(cx, WebView::open_dev_tools);
    }

    pub(super) fn toggle_reader(
        &mut self,
        _: &ToggleReader,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.with_view(cx, WebView::toggle_reader);
    }

    pub(super) fn active_page(&self, cx: &App) -> Option<Url> {
        self.active_view()
            .and_then(|view| view.read(cx).url().cloned())
    }

    pub(super) fn toggle_dark_mode(&mut self, cx: &mut Context<Self>) {
        if let Some(page) = self.active_page(cx) {
            self.change_settings(|settings| settings.dark_mode.toggle(&page), cx);
        }
    }

    pub(super) fn keep_reader_style(&mut self, style: reader::Config, cx: &mut Context<Self>) {
        if self.settings().reader != style {
            self.change_settings(|settings| settings.reader = style, cx);
        }
    }

    pub(super) fn stop(&mut self, cx: &mut Context<Self>) {
        self.with_view(cx, WebView::stop);
    }

    fn close_tab(&mut self, _: &CloseTab, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.current().active {
            self.close(id, cx);
        }
    }

    fn split_view(&mut self, _: &SplitView, window: &mut Window, cx: &mut Context<Self>) {
        self.split(window, cx);
    }

    fn panes(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let colors = self.colors();
        let mut banners = self.banners(cx);
        let Some((tab, panes)) = self
            .current()
            .active_tab()
            .and_then(|tab| Some((tab, self.live.get(&tab.id)?)))
        else {
            let welcome = welcome(colors, self.text(Text::WelcomeHint))
                .relative()
                .child(self.top_overlays(banners, None, cx));
            return vec![welcome.into_any_element()];
        };
        let split = panes.len() > 1;
        let ring_color = colors.muted;
        let fullscreen = self.fullscreen.active();
        let corner_px = match fullscreen {
            true => 0.0,
            false => PAGE_CORNER_PX,
        };
        panes
            .iter()
            .enumerate()
            .filter(|(_, pane)| self.fullscreen.shows(pane.view.entity_id()))
            .map(|(index, pane)| {
                let ring: Hsla = match index == tab.focused_index() {
                    true => rgba(ring_color).into(),
                    false => transparent_black(),
                };
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .overflow_hidden()
                    .bg(rgba(colors.surface))
                    .rounded(px(corner_px))
                    .when(!fullscreen, |pane| pane.shadow_sm())
                    .when(split && !fullscreen, |pane| {
                        pane.border_1().border_color(ring)
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |browser, _, _, cx| browser.focus_pane(index, cx)),
                    )
                    .child(webview::surface(
                        pane.view.clone(),
                        self.cutouts.clone(),
                        corner_px,
                    ))
                    .child(self.top_overlays(banners.take(), Some(&pane.view), cx))
                    .children(self.login_dropdown(&pane.view, cx))
                    .into_any_element()
            })
            .collect()
    }

    fn sliding_row_width_px(&self, window: &Window) -> Option<f32> {
        let sliding = self.slide.moving() && !self.fullscreen.active();
        let window_width_px = f32::from(window.viewport_size().width);
        sliding.then_some(window_width_px - 2.0 * INSET_PX)
    }

    fn pane_row(&self, window: &Window, cx: &mut Context<Self>) -> Div {
        let row = div()
            .h_full()
            .flex()
            .flex_row()
            .gap(px(INSET_PX))
            .children(self.panes(cx));
        let row = match self.sliding_row_width_px(window) {
            Some(width_px) => row.flex_none().w(px(width_px)),
            None => row.flex_1().min_w_0(),
        };
        div()
            .relative()
            .flex_1()
            .min_h_0()
            .flex()
            .when(self.layout().sidebar_side == Side::Left, |anchor| {
                anchor.justify_end()
            })
            .child(row)
            .children(self.bottom_overlays(cx))
    }

    fn viewport(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let layout = self.layout();
        let docked = self.docked_width_px().is_some();
        let side = layout.sidebar_side;
        div()
            .track_focus(&self.focus)
            .flex_1()
            .h_full()
            .min_w_0()
            .p(px(INSET_PX))
            .when(docked && side == Side::Left, |viewport| viewport.pl_0())
            .when(docked && side == Side::Right, |viewport| viewport.pr_0())
            .when(layout.top_bar.on(), |viewport| viewport.pt_0())
            .when(self.fullscreen.active(), |viewport| viewport.p_0())
            .when(self.slide.moving(), |viewport| viewport.overflow_hidden())
            .flex()
            .flex_col()
            .child(self.pane_row(window, cx))
    }

    fn docked_width_px(&self) -> Option<f32> {
        let openness = self.slide.openness();
        if self.fullscreen.active() || openness == 0.0 {
            return None;
        }
        Some(INSET_PX + (self.sidebar_width_px() - INSET_PX) * openness)
    }

    fn docked_sidebar(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let width_px = self.docked_width_px()?;
        let settled = !self.slide.moving();
        let dock = div()
            .relative()
            .w(px(width_px))
            .h_full()
            .flex_none()
            .flex()
            .overflow_hidden()
            .when(self.layout().sidebar_side == Side::Left, |dock| {
                dock.justify_end()
            })
            .child(self.sidebar(window, cx))
            .when(settled, |dock| dock.child(self.resize_handle(cx)));
        Some(dock.into_any_element())
    }

    fn drop_stale_hover(&mut self, window: &Window) {
        if window.is_window_hovered() {
            return;
        }
        self.peek = Peek::default();
        self.resize.leave();
        self.tiles.leave();
    }

    fn follow_sidebar(&mut self, window: &Window) {
        self.slide.follow(self.layout().sidebar);
        if self.slide.moving() {
            window.request_animation_frame();
        }
    }

    fn body(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_row()
            .when(self.layout().sidebar_side == Side::Right, |body| {
                body.flex_row_reverse()
            })
            .children(self.docked_sidebar(window, cx))
            .child(self.viewport(window, cx))
    }

    pub(super) fn on_sidebar_side<E: Styled>(&self, element: E, inset_px: f32) -> E {
        match self.layout().sidebar_side {
            Side::Left => element.left(px(inset_px)),
            Side::Right => element.right(px(inset_px)),
        }
    }

    fn below_top_bar(&self, element: Div) -> Div {
        let top = match self.layout().top_bar {
            Switch::Off => 0.0,
            Switch::On => TOP_BAR_HEIGHT_PX,
        };
        self.on_sidebar_side(element.absolute().top(px(top)).bottom_0(), 0.0)
    }

    fn edge_reveal(&self, window: &Window, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        if self.layout().sidebar != Sidebar::Hidden {
            return None;
        }
        let revealed = self.reveal == Some(Reveal::Sidebar);
        let edge = self
            .below_top_bar(div())
            .id("sidebar-edge")
            .on_hover(cx.listener(|browser, hovered: &bool, _, cx| {
                browser.reveal(Reveal::Sidebar, *hovered, cx)
            }));
        Some(match revealed {
            true => edge
                .bg(rgba(self.colors().frame))
                .shadow_2xl()
                .occlude()
                .child(self.sidebar(window, cx)),
            false => edge.w(px(PEEK_ZONE_PX)),
        })
    }

    fn pill_reveal(&self, window: &Window, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let layout = self.layout();
        let bare = layout.sidebar == Sidebar::Hidden && !layout.top_bar.on();
        bare.then(|| self.pill_corner(window, cx))
    }

    fn popover_anchor(&self) -> Div {
        let anchor = self
            .cutouts
            .see_through(PANEL_CORNER_PX)
            .absolute()
            .bottom(px(INSET_PX))
            .flex()
            .flex_col();
        let layout = self.layout();
        match (layout.top_bar, layout.sidebar, self.density()) {
            (Switch::On, _, _) => anchor.top(px(TOP_BAR_HEIGHT_PX)).right(px(INSET_PX)),
            (Switch::Off, Sidebar::Shown, Density::Compact) => {
                self.on_sidebar_side(anchor.top(px(INSET_PX)), self.sidebar_width_px())
            }
            (Switch::Off, Sidebar::Shown, Density::Roomy) => {
                let top_px = self.chrome_height_px() + POPOVER_SPACING_PX;
                self.on_sidebar_side(anchor.top(px(top_px)), INSET_PX)
            }
            (Switch::Off, _, _) => {
                self.on_sidebar_side(anchor.top(px(POPOVER_OFFSET_PX)), INSET_PX)
            }
        }
    }

    fn popover_overlay(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let overlay = div().absolute().inset_0().occlude().on_mouse_down(
            MouseButton::Left,
            cx.listener(|browser, _, window, cx| browser.dismiss_popover(window, cx)),
        );
        Some(match self.popover.as_ref()? {
            Popover::Menu(layers) => {
                overlay.child(self.popover_anchor().child(self.menu(layers, cx)))
            }
            Popover::Passwords(passwords) => {
                overlay.child(self.popover_anchor().child(passwords.sheet.clone()))
            }
            Popover::Context(menu) => overlay
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(|browser, _, window, cx| browser.dismiss_popover(window, cx)),
                )
                .children(self.context_menu(menu, cx)),
            Popover::SiteInfo(info) => overlay.children(self.site_info(info, cx)),
        })
    }

    fn palette_overlay(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let palette = self.palette.as_ref()?;
        Some(
            div()
                .absolute()
                .inset_0()
                .flex()
                .justify_center()
                .items_start()
                .pt(px(COMMAND_BAR_TOP_PX))
                .bg(rgba(self.colors().scrim))
                .occlude()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|browser, _, window, cx| browser.hide_palette(window, cx)),
                )
                .child(palette.bar.clone()),
        )
    }
}

fn welcome(colors: &Colors, hint: &'static str) -> Div {
    div()
        .flex_1()
        .h_full()
        .rounded(px(PAGE_CORNER_PX))
        .bg(rgba(colors.surface))
        .shadow_sm()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .child(
            div()
                .text_3xl()
                .font_weight(gpui::FontWeight::BOLD)
                .child("Magpie"),
        )
        .child(div().text_color(rgba(colors.muted)).child(hint))
}

impl Focusable for Browser {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Browser {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self
            .active_view()
            .map(|view| format!("{} — Magpie", view.read(cx).title()))
            .unwrap_or_else(|| "Magpie".to_owned());
        window.set_window_title(&title);
        self.drop_stale_find(window, cx);
        self.cutouts.clear();
        self.sync_fullscreen(window, cx);
        self.follow_sidebar(window);
        self.drop_stale_hover(window);
        self.show_views(cx);
        self.route_escape(cx);
        let colors = self.colors();
        let chrome = !self.fullscreen.active();
        div()
            .key_context("Browser")
            .capture_any_mouse_down(cx.listener(Self::release_page_focus))
            .on_any_mouse_down(cx.listener(Self::focus_unclaimed))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::close_login_dropdown))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::click_away_from_toast))
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Back),
                cx.listener(|browser, _, window, cx| browser.back(&Back, window, cx)),
            )
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Forward),
                cx.listener(|browser, _, window, cx| browser.forward(&Forward, window, cx)),
            )
            .on_action(cx.listener(Self::toggle_sidebar))
            .on_action(cx.listener(Self::new_tab))
            .on_action(cx.listener(Self::new_window))
            .on_action(cx.listener(Self::new_private_window))
            .on_action(cx.listener(Self::edit_address_action))
            .on_action(cx.listener(Self::back))
            .on_action(cx.listener(Self::forward))
            .on_action(cx.listener(Self::reload))
            .on_action(cx.listener(Self::print))
            .on_action(cx.listener(Self::save_page))
            .on_action(cx.listener(Self::open_dev_tools))
            .on_action(cx.listener(Self::close_tab))
            .on_action(cx.listener(Self::reopen_closed_tab))
            .on_action(cx.listener(Self::split_view))
            .on_action(cx.listener(Self::open_settings))
            .on_action(cx.listener(Self::dismiss_topmost))
            .on_action(cx.listener(Self::open_find))
            .on_action(cx.listener(Self::zoom_in))
            .on_action(cx.listener(Self::zoom_out))
            .on_action(cx.listener(Self::reset_zoom))
            .on_action(cx.listener(Self::show_downloads))
            .on_action(cx.listener(Self::show_history))
            .on_action(cx.listener(Self::toggle_fullscreen))
            .on_action(cx.listener(Self::next_tab))
            .on_action(cx.listener(Self::previous_tab))
            .on_action(cx.listener(Self::select_tab))
            .on_action(cx.listener(Self::toggle_favorite_action))
            .on_action(cx.listener(Self::toggle_reader))
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgba(colors.frame))
            .font_family("Segoe UI")
            .text_sm()
            .text_color(rgba(colors.text))
            .when(chrome && self.layout().top_bar.on(), |root| {
                root.child(self.top_bar(window, cx))
            })
            .child(self.body(window, cx))
            .when(chrome, |root| {
                root.children(self.edge_reveal(window, cx))
                    .children(self.pill_reveal(window, cx))
            })
            .children(self.popover_overlay(cx))
            .children(self.palette_overlay(cx))
            .children(self.resize_overlay(cx))
            .children(self.onboarding_overlay(cx))
    }
}
