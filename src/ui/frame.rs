use gpui::{
    AnyElement, Div, Font, FontFallbacks, Stateful, Window, WindowControlArea, div, font,
    prelude::*, px, rgba, transparent_black,
};

use super::surface::{CONTROL_CORNER_PX, Colors};
use crate::config::{ControlStyle, Controls, Side};

pub const INSET_PX: f32 = 6.0;
pub const GAP_PX: f32 = 4.0;
const BUTTON_COUNT: f32 = 3.0;
const BUTTON_WIDTH_PX: f32 = 28.0;
const BUTTON_HEIGHT_PX: f32 = 28.0;
const GLYPH_PX: f32 = 10.0;
const LIGHT_SLOT_PX: f32 = 20.0;
const LIGHT_PX: f32 = 12.0;
const LIGHT_SYMBOL_PX: f32 = 7.0;
const CONTROLS_GROUP: &str = "window-controls";
const LIGHT_PRESSED_OPACITY: f32 = 0.75;
const ADD: &str = "\u{E710}";
const CAPTION_FONT: &str = "Segoe Fluent Icons";
const CAPTION_FALLBACK_FONT: &str = "Segoe MDL2 Assets";
const MINIMIZE: &str = "\u{E921}";
const MAXIMIZE: &str = "\u{E922}";
const RESTORE: &str = "\u{E923}";
const CLOSE: &str = "\u{E8BB}";

#[derive(Clone, Copy)]
pub enum Flow {
    Row,
    Column,
}

fn caption_font() -> Font {
    let fallbacks = FontFallbacks::from_fonts(vec![CAPTION_FALLBACK_FONT.to_owned()]);
    Font {
        fallbacks: Some(fallbacks),
        ..font(CAPTION_FONT)
    }
}

fn button(id: &'static str, area: WindowControlArea, glyph: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .w(px(BUTTON_WIDTH_PX))
        .h(px(BUTTON_HEIGHT_PX))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(CONTROL_CORNER_PX))
        .font(caption_font())
        .text_size(px(GLYPH_PX))
        .window_control_area(area)
        .child(glyph)
}

fn caption(
    id: &'static str,
    area: WindowControlArea,
    glyph: &'static str,
    colors: &Colors,
) -> Stateful<Div> {
    let (hover, pressed) = (colors.overlay, colors.overlay_pressed);
    button(id, area, glyph)
        .hover(move |button| button.bg(rgba(hover)))
        .active(move |button| button.bg(rgba(pressed)))
}

fn windows_buttons(colors: &Colors, window: &Window) -> [Stateful<Div>; 3] {
    let maximize = match window.is_maximized() {
        true => RESTORE,
        false => MAXIMIZE,
    };
    [
        caption("minimize", WindowControlArea::Min, MINIMIZE, colors),
        caption("maximize", WindowControlArea::Max, maximize, colors),
        caption("close-window", WindowControlArea::Close, CLOSE, colors),
    ]
}

fn light(
    id: &'static str,
    area: WindowControlArea,
    symbol: &'static str,
    colors: &Colors,
) -> Stateful<Div> {
    let symbol_color = colors.on_text;
    let dot = div()
        .size(px(LIGHT_PX))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(rgba(colors.muted))
        .font(caption_font())
        .text_size(px(LIGHT_SYMBOL_PX))
        .text_color(transparent_black())
        .group_hover(CONTROLS_GROUP, move |dot| {
            dot.text_color(rgba(symbol_color))
        })
        .child(symbol);
    div()
        .id(id)
        .w(px(LIGHT_SLOT_PX))
        .h(px(BUTTON_HEIGHT_PX))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .window_control_area(area)
        .active(|button| button.opacity(LIGHT_PRESSED_OPACITY))
        .child(dot)
}

fn mac_buttons(colors: &Colors) -> [Stateful<Div>; 3] {
    [
        light("close-window", WindowControlArea::Close, CLOSE, colors),
        light("minimize", WindowControlArea::Min, MINIMIZE, colors),
        light("maximize", WindowControlArea::Max, ADD, colors),
    ]
}

fn home_side(style: ControlStyle) -> Side {
    match style {
        ControlStyle::Windows => Side::Right,
        ControlStyle::Mac => Side::Left,
    }
}

pub fn controls_width_px(style: ControlStyle) -> f32 {
    match style {
        ControlStyle::Windows => BUTTON_COUNT * BUTTON_WIDTH_PX,
        ControlStyle::Mac => BUTTON_COUNT * LIGHT_SLOT_PX,
    }
}

pub fn controls(controls: Controls, flow: Flow, colors: &Colors, window: &Window) -> Div {
    let mut buttons = match controls.style {
        ControlStyle::Windows => windows_buttons(colors, window),
        ControlStyle::Mac => mac_buttons(colors),
    };
    if controls.side != home_side(controls.style) {
        buttons.reverse();
    }
    let strip = div()
        .group(CONTROLS_GROUP)
        .flex()
        .flex_none()
        .children(buttons);
    match flow {
        Flow::Row => strip.flex_row(),
        Flow::Column => strip.flex_col(),
    }
}

pub fn drag() -> Div {
    div()
        .flex_1()
        .size_full()
        .window_control_area(WindowControlArea::Drag)
}

fn gap() -> AnyElement {
    div()
        .w(px(GAP_PX))
        .h_full()
        .flex_none()
        .window_control_area(WindowControlArea::Drag)
        .into_any_element()
}

fn spaced(items: impl Iterator<Item = AnyElement>) -> impl Iterator<Item = AnyElement> {
    items
        .enumerate()
        .flat_map(|(index, item)| (index > 0).then(gap).into_iter().chain([item]))
}

pub fn bar(placement: Side, controls: Div, content: Vec<AnyElement>) -> Div {
    let controls = Some(controls.into_any_element());
    let (left, right) = match placement {
        Side::Left => (controls, None),
        Side::Right => (None, controls),
    };
    let items = left.into_iter().chain(content).chain(right);
    div().flex().items_center().children(spaced(items))
}
