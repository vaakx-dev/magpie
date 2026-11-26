use gpui::{Context, Entity, IntoElement, WeakEntity, Window, div, prelude::*, px, rgba};
use zeroize::Zeroizing;

use super::{
    keyring::{Entry, Phase},
    look::Emphasis,
    unlock::AfterUnlock,
};
use crate::{
    locale::Text,
    ui::{browser::Browser, icon::Icon, surface::PANEL_CORNER_PX, window},
    webview::{WebView, autofill::Submission},
};
use magpie_engine::{
    bitwarden::{CipherId, Login},
    blocker::site,
};

#[derive(Clone)]
enum Change {
    New,
    Password(CipherId),
    Unknown,
}

pub struct Offer {
    view: WeakEntity<WebView>,
    submission: Submission,
    change: Change,
}

const WIDTH_PX: f32 = 310.0;

fn same_password(login: &Login, submission: &Submission) -> bool {
    login.password.as_deref() == Some(submission.password.as_str())
}

fn change_for(logins: &[&Login], submission: &Submission) -> Option<Change> {
    let unnamed = submission.username.is_empty();
    let owner = |login: &&&Login| {
        let username = login.username.as_deref().unwrap_or_default();
        username == submission.username || (unnamed && logins.len() == 1)
    };
    match logins.iter().find(owner) {
        Some(login) if same_password(login, submission) => None,
        Some(login) => Some(Change::Password(login.id.clone())),
        None if unnamed && logins.iter().any(|login| same_password(login, submission)) => None,
        None => Some(Change::New),
    }
}

impl Offer {
    pub fn on(&self, view: &Entity<WebView>) -> bool {
        self.view == view.downgrade()
    }

    fn repeats(&self, view: &Entity<WebView>, submission: &Submission) -> bool {
        self.on(view)
            && self.submission.page.origin() == submission.page.origin()
            && self.submission.username == submission.username
            && self.submission.password == submission.password
    }

    fn entry(&self) -> Entry {
        let page = &self.submission.page;
        Entry {
            name: site(page).unwrap_or_else(|| page.host_str().unwrap_or_default().to_owned()),
            username: self.submission.username.clone(),
            password: Zeroizing::new(self.submission.password.to_string()),
            uri: page.origin().ascii_serialization(),
        }
    }

    fn title(&self) -> Text {
        match self.change {
            Change::Password(_) => Text::UpdatePassword,
            Change::New | Change::Unknown => Text::SaveLogin,
        }
    }

    fn action(&self) -> Text {
        match self.change {
            Change::Password(_) => Text::Update,
            Change::New | Change::Unknown => Text::Save,
        }
    }
}

impl Browser {
    pub(in crate::ui) fn offer_to_save(
        &mut self,
        view: &Entity<WebView>,
        submission: &Submission,
        cx: &mut Context<Self>,
    ) {
        if self
            .offer
            .as_ref()
            .is_some_and(|offer| offer.repeats(view, submission))
        {
            return;
        }
        let keyring = window::keyring(cx);
        let change = match keyring.read(cx).phase() {
            Phase::Unlocked(vault) => {
                change_for(&vault.matches_for_url(submission.page.as_str()), submission)
            }
            Phase::Locked(_) => Some(Change::Unknown),
            Phase::Busy | Phase::Unavailable(_) | Phase::SignedOut | Phase::Challenged(_) => None,
        };
        let Some(change) = change else {
            return;
        };
        self.offer = Some(Offer {
            view: view.downgrade(),
            submission: submission.clone(),
            change,
        });
        cx.notify();
    }

    pub(in crate::ui) fn save(&mut self, offer: Offer, cx: &mut Context<Self>) {
        let keyring = window::keyring(cx);
        let change = match (&offer.change, keyring.read(cx).phase()) {
            (Change::Unknown, Phase::Unlocked(vault)) => change_for(
                &vault.matches_for_url(offer.submission.page.as_str()),
                &offer.submission,
            ),
            (change, _) => Some(change.clone()),
        };
        match change {
            Some(Change::New) => {
                keyring.update(cx, |keyring, cx| keyring.create_login(offer.entry(), cx))
            }
            Some(Change::Password(id)) => {
                let password = offer.submission.password.clone();
                keyring.update(cx, |keyring, cx| keyring.change_password(id, password, cx))
            }
            Some(Change::Unknown) | None => {}
        }
    }

    fn accept_offer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(offer) = self.offer.take() else {
            return;
        };
        let locked = matches!(window::keyring(cx).read(cx).phase(), Phase::Locked(_));
        match locked {
            true => self.unlock_then(AfterUnlock::Save(offer), window, cx),
            false => self.save(offer, cx),
        }
        cx.notify();
    }

    pub(in crate::ui) fn dismiss_offer(&mut self, cx: &mut Context<Self>) {
        self.offer = None;
        cx.notify();
    }

    pub(in crate::ui) fn save_toast(
        &self,
        view: &Entity<WebView>,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        let offer = self.offer.as_ref().filter(|offer| offer.on(view))?;
        let look = self.look();
        let colors = look.colors;
        let place = site(&offer.submission.page).unwrap_or_default();
        let detail = match offer.submission.username.is_empty() {
            true => place,
            false => format!("{} · {place}", offer.submission.username),
        };
        let heading = div()
            .flex()
            .items_center()
            .gap_3()
            .child(look.avatar(Icon::KeyRound.tinted(rgba(colors.text))))
            .child(look.titled(look.text(offer.title()), detail))
            .child(
                look.icon_button("offer-close", Icon::X.tinted(rgba(colors.muted)))
                    .on_click(cx.listener(|browser, _, _, cx| browser.dismiss_offer(cx))),
            );
        let actions = div()
            .flex()
            .justify_end()
            .gap_2()
            .child(
                look.button(
                    "offer-not-now",
                    look.text(Text::NotNow),
                    Emphasis::Secondary,
                )
                .on_click(cx.listener(|browser, _, _, cx| browser.dismiss_offer(cx))),
            )
            .child(
                look.button("offer-save", look.text(offer.action()), Emphasis::Primary)
                    .on_click(
                        cx.listener(|browser, _, window, cx| browser.accept_offer(window, cx)),
                    ),
            );
        let card = look
            .float()
            .w(px(WIDTH_PX))
            .p_3p5()
            .gap_3()
            .child(heading)
            .child(actions);
        Some(self.cutouts.see_through(PANEL_CORNER_PX).child(card))
    }
}
