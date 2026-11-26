use std::{cell::Cell, collections::BTreeSet, rc::Rc};

use gpui::{
    AnyElement, Bounds, ClickEvent, Context, Div, ElementId, Entity, FontWeight, Pixels, Point,
    SharedString, Stateful, Window, anchored, div, point, prelude::*, px, rgba,
};
use url::Url;

use super::{
    badge::{brand_badge, pill},
    browser::{Browser, Popover},
    icon::Icon,
    menu::{divider, heading, line},
    permission::icon,
    surface::{PANEL_CORNER_PX, panel},
};
use crate::{
    internal::Page,
    locale::Text,
    storage::permissions::{Kind, Origin, State},
    webview::WebView,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Security {
    Secure,
    NotSecure,
}

#[derive(Clone, Default)]
pub struct Anchor(Rc<Cell<Bounds<Pixels>>>);

struct Setting {
    origin: Origin,
    kind: Kind,
    state: State,
}

const WIDTH_PX: f32 = 320.0;
const ICON_PX: f32 = 14.0;
const LOCK_PX: f32 = 22.0;
const SPACING_PX: f32 = 6.0;
const WINDOW_MARGIN_PX: f32 = 8.0;
const CHOICE_HEIGHT_PX: f32 = 22.0;
const CHOICE_CORNER_PX: f32 = 4.0;
const CHOICES_PADDING_PX: f32 = 2.0;

fn security(url: &Url) -> Option<Security> {
    match url.scheme() {
        "https" => Some(Security::Secure),
        "http" => Some(Security::NotSecure),
        _ => None,
    }
}

fn security_icon(security: Security) -> Icon {
    match security {
        Security::Secure => Icon::Lock,
        Security::NotSecure => Icon::LockOpen,
    }
}

fn security_text(security: Security) -> (Text, Text) {
    match security {
        Security::Secure => (Text::ConnectionSecure, Text::ConnectionSecureHint),
        Security::NotSecure => (Text::NotSecure, Text::NotSecureHint),
    }
}

fn choice_id(kind: Kind, state: State) -> ElementId {
    SharedString::from(format!("site-{}-{}", kind.name(), state.name())).into()
}

impl Anchor {
    fn track(&self, button: Stateful<Div>) -> Div {
        let bounds = Rc::clone(&self.0);
        div()
            .flex_none()
            .on_children_prepainted(move |children, _, _| {
                if let Some(button) = children.first() {
                    bounds.set(*button);
                }
            })
            .child(button)
    }

    fn below(&self) -> Point<Pixels> {
        self.0.get().bottom_left() + point(px(0.0), px(SPACING_PX))
    }
}

impl Browser {
    pub(super) fn site_badge(
        &self,
        url: Option<&Url>,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let url = url?;
        if Page::of(url).is_some() {
            return Some(brand_badge(self.colors()).into_any_element());
        }
        let button = self.security_button(security(url)?, cx);
        Some(self.site_anchor.track(button).into_any_element())
    }

    fn security_button(&self, security: Security, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let hover = colors.overlay;
        let glyph = security_icon(security)
            .tinted(rgba(colors.muted))
            .size(px(ICON_PX));
        let button = match security {
            Security::Secure => div()
                .size(px(LOCK_PX))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .child(glyph),
            Security::NotSecure => pill(colors).child(glyph).child(self.text(Text::NotSecure)),
        };
        button
            .id("site-info")
            .cursor_pointer()
            .hover(move |button| button.bg(rgba(hover)))
            .on_click(cx.listener(|browser, _: &ClickEvent, _, cx| {
                cx.stop_propagation();
                browser.show_site_info(cx);
            }))
    }

    fn show_site_info(&mut self, cx: &mut Context<Self>) {
        self.popover = Some(Popover::SiteInfo(self.site_anchor.below()));
        cx.notify();
    }

    fn security_row(&self, security: Security) -> Div {
        let colors = self.colors();
        let (title, hint) = security_text(security);
        div()
            .px_2()
            .py_1()
            .flex()
            .items_start()
            .gap_2()
            .child(security_icon(security).tinted(rgba(colors.muted)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.text(title)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgba(colors.muted))
                            .child(self.text(hint)),
                    ),
            )
    }

    fn state_choice(
        &self,
        setting: &Setting,
        state: State,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let colors = self.colors();
        let hover = colors.overlay;
        let origin = setting.origin.clone();
        let kind = setting.kind;
        let choice = div()
            .id(choice_id(kind, state))
            .h(px(CHOICE_HEIGHT_PX))
            .px_2()
            .flex()
            .items_center()
            .rounded(px(CHOICE_CORNER_PX))
            .text_xs()
            .cursor_pointer()
            .on_click(cx.listener(move |browser, _: &ClickEvent, _, cx| {
                browser.set_permission(&origin, kind, state, cx)
            }))
            .child(self.text(state.label()));
        match state == setting.state {
            true => choice
                .bg(rgba(colors.surface))
                .shadow_sm()
                .font_weight(FontWeight::SEMIBOLD),
            false => choice
                .text_color(rgba(colors.muted))
                .hover(move |choice| choice.bg(rgba(hover))),
        }
    }

    fn permission_row(&self, setting: &Setting, cx: &mut Context<Self>) -> Div {
        let colors = self.colors();
        let choices = State::ALL.map(|state| self.state_choice(setting, state, cx));
        line(icon(setting.kind), self.text(setting.kind.label()), colors).child(
            div()
                .flex()
                .flex_none()
                .p(px(CHOICES_PADDING_PX))
                .gap(px(CHOICES_PADDING_PX))
                .rounded_md()
                .bg(rgba(colors.raised))
                .children(choices),
        )
    }

    fn permission_settings(&self, origin: &Origin, asked: &BTreeSet<Kind>) -> Vec<Setting> {
        let rules = self.runtime.permissions().rules(origin);
        let kinds: BTreeSet<Kind> = rules.keys().chain(asked).copied().collect();
        kinds
            .into_iter()
            .map(|kind| Setting {
                origin: origin.clone(),
                kind,
                state: rules.get(&kind).copied().unwrap_or_default(),
            })
            .collect()
    }

    fn clear_site_data(
        &mut self,
        view: &Entity<WebView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.dismiss_popover(window, cx);
        let Some(cleared) = view.read(cx).clear_site_data() else {
            return;
        };
        let view = view.downgrade();
        cx.spawn(async move |_, cx| match cleared.await {
            Ok(()) => view.update(cx, WebView::reload).unwrap_or_default(),
            Err(error) => eprintln!("magpie: could not clear the site data: {error}"),
        })
        .detach();
    }

    fn clear_site_data_row(&self, view: &Entity<WebView>, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let hover = colors.overlay;
        let view = view.clone();
        line(Icon::Trash, self.text(Text::ClearSiteData), colors)
            .id("site-clear-data")
            .cursor_pointer()
            .hover(move |row| row.bg(rgba(hover)))
            .on_click(cx.listener(move |browser, _: &ClickEvent, window, cx| {
                browser.clear_site_data(&view, window, cx)
            }))
    }

    pub(super) fn site_info(
        &self,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        let view = self.active_view()?.clone();
        let (security, origin, asked) = {
            let page = view.read(cx);
            (security(page.url()?)?, page.origin()?, page.asked().clone())
        };
        let colors = self.colors();
        let rows: Vec<Div> = self
            .permission_settings(&origin, &asked)
            .iter()
            .map(|setting| self.permission_row(setting, cx))
            .collect();
        let card = panel(colors)
            .w(px(WIDTH_PX))
            .child(
                div()
                    .px_2()
                    .pt_1()
                    .truncate()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(origin.host().to_owned()),
            )
            .child(self.security_row(security))
            .when(!rows.is_empty(), |card| {
                card.child(divider(colors))
                    .child(heading(self.text(Text::Permissions), colors))
                    .children(rows)
            })
            .child(divider(colors))
            .child(self.clear_site_data_row(&view, cx));
        Some(
            anchored()
                .position(position)
                .snap_to_window_with_margin(px(WINDOW_MARGIN_PX))
                .child(self.cutouts.see_through(PANEL_CORNER_PX).child(card)),
        )
    }
}
