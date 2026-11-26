use gpui::{
    Div, ElementId, FontWeight, IntoElement, MouseButton, SharedString, Stateful, div, prelude::*,
    px, rgba,
};

use super::keyring::Problem;
use crate::{
    locale::{Locale, Text},
    ui::surface::{CONTROL_CORNER_PX, Colors, PANEL_CORNER_PX, surface, tint},
};
use magpie_engine::bitwarden::Login;

#[derive(Clone, Copy)]
pub struct Look {
    pub colors: &'static Colors,
    pub locale: Locale,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    Primary,
    Secondary,
}

pub const MASKED_PASSWORD: &str = "••••••••";
const BUTTON_HEIGHT_PX: f32 = 30.0;
const ICON_BUTTON_PX: f32 = 30.0;
const AVATAR_PX: f32 = 32.0;
const ROW_INSET_PX: f32 = 4.0;
const PRIMARY_HOVER_OPACITY: f32 = 0.9;
const AVATAR_ALPHA: u8 = 0x12;

pub fn login_name(login: &Login) -> &str {
    login
        .username
        .as_deref()
        .filter(|username| !username.is_empty())
        .unwrap_or(&login.name)
}

impl Look {
    pub fn text(&self, text: Text) -> &'static str {
        self.locale.text(text)
    }

    pub fn fill(&self, text: Text, value: &str) -> String {
        self.locale.fill(text, value)
    }

    pub fn float(&self) -> Div {
        surface(self.colors)
            .rounded(px(PANEL_CORNER_PX))
            .py_1()
            .flex()
            .flex_col()
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
    }

    pub fn button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        emphasis: Emphasis,
    ) -> Stateful<Div> {
        let colors = self.colors;
        let hover = colors.raised_hover;
        let button = div()
            .id(id)
            .h(px(BUTTON_HEIGHT_PX))
            .px_3()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap_1p5()
            .rounded(px(CONTROL_CORNER_PX))
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .cursor_pointer()
            .child(label.into());
        match emphasis {
            Emphasis::Primary => button
                .bg(rgba(colors.text))
                .text_color(rgba(colors.on_text))
                .hover(|button| button.opacity(PRIMARY_HOVER_OPACITY)),
            Emphasis::Secondary => button
                .bg(rgba(colors.raised))
                .hover(move |button| button.bg(rgba(hover))),
        }
    }

    pub fn icon_button(&self, id: impl Into<ElementId>, icon: impl IntoElement) -> Stateful<Div> {
        let hover = self.colors.overlay;
        div()
            .id(id)
            .size(px(ICON_BUTTON_PX))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(px(CONTROL_CORNER_PX))
            .cursor_pointer()
            .hover(move |button| button.bg(rgba(hover)))
            .child(icon)
    }

    pub fn row(&self, id: impl Into<ElementId>) -> Stateful<Div> {
        let hover = self.colors.overlay;
        div()
            .id(id)
            .mx(px(ROW_INSET_PX))
            .px_2p5()
            .py_2()
            .flex()
            .items_center()
            .gap_3()
            .rounded(px(CONTROL_CORNER_PX))
            .cursor_pointer()
            .hover(move |row| row.bg(rgba(hover)))
    }

    pub fn avatar(&self, content: impl IntoElement) -> Div {
        div()
            .size(px(AVATAR_PX))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(px(CONTROL_CORNER_PX))
            .bg(tint(self.colors.text, AVATAR_ALPHA))
            .font_weight(FontWeight::BOLD)
            .child(content)
    }

    pub fn initial(&self, text: &str) -> Div {
        let initial = text
            .chars()
            .find(|character| character.is_alphanumeric())
            .map(|character| character.to_uppercase().to_string())
            .unwrap_or_default();
        self.avatar(initial)
    }

    pub fn titled(&self, title: impl Into<SharedString>, detail: impl Into<SharedString>) -> Div {
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .truncate()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title.into()),
            )
            .child(
                div()
                    .truncate()
                    .text_xs()
                    .text_color(rgba(self.colors.muted))
                    .child(detail.into()),
            )
    }

    pub fn separator(&self) -> Div {
        div().h(px(1.0)).mx_1p5().my_1().bg(rgba(self.colors.line))
    }

    pub fn note(&self, text: impl Into<SharedString>) -> Div {
        div()
            .px_4()
            .py_2()
            .text_xs()
            .text_color(rgba(self.colors.muted))
            .child(text.into())
    }

    pub fn problem(&self, problem: Option<&Problem>) -> Option<Div> {
        let message = match problem? {
            Problem::Failed(message) => SharedString::from(message.clone()),
            Problem::WrongCode => self.text(Text::WrongCode).into(),
        };
        Some(
            div()
                .mx_2()
                .px_2()
                .py_1()
                .rounded(px(CONTROL_CORNER_PX))
                .text_xs()
                .bg(rgba(self.colors.raised))
                .child(message),
        )
    }
}
