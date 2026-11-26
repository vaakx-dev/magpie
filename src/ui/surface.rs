use gpui::{Div, ElementId, MouseButton, Rgba, Stateful, div, prelude::*, px, rgba};

use crate::config::Scheme;

const PANEL_WIDTH_PX: f32 = 264.0;
pub const PANEL_PADDING_PX: f32 = 4.0;
pub const PANEL_CORNER_PX: f32 = 12.0;
pub const CONTROL_CORNER_PX: f32 = 8.0;
pub const PAGE_CORNER_PX: f32 = 8.0;
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
    pub surface_hover: u32,
    pub raised: u32,
    pub raised_hover: u32,
    pub scrim: u32,
    pub switch_off: u32,
    pub on_text: u32,
}

const LIGHT: Colors = Colors {
    frame: 0xf3f3f3ff,
    text: 0x141414ff,
    muted: 0x666666ff,
    faint: 0x14141459,
    row_active: 0xffffffff,
    row_hover: 0xffffff80,
    overlay: 0x14141414,
    overlay_pressed: 0x1414140d,
    line: 0x1414141a,
    surface: 0xffffffff,
    surface_hover: 0xffffff99,
    raised: 0xeaeaeaff,
    raised_hover: 0x1414141f,
    scrim: 0xf3f3f399,
    switch_off: 0x66666680,
    on_text: 0xffffffff,
};

const DARK: Colors = Colors {
    frame: 0x141414ff,
    text: 0xf2f2f2ff,
    muted: 0x969696ff,
    faint: 0xf2f2f259,
    row_active: 0x2a2a2aff,
    row_hover: 0x2a2a2a80,
    overlay: 0xf2f2f214,
    overlay_pressed: 0xf2f2f20d,
    line: 0xf2f2f21a,
    surface: 0x1f1f1fff,
    surface_hover: 0x2a2a2aff,
    raised: 0x2a2a2aff,
    raised_hover: 0xf2f2f21f,
    scrim: 0x14141499,
    switch_off: 0x96969680,
    on_text: 0x141414ff,
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
    let hover = colors.raised_hover;
    div()
        .id(id)
        .h(px(BUTTON_HEIGHT_PX))
        .px_3()
        .flex()
        .items_center()
        .rounded(px(CONTROL_CORNER_PX))
        .bg(rgba(colors.raised))
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
