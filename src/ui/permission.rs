use gpui::{Context, Div, ElementId, Entity, SharedString, Stateful, div, prelude::*, px, rgba};

use super::{
    browser::Browser,
    icon::Icon,
    surface::{Colors, PANEL_CORNER_PX, surface},
    window,
};
use crate::{
    internal::Page,
    locale::Text,
    storage::permissions::{Kind, Origin, State},
    webview::WebView,
};

const WIDTH_PX: f32 = 360.0;
const TOP_PX: f32 = 8.0;
const LEFT_PX: f32 = 8.0;
const BUTTON_HEIGHT_PX: f32 = 28.0;

pub fn icon(kind: Kind) -> Icon {
    match kind {
        Kind::Camera => Icon::Camera,
        Kind::Microphone => Icon::Microphone,
        Kind::Notifications => Icon::Bell,
        Kind::Location => Icon::MapPin,
        Kind::Clipboard => Icon::Clipboard,
        Kind::Popups => Icon::AppWindow,
    }
}

fn answer_button(id: impl Into<ElementId>, label: &'static str, colors: &Colors) -> Stateful<Div> {
    let hover = colors.overlay;
    div()
        .id(id)
        .h(px(BUTTON_HEIGHT_PX))
        .px_3()
        .flex()
        .items_center()
        .rounded_md()
        .bg(rgba(colors.raised))
        .border_1()
        .border_color(rgba(colors.line))
        .cursor_pointer()
        .hover(move |button| button.bg(rgba(hover)))
        .child(label)
}

impl Browser {
    pub(super) fn set_permission(
        &self,
        origin: &Origin,
        kind: Kind,
        state: State,
        cx: &mut Context<Self>,
    ) {
        self.runtime.set_permission(origin, kind, state);
        window::each_browser(cx, Self::apply_permissions);
    }

    fn apply_permissions(&mut self, cx: &mut Context<Self>) {
        for pane in self.live.values().flatten() {
            pane.view.update(cx, WebView::apply_permissions);
        }
        self.refresh_pages(&[Page::Settings], cx);
        cx.notify();
    }

    fn remember_answer(&self, view: &Entity<WebView>, state: State, cx: &mut Context<Self>) {
        let prompt = view.read(cx).prompt();
        let Some((origin, kind)) = prompt.map(|prompt| (prompt.origin.clone(), prompt.kind)) else {
            return;
        };
        self.set_permission(&origin, kind, state, cx);
    }

    fn remembering(
        &self,
        view: &Entity<WebView>,
        state: State,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let view = view.clone();
        let id = SharedString::from(format!("permission-{}", state.name()));
        answer_button(id, self.text(state.label()), self.colors()).on_click(
            cx.listener(move |browser, _, _, cx| browser.remember_answer(&view, state, cx)),
        )
    }

    fn allowing_once(&self, view: &Entity<WebView>, cx: &mut Context<Self>) -> Stateful<Div> {
        let view = view.clone();
        answer_button(
            "permission-allow-once",
            self.text(Text::AllowOnce),
            self.colors(),
        )
        .on_click(cx.listener(move |_, _, _, cx| view.update(cx, WebView::allow_once)))
    }

    pub(super) fn permission_card(
        &self,
        view: &Entity<WebView>,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        let prompt = view.read(cx).prompt()?;
        let colors = self.colors();
        let question = self
            .locale()
            .fill(prompt.kind.request(), prompt.origin.host());
        let kind = prompt.kind;
        let card = surface(colors)
            .w(px(WIDTH_PX))
            .p_3()
            .flex()
            .flex_col()
            .gap_3()
            .rounded(px(PANEL_CORNER_PX))
            .occlude()
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_2()
                    .child(icon(kind).tinted(rgba(colors.muted)))
                    .child(div().flex_1().min_w_0().child(question)),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(self.remembering(view, State::Block, cx))
                    .child(self.allowing_once(view, cx))
                    .child(self.remembering(view, State::Allow, cx)),
            );
        Some(
            self.cutouts
                .see_through(PANEL_CORNER_PX)
                .absolute()
                .top(px(TOP_PX))
                .left(px(LEFT_PX))
                .child(card),
        )
    }
}
