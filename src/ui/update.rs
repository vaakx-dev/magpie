use std::{path::Path, time::SystemTime};

use futures::{StreamExt, channel::mpsc::unbounded};
use gpui::{App, AppContext, Context, Entity, prelude::*};

use super::{browser::Browser, window};
use crate::{
    config::Updates,
    internal::settings::Section,
    locale::Text,
    update::{self, Fault, Phase},
};
use magpie_engine::update::{Offer, Request, Schedule, Status, check, fetch};

const SCHEDULE_FILE: &str = "update-check";

pub struct Updater {
    phase: Phase,
    dismissed: bool,
    schedule: Schedule,
}

fn last_attempt(folder: &Path) -> Phase {
    let failure = update::failure(folder);
    if let Err(error) = update::clear(folder) {
        eprintln!("magpie: could not remove old update files: {error}");
    }
    match failure {
        Ok(Some(message)) => Phase::InstallFailed { message },
        Ok(None) => Phase::Idle,
        Err(error) => {
            eprintln!("magpie: could not read how the last update went: {error}");
            Phase::Idle
        }
    }
}

impl Updater {
    pub fn new(config_folder: &Path) -> Self {
        Self {
            phase: last_attempt(&update::folder()),
            dismissed: false,
            schedule: Schedule::at(config_folder.join(SCHEDULE_FILE)),
        }
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    fn busy(&self) -> bool {
        matches!(
            self.phase,
            Phase::Checking | Phase::Downloading { .. } | Phase::Installing { .. }
        )
    }

    fn announcement(&self) -> Option<&str> {
        match &self.phase {
            Phase::Available { offer } if !self.dismissed => Some(&offer.version),
            _ => None,
        }
    }

    fn installable(&self) -> Option<&Offer> {
        match &self.phase {
            Phase::Available { offer } | Phase::DownloadFailed { offer, .. } => Some(offer),
            _ => None,
        }
    }

    pub fn check_if_due(&mut self, updates: &Updates, cx: &mut Context<Self>) {
        if updates.automatic.on()
            && self.phase == Phase::Idle
            && self.schedule.due(SystemTime::now())
        {
            self.check(updates, cx);
        }
    }

    pub fn check(&mut self, updates: &Updates, cx: &mut Context<Self>) {
        if self.busy() {
            return;
        }
        self.phase = Phase::Checking;
        cx.notify();
        let repo = updates.source();
        let channel = updates.channel;
        let checking = cx.background_spawn(async move {
            let request = Request {
                repo: &repo,
                current: &update::current(),
                platform: update::platform(),
                channel,
            };
            check(&request, fetch)
        });
        cx.spawn(async move |updater, cx| {
            let status = checking.await;
            updater
                .update(cx, |updater, cx| updater.checked(status, cx))
                .ok();
        })
        .detach();
    }

    fn checked(&mut self, status: Status, cx: &mut Context<Self>) {
        if let Err(error) = self.schedule.record(SystemTime::now()) {
            eprintln!("magpie: could not note the update check: {error}");
        }
        self.phase = status.into();
        self.dismissed = false;
        cx.notify();
    }

    pub fn install(&mut self, cx: &mut Context<Self>) {
        let Some(offer) = self.installable().cloned() else {
            return;
        };
        let Some(download) = offer.download.clone() else {
            return;
        };
        self.phase = Phase::Downloading {
            offer: offer.clone(),
            percent: 0,
        };
        cx.notify();
        let (progress, mut reports) = unbounded();
        let installing = cx.background_spawn(async move {
            let report = |percent| {
                progress.unbounded_send(percent).ok();
            };
            let file = update::download(&download, &update::folder(), report)?;
            update::install(&file, download.package).map_err(Fault::Disk)
        });
        cx.spawn(async move |updater, cx| {
            while let Some(percent) = reports.next().await {
                updater
                    .update(cx, |updater, cx| updater.progressed(percent, cx))
                    .ok();
            }
            let outcome = installing.await;
            updater
                .update(cx, |updater, cx| updater.finished(offer, outcome, cx))
                .ok();
        })
        .detach();
    }

    fn progressed(&mut self, done: u8, cx: &mut Context<Self>) {
        if let Phase::Downloading { percent, .. } = &mut self.phase {
            *percent = done;
            cx.notify();
        }
    }

    fn finished(&mut self, offer: Offer, outcome: Result<(), Fault>, cx: &mut Context<Self>) {
        match outcome {
            Ok(()) => {
                self.phase = Phase::Installing { offer };
                cx.quit();
            }
            Err(fault) => {
                self.phase = Phase::DownloadFailed {
                    offer,
                    message: fault.to_string(),
                };
                cx.notify();
            }
        }
    }

    fn dismiss(&mut self, cx: &mut Context<Self>) {
        self.dismissed = true;
        cx.notify();
    }
}

pub(super) fn updater(cx: &App) -> Entity<Updater> {
    cx.global::<window::Shared>().updater.clone()
}

impl Browser {
    fn see_update(&mut self, cx: &mut Context<Self>) {
        self.show_settings(Section::About, cx);
    }

    fn dismiss_update(&mut self, cx: &mut Context<Self>) {
        updater(cx).update(cx, Updater::dismiss);
    }

    pub(super) fn update_banner(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let message = updater(cx)
            .read(cx)
            .announcement()
            .map(|version| self.locale().fill(Text::UpdateAvailable, version))?;
        let buttons = [
            self.banner_button("see-update", Text::SeeUpdate, Self::see_update, cx),
            self.banner_button("dismiss-update", Text::NotNow, Self::dismiss_update, cx),
        ];
        Some(self.banner(message, buttons))
    }
}
