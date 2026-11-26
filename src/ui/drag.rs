use gpui::{
    App, Bounds, Context, Div, DragMoveEvent, Pixels, Rgba, SharedString, Stateful, Window, div,
    prelude::*, px, rgba,
};

use super::{
    browser::Browser,
    surface::{CONTROL_CORNER_PX, Colors, surface, tint},
};
use crate::{
    config::FavoritesSize,
    storage::{
        arrange::{Item, Spot},
        session::SpaceId,
    },
};

const PREVIEW_MAX_WIDTH_PX: f32 = 220.0;
const LINE_THICKNESS_PX: f32 = 2.0;
const FAVORITE_LINE_OFFSET_PX: f32 = -(FavoritesSize::GAP_PX + LINE_THICKNESS_PX) / 2.0;
const INSIDE_ALPHA: u8 = 0x33;
const SPACE_TARGET_ALPHA: u8 = 0x40;

#[derive(Clone)]
pub struct Dragged<T> {
    payload: T,
    label: SharedString,
    colors: &'static Colors,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Hint {
    spot: Spot,
    bounds: Bounds<Pixels>,
}

#[derive(Clone, Copy)]
pub struct Zones {
    pub upper: Spot,
    pub lower: Spot,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Half {
    Leading,
    Trailing,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FavoriteSpot {
    index: usize,
    half: Half,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    Above,
    Below,
    Inside,
}

impl<T: 'static> Render for Dragged<T> {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        surface(self.colors)
            .max_w(px(PREVIEW_MAX_WIDTH_PX))
            .px_3()
            .py_1()
            .rounded(px(CONTROL_CORNER_PX))
            .font_family("Segoe UI")
            .text_sm()
            .truncate()
            .child(self.label.clone())
    }
}

impl Zones {
    pub fn whole(spot: Spot) -> Self {
        Self {
            upper: spot,
            lower: spot,
        }
    }

    fn spot(&self, event: &DragMoveEvent<Dragged<Item>>) -> Spot {
        match event.event.position.y > event.bounds.center().y {
            true => self.lower,
            false => self.upper,
        }
    }

    fn mark(&self, spot: Spot) -> Option<Mark> {
        match spot {
            Spot::Into(_) if spot == self.lower => Some(Mark::Inside),
            _ if spot == self.lower => Some(Mark::Below),
            _ if spot == self.upper => Some(Mark::Above),
            _ => None,
        }
    }
}

impl FavoriteSpot {
    fn at(index: usize, event: &DragMoveEvent<Dragged<usize>>) -> Self {
        let half = match event.event.position.x > event.bounds.center().x {
            true => Half::Trailing,
            false => Half::Leading,
        };
        Self { index, half }
    }

    fn slot(self) -> usize {
        match self.half {
            Half::Leading => self.index,
            Half::Trailing => self.index + 1,
        }
    }
}

fn line(mark: Mark, color: Rgba) -> Option<Div> {
    let line = div()
        .absolute()
        .left_0()
        .right_0()
        .h(px(LINE_THICKNESS_PX))
        .rounded_full()
        .bg(color);
    match mark {
        Mark::Above => Some(line.top_0()),
        Mark::Below => Some(line.bottom_0()),
        Mark::Inside => None,
    }
}

fn favorite_line(half: Half, color: Rgba) -> Div {
    let line = div()
        .absolute()
        .top_0()
        .bottom_0()
        .w(px(LINE_THICKNESS_PX))
        .rounded_full()
        .bg(color);
    match half {
        Half::Leading => line.left(px(FAVORITE_LINE_OFFSET_PX)),
        Half::Trailing => line.right(px(FAVORITE_LINE_OFFSET_PX)),
    }
}

impl Browser {
    fn hint_at(
        &mut self,
        zones: Zones,
        event: &DragMoveEvent<Dragged<Item>>,
        cx: &mut Context<Self>,
    ) {
        let inside = event.bounds.contains(&event.event.position);
        let owned = self
            .drop_hint
            .is_some_and(|hint| hint.bounds == event.bounds);
        let hint = Hint {
            spot: zones.spot(event),
            bounds: event.bounds,
        };
        match (inside, owned) {
            (true, _) if self.drop_hint != Some(hint) => self.drop_hint = Some(hint),
            (false, true) => self.drop_hint = None,
            _ => return,
        }
        cx.notify();
    }

    fn mark(&self, zones: Zones, cx: &App) -> Option<Mark> {
        let hint = self.drop_hint.filter(|_| cx.has_active_drag())?;
        zones.mark(hint.spot)
    }

    fn drop_dragged(&mut self, dragged: &Dragged<Item>, cx: &mut Context<Self>) {
        let Some(hint) = self.drop_hint.take() else {
            return;
        };
        if self.current_mut().arrange(dragged.payload, hint.spot) {
            self.save_session(cx);
        }
        cx.notify();
    }

    fn drop_on_space(&mut self, dragged: &Dragged<Item>, space: SpaceId, cx: &mut Context<Self>) {
        self.move_to_space(dragged.payload, space, cx);
    }

    fn favorite_hint_at(
        &mut self,
        index: usize,
        event: &DragMoveEvent<Dragged<usize>>,
        cx: &mut Context<Self>,
    ) {
        let inside = event.bounds.contains(&event.event.position);
        let spot = Some(FavoriteSpot::at(index, event));
        if inside && self.favorite_hint != spot {
            self.favorite_hint = spot;
            cx.notify();
        }
    }

    fn leave_favorites(&mut self, event: &DragMoveEvent<Dragged<usize>>, cx: &mut Context<Self>) {
        let outside = !event.bounds.contains(&event.event.position);
        if outside && self.favorite_hint.take().is_some() {
            cx.notify();
        }
    }

    fn drop_favorite(&mut self, dragged: &Dragged<usize>, cx: &mut Context<Self>) {
        let Some(hint) = self.favorite_hint.take() else {
            return;
        };
        if self
            .current_mut()
            .move_favorite(dragged.payload, hint.slot())
        {
            self.save_session(cx);
        }
        cx.notify();
    }

    pub(super) fn draggable<T: Clone + 'static>(
        &self,
        element: Stateful<Div>,
        payload: T,
        label: impl Into<SharedString>,
    ) -> Stateful<Div> {
        let dragged = Dragged {
            payload,
            label: label.into(),
            colors: self.colors(),
        };
        element.on_drag(dragged, |dragged, _, _, cx| cx.new(|_| dragged.clone()))
    }

    pub(super) fn drop_zone<E>(&self, element: E, zones: Zones, cx: &mut Context<Self>) -> E
    where
        E: InteractiveElement + ParentElement + Styled + FluentBuilder,
    {
        let colors = self.colors();
        let (ink, indicator) = (colors.text, rgba(colors.muted));
        let mark = self.mark(zones, cx);
        element
            .relative()
            .when(mark == Some(Mark::Inside), |element| {
                element.bg(tint(ink, INSIDE_ALPHA))
            })
            .children(mark.and_then(|mark| line(mark, indicator)))
            .on_drag_move(
                cx.listener(move |browser, event, _, cx| browser.hint_at(zones, event, cx)),
            )
            .on_drop(cx.listener(|browser, dragged, _, cx| browser.drop_dragged(dragged, cx)))
    }

    pub(super) fn space_drop_zone(
        &self,
        button: Stateful<Div>,
        space: SpaceId,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let ink = self.colors().text;
        button
            .drag_over::<Dragged<Item>>(move |style, _, _, _| {
                style.bg(tint(ink, SPACE_TARGET_ALPHA))
            })
            .on_drop(
                cx.listener(move |browser, dragged, _, cx| {
                    browser.drop_on_space(dragged, space, cx)
                }),
            )
    }

    pub(super) fn favorite_drop_slot(
        &self,
        tile: Stateful<Div>,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let half = self
            .favorite_hint
            .filter(|spot| spot.index == index && cx.has_active_drag())
            .map(|spot| spot.half);
        tile.relative()
            .children(half.map(|half| favorite_line(half, rgba(self.colors().muted))))
            .on_drag_move(
                cx.listener(move |browser, event, _, cx| {
                    browser.favorite_hint_at(index, event, cx)
                }),
            )
    }

    pub(super) fn favorites_drop_zone(&self, grid: Div, cx: &mut Context<Self>) -> Div {
        grid.on_drag_move(cx.listener(|browser, event, _, cx| browser.leave_favorites(event, cx)))
            .on_drop(cx.listener(|browser, dragged, _, cx| browser.drop_favorite(dragged, cx)))
    }
}
