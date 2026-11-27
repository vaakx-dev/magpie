use std::{io, rc::Rc};

use gpui::{App, AppContext, Context, Entity, Subscription, Window};
use url::Url;

use super::{
    NextTab, PreviousTab, ReopenClosedTab, SelectTab, ToggleFavorite,
    browser::Browser,
    command::{Mode, Target},
    internal::history_changed,
    space::{self, THEMES},
    window,
};
use crate::{
    internal::Page,
    storage::{
        arrange::{Item, Spot},
        clock::Epoch,
        closed::Closed,
        profile::ProfileId,
        search::{OpenTab, SearchResult},
        session::{self, Direction, Folder, Link, Pinned, Section, SpaceId, Tab, TabId},
    },
    webview::{Disposition, Event, Opening, Visit, WebView},
};
use magpie_engine::blocker::site;

const NEW_TAB_TITLE: &str = "New Tab";
const FOLDER_NAME: &str = "Folder";

pub struct Pane {
    pub view: Entity<WebView>,
    _subscriptions: [Subscription; 2],
}

#[derive(Clone)]
pub enum Partner {
    Tab(TabId),
    Page(session::Pane),
}

fn report(result: io::Result<()>) {
    if let Err(error) = result {
        eprintln!("magpie: could not save the session: {error}");
    }
}

fn host(address: &str) -> Option<String> {
    Url::parse(address).ok().as_ref().and_then(site)
}

pub fn pane_label(pane: &session::Pane) -> String {
    match pane.title.is_empty() {
        false => pane.title.clone(),
        true => host(&pane.url).unwrap_or_else(|| NEW_TAB_TITLE.to_owned()),
    }
}

impl Browser {
    pub(super) fn save_session(&self, cx: &App) {
        report(self.store.save_session());
        self.refresh_pages(&[Page::Bookmarks, Page::NewTab], cx);
    }

    pub(super) fn edit_session(
        &mut self,
        edit: impl FnOnce(&mut session::Session) -> bool,
        cx: &mut Context<Self>,
    ) {
        if edit(&mut self.store.session) {
            self.save_session(cx);
            cx.notify();
        }
    }

    fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        let space = self.store.session.space_of(id)?;
        self.store.session.space_mut(space)?.tab_mut(id)
    }

    fn pane(&self, stored: &session::Pane, profile: &ProfileId, cx: &mut Context<Self>) -> Pane {
        let runtime = Rc::clone(&self.runtime);
        let opening = Opening {
            profile: profile.clone(),
            url: Url::parse(&stored.url).ok(),
            backdrop: self.backdrop(),
        };
        let view = cx.new(|cx| WebView::new(runtime, opening, cx));
        let redraw = cx.observe(&view, |_, _, cx| cx.notify());
        let events = cx.subscribe(&view, |browser, view, event: &Event, cx| {
            browser.react(&view, event, cx)
        });
        Pane {
            view,
            _subscriptions: [redraw, events],
        }
    }

    fn react(&mut self, view: &Entity<WebView>, event: &Event, cx: &mut Context<Self>) {
        match event {
            Event::Visited(visit) => self.visited(view, visit, cx),
            Event::Updated(visit) => self.updated(view, visit, cx),
            Event::Opened(url, Disposition::Foreground) => self.open_in_new_tab(url.clone(), cx),
            Event::Opened(url, Disposition::Background) => self.open_in_background(url.clone(), cx),
            Event::Opened(url, Disposition::Split) => self.open_in_split(url, cx),
            Event::Favicon { page, png } => self
                .favicons
                .update(cx, |favicons, cx| favicons.store(page, png.clone(), cx)),
            Event::Snapshot { page, jpeg } => self.keep_snapshot(page, jpeg.clone(), cx),
            Event::Focused => self.focused(view, cx),
            Event::Download(transfer) => self.track_download(transfer, cx),
            Event::DownloadVideo(url) => self.download_video(url, cx),
            Event::Zoomed => self.zoom_changed(cx),
            Event::Internal(message) => self.answer_page(view, message.clone(), cx),
            Event::ReaderStyle(style) => self.keep_reader_style(*style, cx),
            Event::Fullscreen(true) => self.enter_page_fullscreen(view, cx),
            Event::Fullscreen(false) => self.exit_page_fullscreen(view, cx),
            Event::Submitted(submission) => self.offer_to_save(view, submission, cx),
        }
    }

    pub(super) fn locate(&self, view: &Entity<WebView>) -> Option<(TabId, usize)> {
        self.live.iter().find_map(|(id, panes)| {
            let index = panes.iter().position(|pane| &pane.view == view)?;
            Some((*id, index))
        })
    }

    fn focused(&mut self, view: &Entity<WebView>, cx: &mut Context<Self>) {
        self.popover = None;
        self.stop_editing_address(cx);
        let Some((id, index)) = self.locate(view) else {
            return;
        };
        if self.current().active == Some(id) {
            self.focus_pane(index, cx);
        }
    }

    fn updated(&mut self, view: &Entity<WebView>, visit: &Visit, cx: &mut Context<Self>) {
        let Some((id, index)) = self.locate(view) else {
            return;
        };
        let pane = session::Pane {
            url: visit.url.to_string(),
            title: visit.title.clone(),
        };
        let Some(stored) = self.tab_mut(id).and_then(|tab| tab.pane_mut(index)) else {
            return;
        };
        if *stored == pane {
            return;
        }
        *stored = pane;
        self.save_session(cx);
        cx.notify();
    }

    fn wake(&mut self, id: TabId, cx: &mut Context<Self>) {
        if self.live.contains_key(&id) {
            return;
        }
        let Some((space, tab)) = self
            .store
            .session
            .spaces
            .iter()
            .find_map(|space| Some((space, space.tab(id)?)))
        else {
            return;
        };
        let profile = space.profile.clone();
        let stored = tab.panes().to_vec();
        let panes = stored
            .iter()
            .map(|pane| self.pane(pane, &profile, cx))
            .collect();
        self.live.insert(id, panes);
    }

    fn misplaced(&self, id: TabId, panes: &[Pane], cx: &App) -> bool {
        let wanted = self.store.session.profile_of(id);
        panes
            .iter()
            .any(|pane| Some(pane.view.read(cx).profile()) != wanted)
    }

    pub(super) fn rehome(&mut self, cx: &mut Context<Self>) {
        let misplaced: Vec<TabId> = self
            .live
            .iter()
            .filter(|(id, panes)| self.misplaced(**id, panes, cx))
            .map(|(id, _)| *id)
            .collect();
        for id in misplaced {
            self.live.remove(&id);
            self.wake(id, cx);
        }
        cx.notify();
    }

    fn visited(&mut self, view: &Entity<WebView>, visit: &Visit, cx: &mut Context<Self>) {
        self.updated(view, visit, cx);
        if Page::of(&visit.url).is_some() {
            return;
        }
        let url = visit.url.to_string();
        self.store.record_visit(&url, &visit.title, Epoch::now());
        history_changed(cx);
    }

    pub(super) fn activate(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(space) = self.store.session.space_of(id) else {
            return;
        };
        self.store.session.active = Some(space);
        self.popover = None;
        self.stop_editing_address(cx);
        self.close_current_tab_palette(cx);
        self.current_mut().activate(id, Epoch::now());
        self.player.activated(id);
        self.wake(id, cx);
        self.save_session(cx);
        cx.notify();
    }

    fn activate_some(&mut self, id: Option<TabId>, cx: &mut Context<Self>) {
        if let Some(id) = id {
            self.activate(id, cx);
        }
    }

    pub(super) fn next_tab(&mut self, _: &NextTab, _: &mut Window, cx: &mut Context<Self>) {
        self.activate_some(self.current().cycled(Direction::Forward), cx);
    }

    pub(super) fn previous_tab(&mut self, _: &PreviousTab, _: &mut Window, cx: &mut Context<Self>) {
        self.activate_some(self.current().cycled(Direction::Backward), cx);
    }

    pub(super) fn select_tab(
        &mut self,
        SelectTab(position): &SelectTab,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.activate_some(self.current().tab_at(*position), cx);
    }

    fn add_tab(&mut self, url: Url) -> TabId {
        let id = self.store.next_tab_id();
        let pane = session::Pane {
            url: url.to_string(),
            title: String::new(),
        };
        self.current_mut()
            .today
            .insert(0, Tab::new(id, pane, Epoch::now()));
        id
    }

    pub(super) fn open_in_new_tab(&mut self, url: Url, cx: &mut Context<Self>) {
        let id = self.add_tab(url);
        self.activate(id, cx);
    }

    fn open_in_background(&mut self, url: Url, cx: &mut Context<Self>) {
        let id = self.add_tab(url);
        self.wake(id, cx);
        self.save_session(cx);
        cx.notify();
    }

    pub(super) fn open_favorite(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(link) = self.current().favorites.get(index) else {
            return;
        };
        let wanted = host(&link.url);
        let url = Url::parse(&link.url).ok();
        let existing = self
            .current()
            .tabs()
            .find(|tab| host(tab.url()) == wanted)
            .map(|tab| tab.id);
        match (existing, url) {
            (Some(id), _) => self.activate(id, cx),
            (None, Some(url)) => self.open_in_new_tab(url, cx),
            (None, None) => {}
        }
    }

    pub(super) fn show_active(&mut self, cx: &mut Context<Self>) {
        match self.current().active {
            Some(tab) => self.activate(tab, cx),
            None => {
                self.save_session(cx);
                cx.notify();
            }
        }
    }

    pub(super) fn switch_space(&mut self, id: SpaceId, cx: &mut Context<Self>) {
        self.store.session.active = Some(id);
        self.show_active(cx);
    }

    pub(super) fn new_space(&mut self, cx: &mut Context<Self>) {
        let id = self.store.session.next_space_id();
        let theme = &THEMES[self.store.session.spaces.len() % THEMES.len()];
        self.store.session.spaces.push(space::space(id, theme));
        self.switch_space(id, cx);
        self.refresh_pages(&[Page::Settings], cx);
    }

    fn drop_tab(&mut self, space: SpaceId, id: TabId, cx: &mut Context<Self>) {
        let showing = self
            .store
            .session
            .space_mut(space)
            .filter(|space| space.active == Some(id));
        if let Some(space) = showing {
            space.active = space.today.first().map(|tab| tab.id);
        }
        self.live.remove(&id);
        self.show_active(cx);
    }

    pub(super) fn close(&mut self, id: TabId, cx: &mut Context<Self>) {
        let space = self.current_mut();
        let removed = match space.section(id) {
            Some(Section::Today) => space.remove(id),
            Some(Section::Pinned) | None => None,
        };
        let space = space.id;
        if let Some(tab) = removed {
            self.closed.push(Closed { tab, space });
        }
        self.drop_tab(space, id, cx);
    }

    pub(super) fn remove_pinned(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(space) = self.store.session.space_of(id) else {
            return;
        };
        let removed = self
            .store
            .session
            .space_mut(space)
            .filter(|space| space.section(id) == Some(Section::Pinned))
            .and_then(|space| space.remove(id));
        if removed.is_some() {
            self.drop_tab(space, id, cx);
        }
    }

    fn bring_back(&mut self, Closed { mut tab, space }: Closed) -> Option<TabId> {
        tab.id = self.store.next_tab_id();
        let id = tab.id;
        self.store.session.home(space)?.reopen(tab, Epoch::now());
        Some(id)
    }

    pub(super) fn reopen_closed(&mut self, newest_index: usize, cx: &mut Context<Self>) {
        let reopened = self
            .closed
            .take(newest_index)
            .and_then(|closed| self.bring_back(closed));
        if let Some(id) = reopened {
            self.activate(id, cx);
        }
    }

    pub(super) fn reopen_all_closed(&mut self, cx: &mut Context<Self>) {
        let closed: Vec<Closed> = self.closed.drain_oldest_first().collect();
        let newest = closed
            .into_iter()
            .filter_map(|closed| self.bring_back(closed))
            .last();
        if let Some(id) = newest {
            self.activate(id, cx);
        }
    }

    pub(super) fn reopen_closed_tab(
        &mut self,
        _: &ReopenClosedTab,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.reopen_closed(0, cx);
    }

    pub(super) fn move_active(&mut self, to: Section, cx: &mut Context<Self>) {
        if let Some(id) = self.current().active {
            self.move_tab(id, to, cx);
        }
    }

    pub(super) fn move_tab(&mut self, id: TabId, to: Section, cx: &mut Context<Self>) {
        let space = self.current_mut();
        if space.section(id) == Some(to) {
            return;
        }
        if space.arrange(Item::Tab(id), Spot::Top(to)) {
            self.save_session(cx);
            cx.notify();
        }
    }

    pub(super) fn pin_in_new_folder(&mut self, cx: &mut Context<Self>) {
        let folder_id = self.store.session.next_folder_id();
        let space = self.current_mut();
        let Some(tab) = space.active.and_then(|id| space.remove(id)) else {
            return;
        };
        let name = host(tab.url()).unwrap_or_else(|| FOLDER_NAME.to_owned());
        let mut folder = Folder::new(folder_id, name);
        folder.items.push(Pinned::Tab(tab));
        space.pinned.push(Pinned::Folder(folder));
        self.save_session(cx);
        cx.notify();
    }

    pub(super) fn toggle_folder(&mut self, id: session::FolderId, cx: &mut Context<Self>) {
        let Some(folder) = self.current_mut().folder_mut(id) else {
            return;
        };
        folder.collapsed = !folder.collapsed;
        self.save_session(cx);
        cx.notify();
    }

    pub(super) fn toggle_favorite(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.current().active {
            self.toggle_favorite_of(id, cx);
        }
    }

    pub(super) fn toggle_favorite_action(
        &mut self,
        _: &ToggleFavorite,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle_favorite(cx);
    }

    pub(super) fn toggle_favorite_of(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(pane) = self.current().tab(id).map(|tab| tab.focused().clone()) else {
            return;
        };
        let favorites = &mut self.current_mut().favorites;
        match favorites.iter().position(|link| link.url == pane.url) {
            Some(index) => {
                favorites.remove(index);
            }
            None => favorites.push(Link {
                title: pane_label(&pane),
                url: pane.url,
            }),
        }
        self.save_session(cx);
        cx.notify();
    }

    pub(super) fn remove_favorite(&mut self, index: usize, cx: &mut Context<Self>) {
        let favorites = &mut self.current_mut().favorites;
        if index < favorites.len() {
            favorites.remove(index);
            self.save_session(cx);
            cx.notify();
        }
    }

    pub(super) fn remove_favorite_in(&mut self, space: SpaceId, url: &str, cx: &mut Context<Self>) {
        let Some(space) = self.store.session.space_mut(space) else {
            return;
        };
        space.favorites.retain(|link| link.url != url);
        self.save_session(cx);
        cx.notify();
    }

    fn default_partner(&self) -> Option<TabId> {
        let active = self.current().active?;
        self.current()
            .today
            .iter()
            .find(|tab| tab.id != active && tab.panes().len() == 1)
            .map(|tab| tab.id)
    }

    pub(super) fn split(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.current().active.is_none() || self.split_active() {
            return;
        }
        match self.default_partner() {
            Some(id) => self.split_with(Partner::Tab(id), cx),
            None => {
                self.split_with(Partner::Page(session::Pane::default()), cx);
                self.show_palette(Mode::Everything(Target::Current), String::new(), window, cx);
            }
        }
    }

    fn take_partner(
        &mut self,
        partner: Partner,
        cx: &mut Context<Self>,
    ) -> Option<(session::Pane, Vec<Pane>)> {
        match partner {
            Partner::Tab(id) => {
                let stored = self.current_mut().remove(id)?.focused().clone();
                let live = match self.live.remove(&id) {
                    Some(panes) => panes,
                    None => vec![self.pane(&stored, &self.current().profile, cx)],
                };
                Some((stored, live))
            }
            Partner::Page(stored) => {
                let live = vec![self.pane(&stored, &self.current().profile, cx)];
                Some((stored, live))
            }
        }
    }

    pub(super) fn partner(&self, id: TabId) -> Option<Partner> {
        let space = self.current();
        let tab = space.tab(id)?;
        let movable = space.section(id) == Some(Section::Today) && tab.panes().len() == 1;
        Some(match movable {
            true => Partner::Tab(id),
            false => Partner::Page(tab.focused().clone()),
        })
    }

    pub(super) fn split_with(&mut self, partner: Partner, cx: &mut Context<Self>) {
        let Some(active) = self.current().active else {
            return;
        };
        self.unsplit(cx);
        let Some((stored, live)) = self.take_partner(partner, cx) else {
            return;
        };
        self.wake(active, cx);
        if let Some(tab) = self.tab_mut(active) {
            tab.split(stored);
        }
        if let Some(panes) = self.live.get_mut(&active) {
            panes.extend(live);
        }
        self.save_session(cx);
        cx.notify();
    }

    fn open_in_split(&mut self, url: &Url, cx: &mut Context<Self>) {
        let page = session::Pane {
            url: url.to_string(),
            title: String::new(),
        };
        self.split_with(Partner::Page(page), cx);
    }

    pub(super) fn duplicate(&mut self, id: TabId, cx: &mut Context<Self>) {
        let copy_id = self.store.next_tab_id();
        let Some(copy) = self
            .current()
            .tab(id)
            .map(|tab| tab.duplicate(copy_id, Epoch::now()))
        else {
            return;
        };
        self.current_mut().today.insert(0, copy);
        self.activate(copy_id, cx);
    }

    pub(super) fn move_to_space(&mut self, item: Item, space: SpaceId, cx: &mut Context<Self>) {
        if self.store.session.move_to_space(item, space) {
            self.rehome(cx);
            self.show_active(cx);
        }
    }

    pub(super) fn unsplit(&mut self, cx: &mut Context<Self>) {
        let Some(active) = self.current().active else {
            return;
        };
        let Some(stored) = self.tab_mut(active).and_then(|tab| tab.detach(1)) else {
            return;
        };
        let id = self.store.next_tab_id();
        let detached = self
            .live
            .get_mut(&active)
            .filter(|panes| panes.len() > 1)
            .map(|panes| panes.remove(1));
        self.current_mut()
            .today
            .insert(0, Tab::new(id, stored, Epoch::now()));
        if let Some(pane) = detached {
            self.live.insert(id, vec![pane]);
        }
        self.save_session(cx);
        cx.notify();
    }

    pub(super) fn focus_pane(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(active) = self.current().active else {
            return;
        };
        if self.tab_mut(active).is_some_and(|tab| tab.focus(index)) {
            self.save_session(cx);
        }
        cx.notify();
    }

    pub(super) fn restore(&mut self, id: TabId, cx: &mut Context<Self>) {
        let session = &mut self.store.session;
        if self
            .store
            .archive
            .restore(id, session, Epoch::now())
            .is_none()
        {
            return;
        }
        report(self.store.save_archive());
        self.activate(id, cx);
    }

    fn drop_closed_panes(&mut self, cx: &mut Context<Self>) {
        let session = &self.store.session;
        self.live.retain(|id, _| session.space_of(*id).is_some());
        self.save_session(cx);
        report(self.store.save_archive());
        cx.notify();
    }

    pub(super) fn sweep(&mut self, cx: &mut Context<Self>) {
        self.archive_idle(cx);
        self.sweep_thumbnails(cx);
    }

    fn archive_idle(&mut self, cx: &mut Context<Self>) {
        let Some(idle) = self.settings().general.archive_after.idle() else {
            return;
        };
        if self.store.sweep(Epoch::now(), idle) > 0 {
            self.drop_closed_panes(cx);
        }
    }

    pub(super) fn replace_session(
        &mut self,
        mut session: session::Session,
        cx: &mut Context<Self>,
    ) {
        space::prepare(&mut session);
        self.store.replace_session(session);
        self.drop_closed_panes(cx);
    }

    pub(super) fn archive_today(&mut self, cx: &mut Context<Self>) {
        self.store.archive_today(Epoch::now());
        self.drop_closed_panes(cx);
    }

    pub(super) fn resume(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.current().active {
            self.activate(id, cx);
        }
    }

    pub(super) fn clear_history(&mut self, cx: &mut Context<Self>) {
        self.store.clear_history();
        history_changed(cx);
    }

    pub(super) fn forget_browsing(
        &mut self,
        cx: &mut Context<Self>,
    ) -> impl Future<Output = windows::core::Result<()>> + use<> {
        report(self.store.clear_browsing_data());
        history_changed(cx);
        self.runtime
            .clear_browsing_data(&self.store.profiles().ids())
    }

    pub(super) fn forget_visit(&mut self, url: &str, cx: &mut Context<Self>) {
        self.store.forget_visit(url);
        history_changed(cx);
    }

    pub(super) fn results(&self, query: &str) -> Vec<SearchResult> {
        let open: Vec<OpenTab> = self
            .store
            .session
            .spaces
            .iter()
            .flat_map(|space| &space.today)
            .map(|tab| OpenTab {
                id: tab.id,
                title: tab.title(),
                url: tab.url(),
            })
            .collect();
        self.store.search(query, &open, Epoch::now())
    }

    pub(super) fn toggle_shield(&mut self, url: &Url, cx: &mut Context<Self>) {
        let Some(site) = site(url) else {
            return;
        };
        let shield = self.runtime.shield();
        match shield.paused_on(url) {
            true => shield.resume(&site),
            false => shield.pause(&site),
        }
        self.paused_sites_changed(cx);
        if let Some(view) = self.active_view().cloned() {
            view.update(cx, WebView::reload);
        }
    }

    pub(super) fn resume_site(&mut self, site: &str, cx: &mut Context<Self>) {
        self.runtime.shield().resume(site);
        self.paused_sites_changed(cx);
    }

    fn paused_sites_changed(&mut self, cx: &mut Context<Self>) {
        report(
            self.store
                .save_paused_sites(&self.runtime.shield().paused_sites()),
        );
        window::each_browser(cx, |browser, cx| {
            browser.refresh_pages(&[Page::Settings], cx)
        });
        cx.notify();
    }
}
