use std::{fs, path::Path};

use gpui::{Context, PathPromptOptions, WeakEntity};

use super::{browser::Browser, internal::reply};
use crate::{
    backup::Backup,
    internal::settings::{BackupStatus, Reply, Section},
    locale::Text,
    webview::{WebView, default_downloads},
};
use magpie_engine::file;

const FILE_NAME: &str = "Magpie backup.json";

fn read(path: &Path) -> Result<Backup, String> {
    let json = fs::read_to_string(path).map_err(|error| format!("Reading: {error}"))?;
    Backup::parse(&json).map_err(|error| error.to_string())
}

fn failed(message: String) -> Reply {
    Reply::backup(BackupStatus::Failed, message)
}

impl Browser {
    pub(super) fn export_backup(&self, view: WeakEntity<WebView>, cx: &mut Context<Self>) {
        let json = Backup::new(self.settings(), &self.store.session).json();
        let folder = default_downloads().unwrap_or_else(|| self.store.folder().to_owned());
        let chosen = cx.prompt_for_new_path(&folder, Some(FILE_NAME));
        let saved = Reply::backup(BackupStatus::Done, self.text(Text::BackupSaved));
        cx.spawn(async move |_, cx| {
            let Some(path) = chosen.await.ok().and_then(|picked| picked.ok()).flatten() else {
                return;
            };
            let written = cx
                .background_executor()
                .spawn(async move { file::replace(&path, json.as_bytes()) })
                .await;
            let outcome = match written {
                Ok(()) => saved,
                Err(error) => failed(format!("Saving: {error}")),
            };
            reply(&view, &outcome, cx);
        })
        .detach();
    }

    pub(super) fn import_backup(&self, view: WeakEntity<WebView>, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: None,
        });
        cx.spawn(async move |browser, cx| {
            let chosen = picked.await.ok().and_then(|paths| paths.ok()).flatten();
            let Some(path) = chosen.and_then(|paths| paths.into_iter().next()) else {
                return;
            };
            match cx
                .background_executor()
                .spawn(async move { read(&path) })
                .await
            {
                Ok(backup) => browser
                    .update(cx, |browser, cx| browser.restore_backup(backup, cx))
                    .unwrap_or_default(),
                Err(message) => reply(&view, &failed(message), cx),
            }
        })
        .detach();
    }

    fn restore_backup(&mut self, backup: Backup, cx: &mut Context<Self>) {
        let settings = backup.settings();
        self.change_settings(|current| *current = settings, cx);
        self.replace_session(backup.into_session(), cx);
        self.show_settings(Section::Import, cx);
    }
}
