use std::time::Duration;

use gpui::{
    Context, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Task, Window, div,
    prelude::*, px,
};

use super::{browser::Browser, frame::INSET_PX, surface::tint};
use crate::config::{Side, SidebarWidth};

const LINE_PX: f32 = 2.0;
const HIT_PX: f32 = 4.0;
const SEAM_MARGIN_PX: f32 = (INSET_PX - HIT_PX) / 2.0;
const HOVER_ALPHA: u8 = 0x99;
const DRAG_ALPHA: u8 = 0xff;
const HOVER_DWELL: Duration = Duration::from_millis(180);

#[derive(Clone, Copy)]
struct Grip {
    offset_px: f32,
    width: SidebarWidth,
}

#[derive(Default)]
enum Hover {
    #[default]
    Away,
    Dwelling {
        _timer: Task<()>,
    },
    Lit,
}

#[derive(Default)]
pub struct Handle {
    grip: Option<Grip>,
    hover: Hover,
}

impl Handle {
    pub fn leave(&mut self) {
        self.hover = Hover::Away;
    }

    fn line_alpha(&self) -> Option<u8> {
        match (self.grip, &self.hover) {
            (Some(_), _) => Some(DRAG_ALPHA),
            (None, Hover::Lit) => Some(HOVER_ALPHA),
            (None, _) => None,
        }
    }
}

impl Browser {
    pub(super) fn sidebar_width_px(&self) -> f32 {
        self.resize
            .grip
            .map_or(self.layout().sidebar_width, |grip| grip.width)
            .px()
    }

    fn reach_px(&self, x: Pixels, window: &Window) -> f32 {
        let x = f32::from(x);
        match self.layout().sidebar_side {
            Side::Left => x,
            Side::Right => f32::from(window.viewport_size().width) - x,
        }
    }

    fn grab(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let width_px = self.sidebar_width_px();
        self.resize.grip = Some(Grip {
            offset_px: width_px - self.reach_px(event.position.x, window),
            width: SidebarWidth::new(width_px),
        });
        cx.notify();
    }

    fn drag(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(grip) = self.resize.grip {
            let proposed_px = self.reach_px(event.position.x, window) + grip.offset_px;
            let width = SidebarWidth::new(proposed_px);
            self.resize.grip = Some(Grip { width, ..grip });
            cx.notify();
        }
    }

    fn release(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(grip) = self.resize.grip.take() {
            self.change_settings(|settings| settings.layout.sidebar_width = grip.width, cx);
        }
    }

    fn light_after_dwell(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |browser, cx| {
            cx.background_executor().timer(HOVER_DWELL).await;
            browser
                .update(cx, |browser, cx| {
                    browser.resize.hover = Hover::Lit;
                    cx.notify();
                })
                .ok();
        })
    }

    fn approach_handle(&mut self, _: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if matches!(self.resize.hover, Hover::Away) {
            self.resize.hover = Hover::Dwelling {
                _timer: Self::light_after_dwell(cx),
            };
        }
    }

    fn leave_handle(&mut self, hovered: &bool, _: &mut Window, cx: &mut Context<Self>) {
        if !hovered {
            self.resize.leave();
            cx.notify();
        }
    }

    pub(super) fn resize_handle(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let ink = self.colors().text;
        let line = div()
            .w(px(LINE_PX))
            .h_full()
            .rounded_full()
            .when_some(self.resize.line_alpha(), |line, alpha| {
                line.bg(tint(ink, alpha))
            });
        let handle = div()
            .id("sidebar-resize")
            .absolute()
            .top_0()
            .bottom_0()
            .w(px(HIT_PX))
            .when(!self.layout().top_bar.on(), |handle| {
                handle.pt(px(INSET_PX))
            })
            .pb(px(INSET_PX))
            .flex()
            .justify_center()
            .cursor_col_resize()
            .on_mouse_move(cx.listener(Self::approach_handle))
            .on_hover(cx.listener(Self::leave_handle))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::grab))
            .child(line);
        match self.layout().sidebar_side {
            Side::Left => handle.right(px(SEAM_MARGIN_PX)),
            Side::Right => handle.left(px(SEAM_MARGIN_PX)),
        }
    }

    pub(super) fn resize_overlay(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        self.resize.grip.is_some().then(|| {
            div()
                .absolute()
                .inset_0()
                .occlude()
                .cursor_col_resize()
                .on_mouse_move(cx.listener(Self::drag))
                .on_mouse_up(MouseButton::Left, cx.listener(Self::release))
                .on_mouse_up_out(MouseButton::Left, cx.listener(Self::release))
        })
    }
}
