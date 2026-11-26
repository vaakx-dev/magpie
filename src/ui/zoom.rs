use gpui::{ClickEvent, Context, Div, Stateful, Window, div, prelude::*, px, rgba};

use super::{ResetZoom, ZoomIn, ZoomOut, browser::Browser};
use crate::zoom::Step;

const BADGE_HEIGHT_PX: f32 = 20.0;
const HOVER_OPACITY: f32 = 0.8;

impl Browser {
    pub(super) fn step_zoom(&mut self, step: Step, cx: &mut Context<Self>) {
        self.with_view(cx, |view, cx| view.step_zoom(step, cx));
    }

    pub(super) fn zoom_in(&mut self, _: &ZoomIn, _: &mut Window, cx: &mut Context<Self>) {
        self.step_zoom(Step::In, cx);
    }

    pub(super) fn zoom_out(&mut self, _: &ZoomOut, _: &mut Window, cx: &mut Context<Self>) {
        self.step_zoom(Step::Out, cx);
    }

    pub(super) fn reset_zoom(&mut self, _: &ResetZoom, _: &mut Window, cx: &mut Context<Self>) {
        self.step_zoom(Step::Reset, cx);
    }

    pub(super) fn apply_zoom_everywhere(&self, cx: &mut Context<Self>) {
        self.live
            .values()
            .flatten()
            .for_each(|pane| pane.view.read(cx).apply_zoom());
    }

    pub(super) fn zoom_badge(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let view = self.active_view()?.read(cx);
        if !view.zoomed() {
            return None;
        }
        let zoom = view.zoom();
        let colors = self.colors();
        Some(
            div()
                .id("zoom-badge")
                .flex_none()
                .px_2()
                .h(px(BADGE_HEIGHT_PX))
                .flex()
                .items_center()
                .rounded_full()
                .text_xs()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(rgba(colors.text))
                .bg(rgba(colors.overlay))
                .hover(|badge| badge.opacity(HOVER_OPACITY))
                .cursor_pointer()
                .on_click(cx.listener(|browser, _: &ClickEvent, window, cx| {
                    cx.stop_propagation();
                    browser.reset_zoom(&ResetZoom, window, cx);
                }))
                .child(zoom.to_string()),
        )
    }
}
