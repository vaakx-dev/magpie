use gpui::{Context, Window, prelude::*, px};

use super::{
    browser::Browser,
    frame::{self, Flow},
};

pub const TOP_BAR_HEIGHT_PX: f32 = 36.0;
const ADDRESS_WIDTH_PX: f32 = 560.0;
const ADDRESS_MAX_WIDTH_PX: f32 = 760.0;

impl Browser {
    pub(super) fn top_bar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let address = self
            .address_bar(cx)
            .flex_grow()
            .flex_shrink()
            .flex_basis(px(ADDRESS_WIDTH_PX))
            .max_w(px(ADDRESS_MAX_WIDTH_PX))
            .min_w_0();
        let leading = [
            self.navigation(cx).into_any_element(),
            frame::drag().into_any_element(),
            address.into_any_element(),
            frame::drag().into_any_element(),
        ];
        let trailing = [
            self.sidebar_button(cx).into_any_element(),
            self.menu_button(cx).into_any_element(),
        ];
        let content = leading
            .into_iter()
            .chain(self.private_badge())
            .chain(self.downloads_button(cx).map(IntoElement::into_any_element))
            .chain(trailing)
            .collect();
        let placement = self.layout().controls;
        let controls = frame::controls(placement, Flow::Row, self.colors(), window);
        frame::bar(placement.side, controls, content)
            .h(px(TOP_BAR_HEIGHT_PX))
            .flex_none()
            .px_1()
    }
}
