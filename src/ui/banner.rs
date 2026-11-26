use gpui::{App, ClickEvent, Context, Div, Stateful, Window, prelude::*, px, rgba};

use super::{
    browser::Browser,
    icon::Icon,
    password::{Emphasis, FLOAT_CORNER_PX},
};
use crate::locale::Text;

const TOP_PX: f32 = 12.0;
const GAP_PX: f32 = 8.0;

pub(super) type Choose = fn(&mut Browser, &mut Context<Browser>);

impl Browser {
    pub(super) fn banner_button(
        &self,
        id: &'static str,
        text: Text,
        choose: Choose,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        self.look()
            .button(id, self.text(text), Emphasis::Secondary)
            .on_click(chosen(choose, cx))
    }

    pub(super) fn banner_close(
        &self,
        id: &'static str,
        choose: Choose,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let look = self.look();
        look.icon_button(id, Icon::X.tinted(rgba(look.colors.muted)))
            .on_click(chosen(choose, cx))
    }

    pub(super) fn banner(
        &self,
        message: impl IntoElement,
        buttons: impl IntoIterator<Item = Stateful<Div>>,
    ) -> Div {
        self.look()
            .float()
            .flex_row()
            .items_center()
            .gap_2()
            .py_1p5()
            .pl_4()
            .pr_1p5()
            .child(message)
            .children(buttons)
    }

    pub(super) fn banners(&self, cx: &mut Context<Self>) -> Div {
        self.cutouts
            .see_through(FLOAT_CORNER_PX)
            .absolute()
            .top(px(TOP_PX))
            .left_0()
            .right_0()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(GAP_PX))
            .children(self.recovery_banner(cx))
            .children(self.update_banner(cx))
    }
}

fn chosen(
    choose: Choose,
    cx: &mut Context<Browser>,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    cx.listener(move |browser, _: &ClickEvent, _: &mut Window, cx| choose(browser, cx))
}
