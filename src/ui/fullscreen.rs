use std::mem;

use gpui::{App, Context, Entity, EntityId, Window};

use super::{ToggleFullscreen, browser::Browser};
use crate::{storage::session::TabId, webview::WebView};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page {
    pub tab: TabId,
    pub view: EntityId,
}

#[derive(Debug, Default)]
pub struct Fullscreen {
    window: bool,
    page: Option<Page>,
    shell: bool,
}

impl Fullscreen {
    pub fn active(&self) -> bool {
        self.window || self.page.is_some()
    }

    pub fn toggle_window(&mut self) {
        self.window = !self.window;
    }

    pub fn enter_page(&mut self, page: Page) -> Option<Page> {
        self.page
            .replace(page)
            .filter(|displaced| *displaced != page)
    }

    pub fn exit_page(&mut self, view: EntityId) {
        if self.page.is_some_and(|page| page.view == view) {
            self.page = None;
        }
    }

    pub fn shows(&self, view: EntityId) -> bool {
        self.page.is_none_or(|page| page.view == view)
    }

    pub fn follow(&mut self, active: Option<TabId>) -> Option<Page> {
        match self.page {
            Some(page) if Some(page.tab) == active => None,
            _ => self.page.take(),
        }
    }

    fn apply(&mut self, window: &Window) {
        let wanted = self.active();
        if mem::replace(&mut self.shell, wanted) != wanted {
            window.toggle_fullscreen();
        }
    }
}

impl Browser {
    pub(super) fn toggle_fullscreen(
        &mut self,
        _: &ToggleFullscreen,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.fullscreen.toggle_window();
        cx.notify();
    }

    pub(super) fn enter_page_fullscreen(&mut self, view: &Entity<WebView>, cx: &mut Context<Self>) {
        let Some((tab, _)) = self.locate(view) else {
            return;
        };
        let page = Page {
            tab,
            view: view.entity_id(),
        };
        if let Some(displaced) = self.fullscreen.enter_page(page) {
            self.leave_page_fullscreen(displaced, cx);
        }
        cx.notify();
    }

    pub(super) fn exit_page_fullscreen(&mut self, view: &Entity<WebView>, cx: &mut Context<Self>) {
        self.fullscreen.exit_page(view.entity_id());
        cx.notify();
    }

    fn leave_page_fullscreen(&self, page: Page, cx: &App) {
        let pane = self
            .live
            .values()
            .flatten()
            .find(|pane| pane.view.entity_id() == page.view);
        if let Some(pane) = pane {
            pane.view.read(cx).exit_fullscreen();
        }
    }

    pub(super) fn sync_fullscreen(&mut self, window: &Window, cx: &mut Context<Self>) {
        if let Some(left) = self.fullscreen.follow(self.current().active) {
            self.leave_page_fullscreen(left, cx);
        }
        self.fullscreen.apply(window);
    }
}
