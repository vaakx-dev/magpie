use gpui::{App, Context, Div, Entity, Window, div, prelude::*, px};

use super::{Dismiss, browser::Browser, find::CloseFind};
use crate::webview::{Escape, WebView};

enum Dismissible {
    Popover,
    Find,
    Offer,
    Permission(Entity<WebView>),
    Toast,
    Recovery,
    Update,
}

const EDGE_PX: f32 = 8.0;
const GAP_PX: f32 = 8.0;
const SCROLLBAR_CLEARANCE_PX: f32 = 16.0;

pub(super) fn stack(element: Div) -> Div {
    element.flex().flex_col().gap(px(GAP_PX))
}

impl Browser {
    fn corners(&self, view: &Entity<WebView>, cx: &mut Context<Self>) -> Div {
        let start = stack(div()).children(self.permission_card(view, cx));
        let end = stack(div())
            .ml_auto()
            .mr(px(SCROLLBAR_CLEARANCE_PX))
            .items_end()
            .children(self.find_card(view, cx))
            .children(self.save_toast(view, cx));
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap(px(GAP_PX))
            .child(start)
            .child(end)
    }

    pub(super) fn top_overlays(
        &self,
        banners: Option<Div>,
        view: Option<&Entity<WebView>>,
        cx: &mut Context<Self>,
    ) -> Div {
        stack(div())
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .p(px(EDGE_PX))
            .children(banners)
            .children(view.map(|view| self.corners(view, cx)))
    }

    pub(super) fn bottom_overlays(&self, cx: &mut Context<Self>) -> Option<Div> {
        let toast = self.download_toast(cx)?;
        Some(
            div()
                .absolute()
                .bottom(px(EDGE_PX))
                .right(px(EDGE_PX + SCROLLBAR_CLEARANCE_PX))
                .child(toast),
        )
    }

    fn prompted_view(&self, cx: &App) -> Option<Entity<WebView>> {
        self.shown_views()
            .find(|view| view.read(cx).prompt().is_some())
            .cloned()
    }

    fn topmost(&self, cx: &App) -> Option<Dismissible> {
        let offered = self
            .offer
            .as_ref()
            .is_some_and(|offer| self.shown_views().any(|view| offer.on(view)));
        [
            self.popover.is_some().then_some(Dismissible::Popover),
            self.find.is_some().then_some(Dismissible::Find),
            offered.then_some(Dismissible::Offer),
            self.prompted_view(cx).map(Dismissible::Permission),
            self.toast.is_some().then_some(Dismissible::Toast),
            self.recovery.is_some().then_some(Dismissible::Recovery),
            self.update_announced(cx).then_some(Dismissible::Update),
        ]
        .into_iter()
        .flatten()
        .next()
    }

    pub(super) fn dismiss_topmost(
        &mut self,
        _: &Dismiss,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(topmost) = self.topmost(cx) else {
            return;
        };
        match topmost {
            Dismissible::Popover => self.step_back(window, cx),
            Dismissible::Find => self.close_find(&CloseFind, window, cx),
            Dismissible::Offer => self.dismiss_offer(cx),
            Dismissible::Permission(view) => self.dismiss_permission(&view, cx),
            Dismissible::Toast => self.dismiss_toast(cx),
            Dismissible::Recovery => self.dismiss_recovery(cx),
            Dismissible::Update => self.dismiss_update(cx),
        }
    }

    pub(super) fn route_escape(&self, cx: &App) {
        let claimed = self.topmost(cx).is_some() && !self.fullscreen.active();
        let escape = match claimed {
            true => Escape::Shell,
            false => Escape::Page,
        };
        self.runtime.route_escape(escape);
    }
}
