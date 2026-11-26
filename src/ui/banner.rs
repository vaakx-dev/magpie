use gpui::{App, ClickEvent, Context, Div, Stateful, Window, prelude::*, rgba};

use super::{
    browser::Browser, icon::Icon, overlay::stack, password::Emphasis, surface::PANEL_CORNER_PX,
};
use crate::locale::Text;

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

    pub(super) fn banners(&self, cx: &mut Context<Self>) -> Option<Div> {
        let banners: Vec<Div> = [self.recovery_banner(cx), self.update_banner(cx)]
            .into_iter()
            .flatten()
            .collect();
        (!banners.is_empty()).then(|| {
            stack(self.cutouts.see_through(PANEL_CORNER_PX))
                .items_center()
                .children(banners)
        })
    }
}

fn chosen(
    choose: Choose,
    cx: &mut Context<Browser>,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    cx.listener(move |browser, _: &ClickEvent, _: &mut Window, cx| choose(browser, cx))
}
