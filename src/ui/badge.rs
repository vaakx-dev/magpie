use gpui::{AnyElement, Div, FontWeight, div, prelude::*, px, rgba};

use super::{browser::Browser, icon::Icon, surface::Colors};
use crate::{internal::BRAND, locale::Text};

const BADGE_HEIGHT_PX: f32 = 22.0;
const BADGE_ICON_PX: f32 = 13.0;

pub(super) fn pill(colors: &Colors) -> Div {
    div()
        .h(px(BADGE_HEIGHT_PX))
        .px_2()
        .flex()
        .flex_none()
        .items_center()
        .gap_1()
        .rounded_full()
        .bg(rgba(colors.raised))
        .border_1()
        .border_color(rgba(colors.line))
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgba(colors.text))
}

fn private_badge(label: &'static str, colors: &Colors) -> Div {
    pill(colors)
        .child(
            Icon::EyeOff
                .tinted(rgba(colors.muted))
                .size(px(BADGE_ICON_PX)),
        )
        .child(label)
}

pub fn brand_badge(colors: &Colors) -> Div {
    pill(colors).child(BRAND)
}

impl Browser {
    pub(super) fn private_badge(&self) -> Option<AnyElement> {
        let private = self.store.role().private();
        private.then(|| private_badge(self.text(Text::Private), self.colors()).into_any_element())
    }
}
