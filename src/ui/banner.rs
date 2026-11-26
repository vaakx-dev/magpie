use gpui::{ClickEvent, Context, Div, Stateful, Window, div, prelude::*, px, rgba};

use super::{browser::Browser, surface::tint};
use crate::locale::Text;

const BANNER_HEIGHT_PX: f32 = 40.0;
const BANNER_TINT_ALPHA: u8 = 0x26;

pub(super) type Choose = fn(&mut Browser, &mut Context<Browser>);

impl Browser {
    pub(super) fn banner_button(
        &self,
        id: &'static str,
        text: Text,
        choose: Choose,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let colors = self.colors();
        div()
            .id(id)
            .px_3()
            .py_1()
            .rounded_md()
            .bg(rgba(colors.field))
            .hover(|button| button.bg(rgba(colors.field_hover)))
            .cursor_pointer()
            .child(self.text(text))
            .on_click(
                cx.listener(move |browser, _: &ClickEvent, _: &mut Window, cx| choose(browser, cx)),
            )
    }

    pub(super) fn banner(
        &self,
        message: impl IntoElement,
        buttons: impl IntoIterator<Item = Stateful<Div>>,
    ) -> Div {
        div()
            .flex_none()
            .h(px(BANNER_HEIGHT_PX))
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .rounded_lg()
            .bg(tint(self.colors().text, BANNER_TINT_ALPHA))
            .child(div().flex_1().child(message))
            .children(buttons)
    }
}
