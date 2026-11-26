use std::io;

use gpui::{App, Context};

use super::{browser::Browser, window};
use crate::{
    internal::Page,
    locale::Text,
    storage::{
        profile::{Profile, ProfileId, Profiles},
        session::SpaceId,
    },
};

fn report(result: io::Result<()>) {
    if let Err(error) = result {
        eprintln!("magpie: could not save the profiles: {error}");
    }
}

fn profiles_changed(cx: &mut App) {
    window::each_browser(cx, |browser, cx| {
        browser.refresh_pages(&[Page::Settings], cx);
        cx.notify();
    });
}

impl Browser {
    pub(super) fn named_profiles(&self) -> Vec<Profile> {
        self.store.profiles().named(self.text(Text::DefaultProfile))
    }

    fn edit_profiles(&self, edit: impl FnOnce(&mut Profiles) -> bool, cx: &mut App) -> bool {
        let edited = edit(&mut self.store.profiles_mut());
        if edited {
            report(self.store.save_profiles());
            profiles_changed(cx);
        }
        edited
    }

    pub(super) fn create_profile(&mut self, name: &str, cx: &mut Context<Self>) {
        self.edit_profiles(|profiles| profiles.create(name).is_some(), cx);
    }

    pub(super) fn rename_profile(&mut self, id: &ProfileId, name: &str, cx: &mut Context<Self>) {
        self.edit_profiles(|profiles| profiles.rename(id, name), cx);
    }

    pub(super) fn assign_profile(
        &mut self,
        space: SpaceId,
        profile: &ProfileId,
        cx: &mut Context<Self>,
    ) {
        if !self.store.profiles().contains(profile) {
            return;
        }
        if self.store.session.assign_profile(space, profile) {
            self.rehome(cx);
            self.save_session(cx);
            self.refresh_pages(&[Page::Settings], cx);
        }
    }

    pub(super) fn delete_profile(&mut self, id: &ProfileId, cx: &mut Context<Self>) {
        if !self.edit_profiles(|profiles| profiles.delete(id), cx) {
            return;
        }
        let left = id.clone();
        window::each_browser(cx, move |browser, cx| browser.leave_profile(&left, cx));
        let deleted = self.runtime.delete_profile(id);
        cx.spawn(async move |_, _| {
            if let Err(error) = deleted.await {
                eprintln!("magpie: could not delete a WebView2 profile: {error}");
            }
        })
        .detach();
    }

    fn leave_profile(&mut self, id: &ProfileId, cx: &mut Context<Self>) {
        let profiles = self.store.profiles().clone();
        if self.store.session.keep_profiles(&profiles) {
            self.rehome(cx);
            self.save_session(cx);
        }
        self.runtime.forget_profile(id);
    }
}
