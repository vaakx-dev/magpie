use gpui::{ClickEvent, Context, Div, ElementId, Stateful, Window, div, prelude::*, px, rgba};

use super::browser::Browser;
use crate::config::Switch;

const WIDTH_PX: f32 = 36.0;
const HEIGHT_PX: f32 = 20.0;
const KNOB_PX: f32 = 16.0;
const KNOB_OFF: u32 = 0xffffffff;

impl Browser {
    pub(super) fn toggle(
        &self,
        id: impl Into<ElementId>,
        state: Switch,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        let colors = self.colors();
        let (track, knob) = match state {
            Switch::On => (colors.text, colors.on_text),
            Switch::Off => (colors.switch_off, KNOB_OFF),
        };
        div()
            .id(id.into())
            .w(px(WIDTH_PX))
            .h(px(HEIGHT_PX))
            .flex_none()
            .p(px((HEIGHT_PX - KNOB_PX) / 2.0))
            .flex()
            .items_center()
            .when(state.on(), |track| track.justify_end())
            .rounded_full()
            .bg(rgba(track))
            .cursor_pointer()
            .on_click(cx.listener(move |browser, _: &ClickEvent, window, cx| {
                cx.stop_propagation();
                action(browser, window, cx)
            }))
            .child(
                div()
                    .size(px(KNOB_PX))
                    .rounded_full()
                    .bg(rgba(knob))
                    .shadow_sm(),
            )
    }
}
