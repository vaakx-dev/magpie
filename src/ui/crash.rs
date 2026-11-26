use gpui::{Context, Entity, FontWeight, SharedString, div, prelude::*, rgba};

use super::{browser::Browser, surface::text_button};
use crate::{locale::Text, webview::WebView};

impl Browser {
    pub(super) fn crash_notice(
        &self,
        view: &Entity<WebView>,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        if !view.read(cx).crashed() {
            return None;
        }
        let colors = self.colors();
        let id = SharedString::from(format!("reload-crashed-{}", view.entity_id()));
        let crashed = view.clone();
        let reload = text_button(id, self.text(Text::ReloadTab), colors)
            .on_click(cx.listener(move |_, _, _, cx| crashed.update(cx, WebView::reload)));
        Some(
            div()
                .absolute()
                .inset_0()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .bg(rgba(colors.surface))
                .text_color(rgba(colors.text))
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(self.text(Text::TabCrashed)),
                )
                .child(reload),
        )
    }
}
