use gpui::{App, Context, Div, ElementId, MouseButton, Stateful, Svg, div, prelude::*, px, rgba};

use super::{
    browser::Browser,
    surface::{CONTROL_CORNER_PX, Colors},
};
use crate::{storage::session::TabId, webview::sleep::IDLE_BEFORE_SLEEP};

const ASLEEP_OPACITY: f32 = 0.55;
const ROW_BUTTON_PX: f32 = 20.0;

pub fn row_button(id: impl Into<ElementId>, icon: Svg, colors: &Colors) -> Stateful<Div> {
    let overlay = colors.overlay;
    div()
        .id(id)
        .size(px(ROW_BUTTON_PX))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded(px(CONTROL_CORNER_PX))
        .hover(move |button| button.bg(rgba(overlay)))
        .child(icon)
}

impl Browser {
    pub(super) fn rest(&mut self, cx: &mut Context<Self>) {
        for pane in self.live.values().flatten() {
            pane.view
                .update(cx, |view, cx| view.rest(IDLE_BEFORE_SLEEP, cx));
        }
    }

    pub(super) fn asleep(&self, id: TabId, cx: &App) -> bool {
        self.live
            .get(&id)
            .is_none_or(|panes| panes.iter().all(|pane| pane.view.read(cx).asleep()))
    }

    pub(super) fn tab_slot(
        &self,
        slot: Stateful<Div>,
        id: TabId,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        slot.when(self.asleep(id, cx), |slot| slot.opacity(ASLEEP_OPACITY))
            .on_mouse_up(
                MouseButton::Middle,
                cx.listener(move |browser, _, _, cx| browser.close(id, cx)),
            )
    }
}
