use gpui::{Context, Window, div, prelude::*, px, rgba};
use url::Url;

use super::{
    Back,
    browser::{Browser, Reveal},
    frame::{self, Flow},
    icon::Icon,
    surface::surface,
    toolbar::icon_button,
};

const HOT_ZONE_WIDTH_PX: f32 = 70.0;
const HOT_ZONE_HEIGHT_PX: f32 = 54.0;
const INSET_PX: f32 = 16.0;
const HOST_MAX_WIDTH_PX: f32 = 220.0;

impl Browser {
    fn pill(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let host = self
            .active_view()
            .and_then(|view| {
                view.read(cx)
                    .url()
                    .and_then(Url::host_str)
                    .map(str::to_owned)
            })
            .unwrap_or_default();
        let colors = self.colors();
        let content = [
            self.sidebar_button(cx).into_any_element(),
            icon_button("pill-back", Icon::ArrowLeft, colors)
                .on_click(cx.listener(|browser, _, window, cx| browser.back(&Back, window, cx)))
                .into_any_element(),
            div()
                .max_w(px(HOST_MAX_WIDTH_PX))
                .px_1()
                .truncate()
                .text_xs()
                .text_color(rgba(colors.muted))
                .child(host)
                .into_any_element(),
        ]
        .into_iter()
        .chain(self.zoom_badge(cx).map(IntoElement::into_any_element))
        .chain(self.downloads_button(cx).map(IntoElement::into_any_element))
        .chain([self.menu_button(cx).into_any_element()])
        .collect();
        let placement = self.layout().controls;
        let controls = frame::controls(placement, Flow::Row, colors, window);
        surface(colors)
            .p_1()
            .rounded_xl()
            .child(frame::bar(placement.side, controls, content))
    }

    pub(super) fn pill_corner(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shown = self.reveal == Some(Reveal::Pill);
        self.on_sidebar_side(div().id("pill-corner").absolute().top_0(), 0.0)
            .on_hover(cx.listener(|browser, hovered: &bool, _, cx| {
                browser.reveal(Reveal::Pill, *hovered, cx)
            }))
            .when(!shown, |corner| {
                corner.w(px(HOT_ZONE_WIDTH_PX)).h(px(HOT_ZONE_HEIGHT_PX))
            })
            .when(shown, |corner| {
                corner
                    .p(px(INSET_PX))
                    .occlude()
                    .child(self.pill(window, cx))
            })
    }
}
