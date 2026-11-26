use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, KeyBinding, MouseButton,
    Subscription, Window, actions, anchored, deferred, div, point, prelude::*, px, rgba,
};

use super::{
    browser::Browser,
    context_menu::Target,
    favicon::site_icon,
    field::{Changed, Field},
    frame::INSET_PX,
    surface::{CONTROL_CORNER_PX, PANEL_CORNER_PX, ROW_HEIGHT_PX, surface},
};
use crate::{
    address::address,
    config::FavoritesSize,
    storage::session::{Link, SpaceId},
};

actions!(favorite, [SaveEdit, CancelEdit, SwitchField]);

const CONTEXT: &str = "FavoriteEdit";
const LABEL_GAP_PX: f32 = 4.0;
const LABEL_MAX_WIDTH_PX: f32 = 240.0;
const WINDOW_MARGIN_PX: f32 = 8.0;

pub struct Edit {
    space: SpaceId,
    url: String,
    title: Entity<Field>,
    address: Entity<Field>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

#[derive(Default)]
pub struct Tiles {
    hovered: Option<usize>,
    edit: Option<Edit>,
}

impl Tiles {
    pub fn leave(&mut self) {
        self.hovered = None;
    }
}

pub fn bindings() -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    vec![
        KeyBinding::new("enter", SaveEdit, context),
        KeyBinding::new("escape", CancelEdit, context),
        KeyBinding::new("tab", SwitchField, context),
        KeyBinding::new("shift-tab", SwitchField, context),
    ]
}

impl Tiles {
    fn editing(&self, link: &Link) -> bool {
        self.edit.as_ref().is_some_and(|edit| edit.url == link.url)
    }
}

impl Edit {
    fn edited(&self, cx: &App) -> Option<Link> {
        let url = address(self.address.read(cx).text())?;
        Some(Link {
            url: url.into(),
            title: self.title.read(cx).text().to_owned(),
        })
    }
}

impl Browser {
    pub(super) fn start_editing_favorite(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let space = self.current().id;
        let Some(link) = self.current().favorites.get(index).cloned() else {
            return;
        };
        let title = cx.new(|cx| Field::new("Name", &link.title, cx).with_all_selected());
        let address = cx.new(|cx| Field::new("URL", &link.url, cx));
        let focus = cx.focus_handle();
        let subscriptions = vec![
            cx.on_focus_out(&focus, window, |browser, _, _, cx| {
                browser.finish_editing_favorite(cx)
            }),
            cx.subscribe(&title, |_, _, _: &Changed, cx| cx.notify()),
            cx.subscribe(&address, |_, _, _: &Changed, cx| cx.notify()),
        ];
        title.focus_handle(cx).focus(window);
        self.tiles.edit = Some(Edit {
            space,
            url: link.url,
            title,
            address,
            focus,
            _subscriptions: subscriptions,
        });
        cx.notify();
    }

    fn finish_editing_favorite(&mut self, cx: &mut Context<Self>) {
        let Some(edit) = self.tiles.edit.take() else {
            return;
        };
        if let Some(edited) = edit.edited(cx) {
            self.edit_session(
                |session| session.edit_favorite(edit.space, &edit.url, edited),
                cx,
            );
        }
        cx.notify();
    }

    fn save_favorite(&mut self, _: &SaveEdit, window: &mut Window, cx: &mut Context<Self>) {
        let valid = self
            .tiles
            .edit
            .as_ref()
            .is_some_and(|edit| edit.edited(cx).is_some());
        if valid {
            self.finish_editing_favorite(cx);
            self.focus.focus(window);
        }
    }

    fn cancel_favorite_edit(
        &mut self,
        _: &CancelEdit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.tiles.edit = None;
        self.focus.focus(window);
        cx.notify();
    }

    fn switch_favorite_field(
        &mut self,
        _: &SwitchField,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(edit) = &self.tiles.edit else {
            return;
        };
        let title = edit.title.focus_handle(cx);
        let next = match title.is_focused(window) {
            true => edit.address.focus_handle(cx),
            false => title,
        };
        next.focus(window);
    }

    fn hover_favorite(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        let leaving = !hovered && self.tiles.hovered == Some(index);
        match (hovered, leaving) {
            (true, _) => self.tiles.hovered = Some(index),
            (false, true) => self.tiles.hovered = None,
            (false, false) => return,
        }
        cx.notify();
    }

    fn name_shown(&self, index: usize, link: &Link, cx: &Context<Self>) -> bool {
        self.tiles.hovered == Some(index)
            && !self.tiles.editing(link)
            && self.popover.is_none()
            && !cx.has_active_drag()
    }

    fn name_label(&self, name: &str) -> Div {
        let label = surface(self.colors())
            .max_w(px(LABEL_MAX_WIDTH_PX))
            .px_2()
            .py_0p5()
            .rounded(px(CONTROL_CORNER_PX))
            .text_xs()
            .truncate()
            .child(name.to_owned());
        let floating = anchored()
            .offset(point(px(0.0), px(LABEL_GAP_PX)))
            .snap_to_window_with_margin(px(WINDOW_MARGIN_PX))
            .child(self.cutouts.see_through(CONTROL_CORNER_PX).child(label));
        div()
            .absolute()
            .top_full()
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(deferred(floating))
    }

    fn tile(&self, index: usize, link: &Link, cx: &mut Context<Self>) -> AnyElement {
        let colors = self.colors();
        let (rest, hover) = (colors.surface, colors.surface_hover);
        let background = match self.tiles.editing(link) {
            true => hover,
            false => rest,
        };
        let name = self
            .name_shown(index, link, cx)
            .then(|| self.name_label(&link.title));
        let draggable = self.draggable(div().id(("favorite", index)), index, link.title.clone());
        self.favorite_drop_slot(draggable, index, cx)
            .h(px(self.layout().favorites_size.tile_height_px()))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(CONTROL_CORNER_PX))
            .bg(rgba(background))
            .hover(move |style| style.bg(rgba(hover)))
            .on_hover(cx.listener(move |browser, hovered: &bool, _, cx| {
                browser.hover_favorite(index, *hovered, cx)
            }))
            .on_click(cx.listener(move |browser, _, _, cx| browser.open_favorite(index, cx)))
            .on_mouse_down(
                MouseButton::Right,
                self.context_menu_opener(Target::Favorite(index), cx),
            )
            .child(site_icon(&self.favicons, &link.url, rgba(colors.muted), cx))
            .children(name)
            .into_any_element()
    }

    fn editor_field(&self, field: &Entity<Field>, text_color: u32) -> Div {
        div()
            .h(px(ROW_HEIGHT_PX))
            .px_2()
            .flex()
            .items_center()
            .rounded(px(CONTROL_CORNER_PX))
            .bg(rgba(self.colors().raised))
            .text_color(rgba(text_color))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(field.clone()),
            )
    }

    fn editor(&self, edit: &Edit, cx: &mut Context<Self>) -> Div {
        let colors = self.colors();
        let address_color = match edit.edited(cx) {
            Some(_) => colors.text,
            None => colors.faint,
        };
        div()
            .key_context(CONTEXT)
            .track_focus(&edit.focus)
            .on_action(cx.listener(Self::save_favorite))
            .on_action(cx.listener(Self::cancel_favorite_edit))
            .on_action(cx.listener(Self::switch_favorite_field))
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .p_1()
            .flex()
            .flex_col()
            .gap_1()
            .rounded(px(PANEL_CORNER_PX))
            .bg(rgba(colors.surface))
            .child(self.editor_field(&edit.title, colors.text))
            .child(self.editor_field(&edit.address, address_color))
    }

    pub(super) fn favorites_grid(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let layout = self.layout();
        let row_px = self.sidebar_width_px() - 2.0 * INSET_PX;
        let tiles: Vec<AnyElement> = self
            .current()
            .favorites
            .iter()
            .enumerate()
            .map(|(index, link)| self.tile(index, link, cx))
            .collect();
        let grid = div()
            .grid()
            .grid_cols(layout.favorites_columns.fitting(row_px))
            .gap(px(FavoritesSize::GAP_PX))
            .children(tiles);
        let edit = self
            .tiles
            .edit
            .as_ref()
            .filter(|edit| edit.space == self.current().id);
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.favorites_drop_zone(grid, cx))
            .children(edit.map(|edit| self.editor(edit, cx)))
    }
}
