use gpui::{
    AppContext, Context, Entity, Focusable, KeyBinding, MouseButton, Subscription, Window, actions,
    div, prelude::*, px, rgba,
};

use super::{
    Find,
    browser::Browser,
    field::{Changed, Field},
    icon::Icon,
    surface::PANEL_CORNER_PX,
    toolbar::icon_button,
};
use crate::{locale::Text, webview::WebView};

actions!(find, [FindNext, FindPrevious, CloseFind]);

pub struct Card {
    field: Entity<Field>,
    view: Entity<WebView>,
    _changes: Subscription,
}

const CONTEXT: &str = "Find";
const WIDTH_PX: f32 = 380.0;
const HEIGHT_PX: f32 = 36.0;
const TOP_PX: f32 = 8.0;
const RIGHT_PX: f32 = 24.0;
const COUNTER_WIDTH_PX: f32 = 72.0;

pub fn bindings() -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    vec![
        KeyBinding::new("enter", FindNext, context),
        KeyBinding::new("shift-enter", FindPrevious, context),
        KeyBinding::new("escape", CloseFind, context),
    ]
}

impl Card {
    fn new(view: Entity<WebView>, placeholder: &str, cx: &mut Context<Browser>) -> Self {
        let field = cx.new(|cx| Field::new(placeholder, "", cx));
        let searched = view.clone();
        let changes = cx.subscribe(&field, move |_, field, _: &Changed, cx| {
            let query = field.read(cx).text().to_owned();
            searched.update(cx, |view, cx| view.find(&query, cx));
        });
        Self {
            field,
            view,
            _changes: changes,
        }
    }

    fn counter(&self, cx: &Context<Browser>) -> String {
        let searching = !self.field.read(cx).text().is_empty();
        let matches = self.view.read(cx).matches().filter(|_| searching);
        matches.map(|matches| matches.label()).unwrap_or_default()
    }

    fn close(self, cx: &mut Context<Browser>) {
        self.view.update(cx, WebView::stop_find);
    }
}

impl Browser {
    pub(super) fn open_find(&mut self, _: &Find, window: &mut Window, cx: &mut Context<Self>) {
        let Some(view) = self.active_view().cloned() else {
            return;
        };
        if self.find.as_ref().is_none_or(|card| card.view != view) {
            self.stop_find(cx);
            let placeholder = self.text(Text::FindInPage);
            self.find = Some(Card::new(view.clone(), placeholder, cx));
        }
        view.read(cx).blur();
        if let Some(card) = &self.find {
            card.field.focus_handle(cx).focus(window);
        }
        cx.notify();
    }

    fn stop_find(&mut self, cx: &mut Context<Self>) {
        if let Some(card) = self.find.take() {
            card.close(cx);
        }
    }

    pub(super) fn drop_stale_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active = self.active_view();
        if self
            .find
            .as_ref()
            .is_some_and(|card| Some(&card.view) != active)
        {
            self.stop_find(cx);
            self.focus.focus(window);
        }
    }

    fn close_find(&mut self, _: &CloseFind, window: &mut Window, cx: &mut Context<Self>) {
        self.stop_find(cx);
        self.focus.focus(window);
        cx.notify();
    }

    fn find_next(&mut self, _: &FindNext, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(card) = &self.find {
            card.view.update(cx, WebView::find_next);
        }
    }

    fn find_previous(&mut self, _: &FindPrevious, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(card) = &self.find {
            card.view.update(cx, WebView::find_previous);
        }
    }

    pub(super) fn find_card(
        &self,
        view: &Entity<WebView>,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        let card = self.find.as_ref().filter(|card| &card.view == view)?;
        let colors = self.colors();
        let muted = rgba(colors.muted);
        let field = card.field.clone();
        let focused = field.focus_handle(cx);
        let bar = div()
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::find_next))
            .on_action(cx.listener(Self::find_previous))
            .on_action(cx.listener(Self::close_find))
            .on_mouse_down(MouseButton::Left, move |_, window, _| focused.focus(window))
            .w(px(WIDTH_PX))
            .h(px(HEIGHT_PX))
            .flex()
            .items_center()
            .gap_1()
            .pl_3()
            .pr_1()
            .rounded(px(PANEL_CORNER_PX))
            .bg(rgba(colors.surface))
            .border_1()
            .border_color(rgba(colors.line))
            .child(Icon::Search.tinted(muted))
            .child(div().flex_1().min_w_0().px_1().child(field))
            .child(
                div()
                    .w(px(COUNTER_WIDTH_PX))
                    .flex_none()
                    .text_xs()
                    .text_right()
                    .text_color(muted)
                    .child(card.counter(cx)),
            )
            .child(
                icon_button("find-previous", Icon::ChevronUp, colors).on_click(cx.listener(
                    |browser, _, window, cx| browser.find_previous(&FindPrevious, window, cx),
                )),
            )
            .child(
                icon_button("find-next", Icon::ChevronDown, colors).on_click(
                    cx.listener(|browser, _, window, cx| browser.find_next(&FindNext, window, cx)),
                ),
            )
            .child(icon_button("find-close", Icon::X, colors).on_click(
                cx.listener(|browser, _, window, cx| browser.close_find(&CloseFind, window, cx)),
            ));
        Some(
            self.cutouts
                .see_through(PANEL_CORNER_PX)
                .absolute()
                .top(px(TOP_PX))
                .right(px(RIGHT_PX))
                .child(bar),
        )
    }
}
