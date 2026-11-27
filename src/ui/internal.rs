use std::path::Path;

use gpui::{App, AsyncApp, Context, Entity, PathPromptOptions, WeakEntity, prelude::*};
use url::Url;

use super::{
    browser::Browser,
    command::Target,
    downloads::{Control, Downloads, open_file, show_in_folder},
    password::passwords_state,
    update::{Updater, updater},
    window,
};
use crate::{
    address::destination,
    import::{Imported, detect},
    internal::{
        Message, Page,
        settings::{self, Import, Reply, Section, Snapshot},
    },
    locale::Text,
    pages::{self, LocalTime, PageAction, bookmarks_payload, history_payload, newtab_payload},
    storage::{
        clock::Epoch,
        session::{Space, TabId},
    },
    webview::{WebView, default_downloads},
};

const MISSING_PROFILE: &str = "That browser profile is no longer there";

fn shows(page: Page, address: &str) -> bool {
    Url::parse(address).ok().as_ref().and_then(Page::of) == Some(page)
}

fn read(import: &Import) -> (String, Imported) {
    let profile = detect()
        .into_iter()
        .find(|profile| profile.folder == import.profile);
    match (profile, import.scope()) {
        (Some(profile), Some(scope)) => (profile.label(), profile.read(scope)),
        _ => {
            let missing = Imported {
                warnings: vec![MISSING_PROFILE.to_owned()],
                ..Imported::default()
            };
            (String::new(), missing)
        }
    }
}

pub(super) fn post_to_settings(view: &WebView, json: &str) {
    if view.url().and_then(Page::of) == Some(Page::Settings) {
        view.post(json);
    }
}

pub(super) fn reply(view: &WeakEntity<WebView>, reply: &Reply, cx: &mut AsyncApp) {
    let json = reply.json();
    view.update(cx, |view, _| post_to_settings(view, &json))
        .ok();
}

pub(super) fn history_changed(cx: &mut App) {
    window::each_browser(cx, |browser, cx| {
        browser.refresh_pages(&[Page::History], cx)
    });
}

pub(super) fn updates_changed(cx: &mut App) {
    window::each_browser(cx, |browser, cx| {
        browser.refresh_pages(&[Page::Settings], cx);
        cx.notify();
    });
}

pub(super) fn passwords_changed(cx: &mut App) {
    window::each_browser(cx, |browser, cx| {
        browser.refresh_pages(&[Page::Settings], cx);
        browser.continue_after_unlock(cx);
        cx.notify();
    });
}

impl Browser {
    fn tab_showing(&self, page: Page) -> Option<TabId> {
        self.store
            .session
            .spaces
            .iter()
            .flat_map(Space::tabs)
            .find(|tab| shows(page, tab.url()))
            .map(|tab| tab.id)
    }

    pub(super) fn show_page(&mut self, page: Page, cx: &mut Context<Self>) {
        match self.tab_showing(page) {
            Some(id) => self.activate(id, cx),
            None => self.open_in_new_tab(page.url(), cx),
        }
    }

    pub(super) fn show_settings(&mut self, section: Section, cx: &mut Context<Self>) {
        match self.tab_showing(Page::Settings) {
            Some(id) => {
                self.activate(id, cx);
                self.open(section.url(), Target::Current, cx);
            }
            None => self.open_in_new_tab(section.url(), cx),
        }
    }

    fn settings_state(&self, cx: &App) -> Reply {
        Reply::state(Snapshot {
            settings: self.settings(),
            scheme: self.scheme(),
            paused_sites: self.runtime.shield().paused_sites(),
            zooms: self.runtime.site_zooms(),
            permissions: self.runtime.permissions().sites(),
            default_downloads: default_downloads(),
            data_folder: self.store.folder(),
            passwords: passwords_state(window::keyring(cx).read(cx), self.locale()),
            profiles: &self.store.profiles(),
            spaces: self.assignable_spaces(),
            update: updater(cx).read(cx).phase().clone(),
        })
    }

    fn assignable_spaces(&self) -> &[Space] {
        match self.store.role().private() {
            true => &[],
            false => &self.store.session.spaces,
        }
    }

    fn page_state(&self, page: Page, cx: &App) -> String {
        let scheme = self.scheme();
        match page {
            Page::Settings => self.settings_state(cx).json(),
            Page::History => {
                let history = history_payload(&self.store.history(), LocalTime::current());
                pages::update(scheme, history)
            }
            Page::Bookmarks => pages::update(scheme, bookmarks_payload(&self.store.session)),
            Page::Downloads => pages::update(scheme, self.downloads.read(cx).payload()),
            Page::NewTab => pages::update(scheme, newtab_payload(self.current())),
        }
    }

    fn views_showing(&self, page: Page, cx: &App) -> Vec<Entity<WebView>> {
        self.live
            .values()
            .flatten()
            .map(|pane| &pane.view)
            .filter(|view| view.read(cx).url().and_then(Page::of) == Some(page))
            .cloned()
            .collect()
    }

    fn refresh_page(&self, page: Page, cx: &App) {
        let views = self.views_showing(page, cx);
        if views.is_empty() {
            return;
        }
        let json = self.page_state(page, cx);
        for view in views {
            view.read(cx).post(&json);
        }
    }

    pub(super) fn refresh_pages(&self, pages: &[Page], cx: &App) {
        for page in pages {
            self.refresh_page(*page, cx);
        }
    }

    pub(super) fn answer_page(
        &mut self,
        view: &Entity<WebView>,
        message: Message,
        cx: &mut Context<Self>,
    ) {
        match message {
            Message::Settings(message) => self.answer_settings(view.downgrade(), message, cx),
            Message::Action(page, action) => self.act(view, page, action, cx),
        }
    }

    fn act(
        &mut self,
        view: &Entity<WebView>,
        page: Page,
        action: PageAction,
        cx: &mut Context<Self>,
    ) {
        match action {
            PageAction::Ready => view.read(cx).post(&self.page_state(page, cx)),
            PageAction::OpenUrl { url } => self.go(view, &url, cx),
            PageAction::Navigate { text } => self.go(view, &text, cx),
            PageAction::FocusTab { tab } => self.activate(tab, cx),
            PageAction::DeleteHistory { url } => self.forget_visit(&url, cx),
            PageAction::ClearHistory => self.clear_history(cx),
            PageAction::RemoveFavorite { space, url } => self.remove_favorite_in(space, &url, cx),
            PageAction::RemovePinned { tab } => self.remove_pinned(tab, cx),
            PageAction::EditFavorite { space, url, edited } => {
                self.edit_session(|session| session.edit_favorite(space, &url, edited), cx)
            }
            PageAction::RenameTab { tab, title } => {
                self.edit_session(|session| session.rename_tab(tab, &title), cx)
            }
            PageAction::RenameFolder { folder, name } => {
                self.edit_session(|session| session.rename_folder(folder, &name), cx)
            }
            PageAction::MoveFavorite { space, url, slot } => {
                self.edit_session(|session| session.move_favorite(space, &url, slot), cx)
            }
            PageAction::MovePinned { item, spot } => {
                self.edit_session(|session| session.arrange_pinned(item, spot), cx)
            }
            PageAction::OpenDownload { path } if self.downloaded(&path, cx) => open_file(&path),
            PageAction::ShowDownloadInFolder { path } if self.downloaded(&path, cx) => {
                show_in_folder(&path)
            }
            PageAction::OpenDownload { .. } | PageAction::ShowDownloadInFolder { .. } => {}
            PageAction::PauseDownload { path } => self.control_download(&path, Control::Pause, cx),
            PageAction::ResumeDownload { path } => {
                self.control_download(&path, Control::Resume, cx)
            }
            PageAction::CancelDownload { path } => {
                self.control_download(&path, Control::Cancel, cx)
            }
            PageAction::ClearDownloads => self.downloads.update(cx, Downloads::clear),
            PageAction::OpenSettingsSection { section } => self.open_section(&section, cx),
        }
    }

    fn go(&self, view: &Entity<WebView>, text: &str, cx: &mut Context<Self>) {
        let url = destination(text, &self.settings().general.search).url();
        view.update(cx, |view, cx| view.open(url, cx));
    }

    fn downloaded(&self, path: &Path, cx: &App) -> bool {
        self.downloads.read(cx).find(path).is_some()
    }

    fn control_download(&self, path: &Path, control: Control, cx: &App) {
        let downloads = self.downloads.read(cx);
        if let Some(id) = downloads.find(path) {
            downloads.control(id, control);
        }
    }

    fn open_section(&mut self, anchor: &str, cx: &mut Context<Self>) {
        if let Some(section) = Section::parse(anchor) {
            self.show_settings(section, cx);
        }
    }

    fn check_for_updates(&mut self, cx: &mut Context<Self>) {
        let updates = self.settings().updates.clone();
        updater(cx).update(cx, |updater, cx| updater.check(&updates, cx))
    }

    fn set_setting(&mut self, key: &str, value: &str, cx: &mut Context<Self>) {
        let channel = self.settings().updates.channel;
        self.change_settings(|settings| settings.set(key, value), cx);
        if self.settings().updates.channel != channel {
            self.check_for_updates(cx);
        }
    }

    fn answer_settings(
        &mut self,
        view: WeakEntity<WebView>,
        message: settings::Message,
        cx: &mut Context<Self>,
    ) {
        match message {
            settings::Message::Ready => self.greet_settings(view, cx),
            settings::Message::SetSetting { key, value } => self.set_setting(&key, &value, cx),
            settings::Message::SetSponsorblock { key, value } => {
                self.change_settings(|settings| settings.sponsorblock.set(&key, &value), cx)
            }
            settings::Message::UnpauseChannel { channel } => {
                self.change_settings(|settings| settings.sponsorblock.resume(&channel), cx)
            }
            settings::Message::UnpauseSite { site } => self.resume_site(&site, cx),
            settings::Message::ResetDarkSite { site } => {
                self.change_settings(|settings| settings.dark_mode.reset(&site), cx)
            }
            settings::Message::ClearZoom { site } => self.reset_site_zoom(&site, cx),
            settings::Message::SetPermission {
                origin,
                kind,
                state,
            } => self.set_permission(&origin, kind, state, cx),
            settings::Message::ChooseDownloadFolder => self.choose_download_folder(cx),
            settings::Message::ClearBrowsingData => self.clear_browsing_data(view, cx),
            settings::Message::OpenDataFolder => open_file(self.store.folder()),
            settings::Message::CheckForUpdates => self.check_for_updates(cx),
            settings::Message::InstallUpdate => updater(cx).update(cx, Updater::install),
            settings::Message::RunImport(import) => self.import(view, import, cx),
            settings::Message::ExportBackup => self.export_backup(view, cx),
            settings::Message::ImportBackup => self.import_backup(view, cx),
            settings::Message::Passwords { action } => self.answer_passwords(view, action, cx),
            settings::Message::CreateProfile { name } => self.create_profile(&name, cx),
            settings::Message::RenameProfile { profile, name } => {
                self.rename_profile(&profile, &name, cx)
            }
            settings::Message::DeleteProfile { profile } => self.delete_profile(&profile, cx),
            settings::Message::AssignProfile { space, profile } => {
                self.assign_profile(space, &profile, cx)
            }
        }
    }

    fn greet_settings(&self, view: WeakEntity<WebView>, cx: &mut Context<Self>) {
        let state = self.settings_state(cx);
        let profiles = cx.background_spawn(async { Reply::import_profiles(&detect()) });
        cx.spawn(async move |_, cx| {
            reply(&view, &state, cx);
            reply(&view, &profiles.await, cx);
        })
        .detach();
    }

    fn reset_site_zoom(&mut self, site: &str, cx: &mut Context<Self>) {
        self.runtime.reset_zoom(site);
        self.zoom_changed(cx);
    }

    pub(super) fn zoom_changed(&self, cx: &mut Context<Self>) {
        self.apply_zoom_everywhere(cx);
        window::each_browser(cx, |browser, cx| {
            browser.refresh_pages(&[Page::Settings], cx)
        });
    }

    fn choose_download_folder(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(self.text(Text::UseThisFolder).into()),
        });
        cx.spawn(async move |browser, cx| {
            let chosen = picked.await.ok().and_then(|paths| paths.ok()).flatten();
            let Some(folder) = chosen.and_then(|paths| paths.into_iter().next()) else {
                return;
            };
            browser
                .update(cx, |browser, cx| {
                    browser.change_settings(
                        |settings| settings.general.downloads.folder = Some(folder),
                        cx,
                    )
                })
                .ok();
        })
        .detach();
    }

    fn clear_browsing_data(&mut self, view: WeakEntity<WebView>, cx: &mut Context<Self>) {
        let cleared = self.forget_browsing(cx);
        cx.spawn(async move |_, cx| {
            let error = cleared.await.err().map(|error| error.message());
            reply(&view, &Reply::cleared(error), cx);
        })
        .detach();
    }

    fn import(&mut self, view: WeakEntity<WebView>, import: Import, cx: &mut Context<Self>) {
        let reading = cx.background_spawn(async move { read(&import) });
        cx.spawn(async move |browser, cx| {
            let (label, imported) = reading.await;
            let report =
                browser.update(cx, |browser, cx| browser.merge_import(&label, imported, cx));
            if let Ok(report) = report {
                reply(&view, &report, cx);
            }
        })
        .detach();
    }

    pub(super) fn merge_import(
        &mut self,
        label: &str,
        imported: Imported,
        cx: &mut Context<Self>,
    ) -> Reply {
        let mut report = imported.apply(label, &mut self.store, Epoch::now());
        if let Err(error) = self.store.save() {
            report.warnings.push(format!("Saving: {error}"));
        }
        self.refresh_pages(&[Page::Bookmarks, Page::NewTab], cx);
        history_changed(cx);
        cx.notify();
        Reply::imported(report)
    }
}
