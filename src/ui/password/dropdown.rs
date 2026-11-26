use gpui::{AnyElement, Context, Entity, IntoElement, Window, prelude::*, px, rgba};
use url::Url;

use super::{
    fill,
    keyring::Phase,
    look::{FLOAT_CORNER_PX, MASKED_PASSWORD, login_name},
    unlock::AfterUnlock,
};
use crate::{
    internal::settings::Section,
    locale::Text,
    ui::{browser::Browser, icon::Icon, window},
    webview::{
        WebView,
        autofill::{Field, Role},
    },
};
use magpie_engine::{
    bitwarden::{CipherId, Login, password},
    blocker::site,
};

const MIN_WIDTH_PX: f32 = 280.0;
const GAP_PX: f32 = 6.0;

fn asks_for_password(role: Role) -> bool {
    matches!(role, Role::Password | Role::NewPassword)
}

impl Browser {
    fn fill_from_dropdown(
        &mut self,
        view: &Entity<WebView>,
        id: &CipherId,
        cx: &mut Context<Self>,
    ) {
        let keyring = window::keyring(cx);
        let login = match keyring.read(cx).phase() {
            Phase::Unlocked(vault) => vault.login(id).cloned(),
            _ => None,
        };
        if let Some(login) = login {
            fill::sign_in(view, &login, cx);
        }
    }

    fn generate_into(&mut self, view: &Entity<WebView>, cx: &mut Context<Self>) {
        if let Ok(generated) = password::generate() {
            fill::new_password(view, &generated, cx);
        }
    }

    fn unlock_to_fill(
        &mut self,
        view: &Entity<WebView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(page) = view.read(cx).url().cloned() else {
            return;
        };
        self.unlock_then(AfterUnlock::fill(view, page), window, cx);
    }

    fn login_rows(
        &self,
        view: &Entity<WebView>,
        logins: &[&Login],
        place: &str,
        cx: &Context<Self>,
    ) -> Vec<AnyElement> {
        let look = self.look();
        logins
            .iter()
            .enumerate()
            .map(|(index, login)| {
                let id = login.id.clone();
                let view = view.clone();
                look.row(("autofill-login", index))
                    .child(look.initial(login_name(login)))
                    .child(look.titled(
                        login_name(login).to_owned(),
                        format!("{MASKED_PASSWORD} · {place}"),
                    ))
                    .on_click(cx.listener(move |browser, _, _, cx| {
                        browser.fill_from_dropdown(&view, &id, cx)
                    }))
                    .into_any_element()
            })
            .collect()
    }

    fn unlock_row(&self, view: &Entity<WebView>, cx: &Context<Self>) -> AnyElement {
        let look = self.look();
        let view = view.clone();
        look.row("autofill-unlock")
            .child(look.avatar(Icon::Lock.tinted(rgba(look.colors.text))))
            .child(look.titled(
                look.text(Text::UnlockToFill),
                look.text(Text::MasterPassword),
            ))
            .on_click(
                cx.listener(move |browser, _, window, cx| {
                    browser.unlock_to_fill(&view, window, cx)
                }),
            )
            .into_any_element()
    }

    fn generate_row(&self, view: &Entity<WebView>, cx: &Context<Self>) -> AnyElement {
        let look = self.look();
        let view = view.clone();
        look.row("autofill-generate")
            .child(look.avatar(Icon::Sparkle.tinted(rgba(look.colors.text))))
            .child(look.titled(
                look.text(Text::GenerateStrongPassword),
                look.text(Text::SavedAfterSignIn),
            ))
            .on_click(cx.listener(move |browser, _, _, cx| browser.generate_into(&view, cx)))
            .into_any_element()
    }

    fn manage_row(&self, cx: &Context<Self>) -> AnyElement {
        let look = self.look();
        look.row("autofill-manage")
            .justify_between()
            .text_xs()
            .text_color(rgba(look.colors.muted))
            .child(look.text(Text::ManagePasswords))
            .child(Icon::ChevronRight.tinted(rgba(look.colors.muted)))
            .on_click(
                cx.listener(|browser, _, _, cx| browser.show_settings(Section::Passwords, cx)),
            )
            .into_any_element()
    }

    fn dropdown_rows(
        &self,
        view: &Entity<WebView>,
        field: Field,
        page: &Url,
        cx: &Context<Self>,
    ) -> Vec<AnyElement> {
        let keyring = window::keyring(cx);
        let generate = asks_for_password(field.role).then(|| self.generate_row(view, cx));
        let place = site(page).unwrap_or_default();
        let (saved, signed_in) = match keyring.read(cx).phase() {
            Phase::Unlocked(vault) => {
                let logins = vault.matches_for_url(page.as_str());
                (self.login_rows(view, &logins, &place, cx), true)
            }
            Phase::Locked(_) => (vec![self.unlock_row(view, cx)], true),
            Phase::Busy | Phase::Unavailable(_) | Phase::SignedOut | Phase::Challenged(_) => {
                (Vec::new(), false)
            }
        };
        if saved.is_empty() && generate.is_none() {
            return Vec::new();
        }
        let generated: Vec<AnyElement> = generate.into_iter().collect();
        let ordered = match field.role {
            Role::NewPassword => [generated, saved],
            Role::Username | Role::Password => [saved, generated],
        };
        let manage = signed_in.then(|| vec![self.manage_row(cx)]);
        let look = self.look();
        ordered
            .into_iter()
            .chain(manage)
            .filter(|section| !section.is_empty())
            .enumerate()
            .flat_map(|(index, section)| {
                let separator = (index > 0).then(|| look.separator().into_any_element());
                separator.into_iter().chain(section)
            })
            .collect()
    }

    pub(in crate::ui) fn login_dropdown(
        &self,
        view: &Entity<WebView>,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        if self.popover.is_some() {
            return None;
        }
        let (field, page) = {
            let page = view.read(cx);
            (page.login_field()?, page.url()?.clone())
        };
        let rows = self.dropdown_rows(view, field, &page, cx);
        if rows.is_empty() {
            return None;
        }
        let width = field.bounds.size.width.max(px(MIN_WIDTH_PX));
        let top = field.bounds.bottom() + px(GAP_PX);
        Some(
            self.cutouts
                .see_through(FLOAT_CORNER_PX)
                .absolute()
                .left(field.bounds.left())
                .top(top)
                .child(self.look().float().w(width).children(rows)),
        )
    }
}
