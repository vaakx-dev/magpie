use gpui::{
    ClickEvent, Context, CursorStyle, Div, ElementId, Entity, Focusable, Hsla, KeyBinding,
    MouseButton, Stateful, Subscription, Window, actions, div, prelude::*, px, rgba,
    transparent_black,
};
use url::Url;

use super::{
    Back, Forward, Reload, ToggleSidebar,
    browser::Browser,
    command::{Mode, Target},
    field::Field,
    icon::Icon,
    password::Phase,
    sidebar::Density,
    surface::{CONTROL_CORNER_PX, Colors, tint},
    window,
};
use crate::{
    address::destination,
    config::{Side, Sidebar},
    internal::Page,
    locale::Text,
    webview::autofill::fillable,
};

actions!(address_bar, [Navigate, CancelEdit]);

pub struct Edit {
    field: Entity<Field>,
    _blur: Subscription,
}

pub const BUTTON_PX: f32 = 28.0;
const NAVIGATION_GAP_PX: f32 = 4.0;
const NAVIGATION_BUTTONS: f32 = 3.0;
pub const NAVIGATION_WIDTH_PX: f32 =
    NAVIGATION_BUTTONS * BUTTON_PX + (NAVIGATION_BUTTONS - 1.0) * NAVIGATION_GAP_PX;
pub const ADDRESS_HEIGHT_PX: f32 = 30.0;
const INNER_BUTTON_PX: f32 = 24.0;
const INNER_BUTTON_INSET_PX: f32 = (ADDRESS_HEIGHT_PX - INNER_BUTTON_PX) / 2.0;
const CONTEXT: &str = "AddressBar";
const EDIT_RING_ALPHA: u8 = 0xcc;

pub fn bindings() -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    vec![
        KeyBinding::new("enter", Navigate, context),
        KeyBinding::new("escape", CancelEdit, context),
    ]
}

fn short_address(url: &Url) -> String {
    let host = url
        .host_str()
        .unwrap_or_default()
        .trim_start_matches("www.");
    let path = url.path().trim_end_matches('/');
    format!("{host}{path}")
}

fn shown_address(url: &Url) -> String {
    match Page::of(url) {
        Some(_) => url.to_string(),
        None => short_address(url),
    }
}

fn plain_button(id: impl Into<ElementId>, icon: Icon, color: Hsla) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(BUTTON_PX))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(CONTROL_CORNER_PX))
        .child(icon.tinted(color))
}

pub fn icon_button(id: impl Into<ElementId>, icon: Icon, colors: &Colors) -> Stateful<Div> {
    let hover = colors.overlay;
    plain_button(id, icon, rgba(colors.text).into()).hover(move |button| button.bg(rgba(hover)))
}

fn nav_button(id: &'static str, icon: Icon, enabled: bool, colors: &Colors) -> Stateful<Div> {
    match enabled {
        true => icon_button(id, icon, colors),
        false => plain_button(id, icon, rgba(colors.faint).into()),
    }
}

impl Browser {
    pub(super) fn navigation(&self, cx: &mut Context<Self>) -> Div {
        let view = self.active_view().map(|view| view.read(cx));
        let back = view.is_some_and(|view| view.can_go_back());
        let forward = view.is_some_and(|view| view.can_go_forward());
        let colors = self.colors();
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(NAVIGATION_GAP_PX))
            .child(
                nav_button("back", Icon::ArrowLeft, back, colors).on_click(
                    cx.listener(|browser, _, window, cx| browser.back(&Back, window, cx)),
                ),
            )
            .child(
                nav_button("forward", Icon::ArrowRight, forward, colors).on_click(
                    cx.listener(|browser, _, window, cx| browser.forward(&Forward, window, cx)),
                ),
            )
            .child(self.reload_button(cx))
    }

    pub(super) fn sidebar_button(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let icon = match self.layout().sidebar_side {
            Side::Left => Icon::PanelLeft,
            Side::Right => Icon::PanelRight,
        };
        icon_button("toggle-sidebar", icon, self.colors()).on_click(
            cx.listener(|browser, _, window, cx| {
                browser.toggle_sidebar(&ToggleSidebar, window, cx)
            }),
        )
    }

    fn reload_button(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let view = self.active_view().map(|view| view.read(cx));
        let loading = view.is_some_and(|view| view.loading());
        let reloadable = view.is_some_and(|view| view.url().is_some());
        let (id, icon) = match loading {
            true => ("stop", Icon::X),
            false => ("reload", Icon::RotateCw),
        };
        nav_button(id, icon, reloadable, self.colors()).on_click(cx.listener(
            move |browser, _: &ClickEvent, window, cx| match loading {
                true => browser.stop(cx),
                false => browser.reload(&Reload, window, cx),
            },
        ))
    }

    fn favorited(&self) -> bool {
        let space = self.current();
        space.active_tab().is_some_and(|tab| {
            let url = &tab.focused().url;
            space.favorites.iter().any(|link| &link.url == url)
        })
    }

    fn favorite_button(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let open = self.current().active_tab().is_some();
        let star: Hsla = match (open, self.favorited()) {
            (false, _) => rgba(colors.faint).into(),
            (true, true) => rgba(colors.text).into(),
            (true, false) => rgba(colors.muted).into(),
        };
        let cursor = match open {
            true => CursorStyle::PointingHand,
            false => CursorStyle::Arrow,
        };
        let hover = colors.overlay;
        plain_button("favorite-page", Icon::Star, star)
            .size(px(INNER_BUTTON_PX))
            .cursor(cursor)
            .when(open, |button| {
                button.hover(move |button| button.bg(rgba(hover)))
            })
            .on_click(cx.listener(|browser, _: &ClickEvent, _, cx| {
                cx.stop_propagation();
                browser.toggle_favorite(cx);
            }))
    }

    fn key_button(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let view = self.active_view()?.read(cx);
        let page = view.url()?;
        let saved = || match window::keyring(cx).read(cx).phase() {
            Phase::Unlocked(vault) => !vault.matches_for_url(page.as_str()).is_empty(),
            _ => false,
        };
        if !fillable(page) || !(view.login_form() || saved()) {
            return None;
        }
        let colors = self.colors();
        let hover = colors.overlay;
        let color = match view.login_field().is_some() {
            true => colors.text,
            false => colors.muted,
        };
        Some(
            plain_button("passwords-key", Icon::KeyRound, rgba(color).into())
                .size(px(INNER_BUTTON_PX))
                .cursor_pointer()
                .when(self.passwords_shown(), |button| button.bg(rgba(hover)))
                .hover(move |button| button.bg(rgba(hover)))
                .on_click(cx.listener(|browser, _: &ClickEvent, window, cx| {
                    cx.stop_propagation();
                    browser.toggle_passwords(window, cx);
                })),
        )
    }

    fn address_bar_shown(&self) -> bool {
        let layout = self.layout();
        let roomy_sidebar = layout.sidebar == Sidebar::Shown && self.density() == Density::Roomy;
        layout.top_bar.on() || roomy_sidebar
    }

    fn current_address(&self, cx: &Context<Self>) -> String {
        self.active_view()
            .and_then(|view| view.read(cx).url().map(Url::to_string))
            .unwrap_or_default()
    }

    pub(super) fn edit_address(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let address = self.current_address(cx);
        match self.address_bar_shown() {
            true => self.start_editing_address(&address, window, cx),
            false => self.show_palette(Mode::Everything(Target::Current), address, window, cx),
        }
    }

    fn start_editing_address(
        &mut self,
        address: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let placeholder = self.text(Text::SearchOrEnterAddress);
        let field = cx.new(|cx| Field::new(placeholder, address, cx));
        let focus = field.focus_handle(cx);
        let blur = cx.on_blur(&focus, window, |browser, _, cx| {
            browser.stop_editing_address(cx)
        });
        focus.focus(window);
        self.address_edit = Some(Edit { field, _blur: blur });
        cx.notify();
    }

    pub(super) fn stop_editing_address(&mut self, cx: &mut Context<Self>) {
        self.address_edit = None;
        cx.notify();
    }

    fn navigate(&mut self, _: &Navigate, window: &mut Window, cx: &mut Context<Self>) {
        let Some(edit) = self.address_edit.take() else {
            return;
        };
        let typed = edit.field.read(cx).text().trim().to_owned();
        self.focus.focus(window);
        cx.notify();
        if typed.is_empty() {
            return;
        }
        let url = destination(&typed, &self.settings().general.search).url();
        self.open(url, Target::Current, cx);
    }

    fn cancel_edit(&mut self, _: &CancelEdit, window: &mut Window, cx: &mut Context<Self>) {
        self.stop_editing_address(cx);
        self.focus.focus(window);
    }

    fn address_editor(
        &self,
        bar: Stateful<Div>,
        edit: &Edit,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        bar.key_context(CONTEXT)
            .on_action(cx.listener(Self::navigate))
            .on_action(cx.listener(Self::cancel_edit))
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .border_color(tint(self.colors().muted, EDIT_RING_ALPHA))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(edit.field.clone()),
            )
    }

    fn address_display(&self, bar: Stateful<Div>, cx: &mut Context<Self>) -> Stateful<Div> {
        let url = self
            .active_view()
            .and_then(|view| view.read(cx).url().cloned());
        let address = url.as_ref().map_or_else(
            || self.text(Text::SearchOrEnterAddress).to_owned(),
            shown_address,
        );
        let colors = self.colors();
        let hover = colors.surface_hover;
        bar.hover(move |bar| bar.bg(rgba(hover)))
            .cursor_text()
            .on_click(
                cx.listener(|browser, _: &ClickEvent, window, cx| browser.edit_address(window, cx)),
            )
            .children(self.site_badge(url.as_ref(), cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_color(rgba(colors.text))
                    .child(address),
            )
            .children(self.zoom_badge(cx))
            .children(self.key_button(cx))
            .child(self.favorite_button(cx))
    }

    pub(super) fn address_bar(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let bar = div()
            .id("address-bar")
            .h(px(ADDRESS_HEIGHT_PX))
            .pl_2()
            .pr(px(INNER_BUTTON_INSET_PX))
            .flex()
            .items_center()
            .gap_2()
            .rounded(px(CONTROL_CORNER_PX))
            .border_1()
            .border_color(transparent_black())
            .bg(rgba(self.colors().surface));
        match &self.address_edit {
            Some(edit) => self.address_editor(bar, edit, cx),
            None => self.address_display(bar, cx),
        }
    }
}
