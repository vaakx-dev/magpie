use gpui::{AnyElement, App, Context, prelude::*, px, rgba};

use super::{browser::Browser, icon::Icon, tab::row_button};
use crate::{
    storage::session::TabId,
    webview::{Audio, WebView},
};

const BADGE_ICON_PX: f32 = 14.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sound {
    Playing,
    Muted,
}

fn sound(audio: &[Audio]) -> Option<Sound> {
    let muted = audio.iter().any(|pane| pane.muted);
    let playing = audio.iter().any(|pane| pane.playing);
    match (muted, playing) {
        (true, _) => Some(Sound::Muted),
        (false, true) => Some(Sound::Playing),
        (false, false) => None,
    }
}

impl Browser {
    fn sound(&self, id: TabId, cx: &App) -> Option<Sound> {
        let panes = self.live.get(&id)?;
        let audio: Vec<Audio> = panes
            .iter()
            .map(|pane| pane.view.read(cx).audio())
            .collect();
        sound(&audio)
    }

    pub(super) fn muted(&self, id: TabId, cx: &App) -> Option<bool> {
        let panes = self.live.get(&id)?;
        Some(panes.iter().any(|pane| pane.view.read(cx).audio().muted))
    }

    pub(super) fn toggle_sound(&mut self, id: TabId, cx: &mut Context<Self>) {
        let Some(muted) = self.muted(id, cx) else {
            return;
        };
        let change: fn(&mut WebView, &mut Context<WebView>) = match muted {
            true => WebView::unmute,
            false => WebView::mute,
        };
        for pane in &self.live[&id] {
            pane.view.update(cx, change);
        }
    }

    pub(super) fn sound_badge(&self, id: TabId, cx: &mut Context<Self>) -> Option<AnyElement> {
        let sound = self.sound(id, cx)?;
        let colors = self.colors();
        let (icon, color) = match sound {
            Sound::Playing => (Icon::Volume, colors.text),
            Sound::Muted => (Icon::VolumeMuted, colors.muted),
        };
        let badge = row_button(
            ("sound", id.0),
            icon.tinted(rgba(color)).size(px(BADGE_ICON_PX)),
            colors,
        );
        let toggle = cx.listener(move |browser, _, _, cx| {
            cx.stop_propagation();
            browser.toggle_sound(id, cx);
        });
        Some(badge.on_click(toggle).into_any_element())
    }
}
