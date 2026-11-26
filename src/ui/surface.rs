use gpui::{Div, ElementId, MouseButton, Rgba, Stateful, div, prelude::*, px, rgba};

use crate::config::Scheme;

const PANEL_WIDTH_PX: f32 = 264.0;
pub const PANEL_PADDING_PX: f32 = 4.0;
pub const PANEL_CORNER_PX: f32 = 10.0;
pub const ROW_HEIGHT_PX: f32 = 28.0;
const BUTTON_HEIGHT_PX: f32 = 28.0;

pub struct Colors {
    pub frame: u32,
    pub text: u32,
    pub muted: u32,
    pub faint: u32,
    pub row_active: u32,
    pub row_hover: u32,
    pub overlay: u32,
    pub overlay_pressed: u32,
    pub line: u32,
    pub surface: u32,
    pub raised: u32,
    pub field: u32,
    pub field_hover: u32,
    pub tile: u32,
    pub tile_hover: u32,
    pub scrim: u32,
    pub page: u32,
    pub switch_off: u32,
    pub on_text: u32,
}

const LIGHT: Colors = Colors {
    frame: 0xebebebff,
    text: 0x1f1f1fff,
    muted: 0x5c5c5cff,
    faint: 0xa3a3a3ff,
    row_active: 0xffffffb3,
    row_hover: 0xffffff66,
    overlay: 0x1f1f1f12,
    overlay_pressed: 0x1f1f1f1f,
    line: 0x1f1f1f1a,
    surface: 0xffffffff,
    raised: 0x1f1f1f0d,
    field: 0xffffff99,
    field_hover: 0xffffffd9,
    tile: 0xffffff73,
    tile_hover: 0xffffffcc,
    scrim: 0x1f1f1f2e,
    page: 0xffffffff,
    switch_off: 0x8f8f8f80,
    on_text: 0xffffffff,
};

const DARK: Colors = Colors {
    frame: 0x1c1c1cff,
    text: 0xecececff,
    muted: 0xa3a3a3ff,
    faint: 0x636363ff,
    row_active: 0xffffff1f,
    row_hover: 0xffffff0f,
    overlay: 0xffffff17,
    overlay_pressed: 0xffffff0d,
    line: 0xffffff17,
    surface: 0x262626ff,
    raised: 0xffffff0f,
    field: 0xffffff0f,
    field_hover: 0xffffff1a,
    tile: 0xffffff0d,
    tile_hover: 0xffffff1f,
    scrim: 0x0000006b,
    page: 0x212121ff,
    switch_off: 0xffffff33,
    on_text: 0x1c1c1cff,
};

pub fn colors(scheme: Scheme) -> &'static Colors {
    match scheme {
        Scheme::Light => &LIGHT,
        Scheme::Dark => &DARK,
    }
}

pub fn tint(color: u32, alpha: u8) -> Rgba {
    rgba((color & !0xff) | u32::from(alpha))
}

pub fn surface(colors: &Colors) -> Div {
    div()
        .bg(rgba(colors.surface))
        .text_color(rgba(colors.text))
        .shadow_2xl()
}

pub fn text_button(
    id: impl Into<ElementId>,
    label: &'static str,
    colors: &Colors,
) -> Stateful<Div> {
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

pub fn panel(colors: &Colors) -> Div {
    surface(colors)
        .w(px(PANEL_WIDTH_PX))
        .p(px(PANEL_PADDING_PX))
        .flex()
        .flex_col()
        .gap_0p5()
        .rounded(px(PANEL_CORNER_PX))
        .occlude()
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
}
