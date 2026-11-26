use gpui::{Context, Entity, WeakEntity, Window};
use url::Url;

use super::{fill, keyring::Phase, offer::Offer};
use crate::{
    locale::Text,
    ui::{browser::Browser, window},
    webview::{WebView, window_handle},
};

pub enum AfterUnlock {
    Fill {
        view: WeakEntity<WebView>,
        page: Url,
    },
    Save(Offer),
}

impl AfterUnlock {
    pub fn fill(view: &Entity<WebView>, page: Url) -> Self {
        Self::Fill {
            view: view.downgrade(),
            page,
        }
    }
}

impl Browser {
    pub(in crate::ui) fn unlock_then(
        &mut self,
        then: AfterUnlock,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.after_unlock = Some(then);
        let keyring = window::keyring(cx);
        if !keyring.read(cx).hello_ready() {
            self.show_passwords(window, cx);
            return;
        }
        let Ok(handle) = window_handle(window) else {
            return;
        };
        let reason = self.text(Text::HelloReason).into();
        keyring.update(cx, |keyring, cx| {
            keyring.unlock_with_hello(handle, reason, cx)
        });
    }

    fn fill_only_match(&self, view: &WeakEntity<WebView>, page: &Url, cx: &mut Context<Self>) {
        let Some(view) = view.upgrade() else {
            return;
        };
        if view.read(cx).url() != Some(page) {
            return;
        }
        let keyring = window::keyring(cx);
        let only = match keyring.read(cx).phase() {
            Phase::Unlocked(vault) => match vault.matches_for_url(page.as_str())[..] {
                [login] => Some((*login).clone()),
                _ => None,
            },
            _ => None,
        };
        if let Some(login) = only {
            fill::sign_in(&view, &login, cx);
        }
    }

    pub(in crate::ui) fn continue_after_unlock(&mut self, cx: &mut Context<Self>) {
        let keyring = window::keyring(cx);
        let unlocked = matches!(keyring.read(cx).phase(), Phase::Unlocked(_));
        if !unlocked {
            return;
        }
        match self.after_unlock.take() {
            Some(AfterUnlock::Fill { view, page }) => self.fill_only_match(&view, &page, cx),
            Some(AfterUnlock::Save(offer)) => self.save(offer, cx),
            None => {}
        }
    }
}
