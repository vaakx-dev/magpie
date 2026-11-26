mod player;

use gpui::{
    AnyElement, App, Context, Div, Entity, SharedString, Stateful, div, prelude::*, px, rgba,
};

pub use player::{Candidate, Player};

use super::{
    browser::Browser,
    favicon::site_icon,
    icon::Icon,
    surface::{CONTROL_CORNER_PX, PANEL_CORNER_PX, tint},
    tab::row_button,
    toolbar::icon_button,
};
use crate::{
    storage::session::TabId,
    webview::{
        WebView,
        media::{Control, Playback},
    },
};
use magpie_engine::blocker::site;

const CONTROL_ICON_PX: f32 = 14.0;
const PLAY_BUTTON_PX: f32 = 26.0;
const BUTTON_HIGHLIGHT_ALPHA: u8 = 0x2e;
const PREVIOUS: Skip = Skip {
    id: "mini-player-previous",
    icon: Icon::SkipBack,
    control: Control::Previous,
};
const NEXT: Skip = Skip {
    id: "mini-player-next",
    icon: Icon::SkipForward,
    control: Control::Next,
};

struct Skip {
    id: &'static str,
    icon: Icon,
    control: Control,
}

struct Playing {
    tab: TabId,
    title: SharedString,
    subtitle: SharedString,
    page: String,
    playback: Playback,
    muted: bool,
}

fn playback(view: &WebView) -> Playback {
    match (view.track(), view.audio().playing) {
        (Some(track), _) => track.state,
        (None, true) => Playback::Playing,
        (None, false) => Playback::Paused,
    }
}

fn subtitle(view: &WebView) -> SharedString {
    let artist = view.track().map(|track| track.artist.clone());
    let host = || view.url().and_then(site).unwrap_or_default();
    artist
        .filter(|artist| !artist.is_empty())
        .unwrap_or_else(host)
        .into()
}

fn toggle_icon(playback: Playback) -> Icon {
    match playback {
        Playback::Playing => Icon::Pause,
        Playback::Paused => Icon::Play,
    }
}

impl Browser {
    fn media_view(&self, tab: TabId, cx: &App) -> Option<&Entity<WebView>> {
        let panes = self.live.get(&tab)?;
        let views = || panes.iter().map(|pane| &pane.view);
        let playing = views().find(|view| view.read(cx).audio().playing);
        playing.or_else(|| views().find(|view| view.read(cx).track().is_some()))
    }

    fn candidates(&self, cx: &App) -> Vec<Candidate> {
        self.store
            .session
            .spaces
            .iter()
            .flat_map(|space| space.tabs())
            .filter_map(|tab| {
                let view = self.media_view(tab.id, cx)?.read(cx);
                Some(Candidate {
                    tab: tab.id,
                    playing: view.audio().playing,
                })
            })
            .collect()
    }

    fn playing(&self, cx: &App) -> Option<Playing> {
        let tab = self
            .player
            .shown(&self.candidates(cx), self.current().active)?;
        let page = self.media_view(tab, cx)?.read(cx);
        let title = page
            .track()
            .map_or_else(|| page.title(), |track| track.title.clone().into());
        Some(Playing {
            tab,
            title,
            subtitle: subtitle(page),
            page: page.url().map(|url| url.to_string()).unwrap_or_default(),
            playback: playback(page),
            muted: page.audio().muted,
        })
    }

    fn control_media(&mut self, tab: TabId, control: Control, cx: &mut Context<Self>) {
        let Some(view) = self.media_view(tab, cx) else {
            return;
        };
        view.read(cx).control(control);
        self.player.hold(tab);
        cx.notify();
    }

    fn dismiss_media(&mut self, cx: &mut Context<Self>) {
        self.player.release();
        cx.notify();
    }

    fn skip_button(&self, tab: TabId, skip: Skip, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let glyph = skip
            .icon
            .tinted(rgba(colors.text))
            .size(px(CONTROL_ICON_PX));
        let control = skip.control;
        row_button(skip.id, glyph, colors)
            .on_click(cx.listener(move |browser, _, _, cx| browser.control_media(tab, control, cx)))
    }

    fn play_button(&self, playing: &Playing, cx: &mut Context<Self>) -> Stateful<Div> {
        let tab = playing.tab;
        let colors = self.colors();
        let ink = colors.text;
        let glyph = toggle_icon(playing.playback)
            .tinted(rgba(colors.on_text))
            .size(px(CONTROL_ICON_PX));
        div()
            .id("mini-player-toggle")
            .size(px(PLAY_BUTTON_PX))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(px(CONTROL_CORNER_PX))
            .bg(rgba(ink))
            .hover(move |button| button.bg(tint(ink, 0xd9)))
            .on_click(
                cx.listener(move |browser, _, _, cx| {
                    browser.control_media(tab, Control::Toggle, cx)
                }),
            )
            .child(glyph)
    }

    fn mute_button(&self, playing: &Playing, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let (icon, color) = match playing.muted {
            true => (Icon::VolumeMuted, colors.muted),
            false => (Icon::Volume, colors.text),
        };
        let tab = playing.tab;
        let glyph = icon.tinted(rgba(color)).size(px(CONTROL_ICON_PX));
        row_button("mini-player-mute", glyph, colors)
            .on_click(cx.listener(move |browser, _, _, cx| browser.toggle_sound(tab, cx)))
    }

    fn dismiss_button(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let glyph = Icon::X.tinted(rgba(colors.muted)).size(px(CONTROL_ICON_PX));
        row_button("mini-player-dismiss", glyph, colors)
            .on_click(cx.listener(|browser, _, _, cx| browser.dismiss_media(cx)))
    }

    fn track_details(&self, playing: &Playing, cx: &mut Context<Self>) -> Stateful<Div> {
        let tab = playing.tab;
        div()
            .id("mini-player-track")
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .cursor_pointer()
            .on_click(cx.listener(move |browser, _, _, cx| browser.activate(tab, cx)))
            .child(
                div()
                    .truncate()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(playing.title.clone()),
            )
            .child(
                div()
                    .truncate()
                    .text_xs()
                    .text_color(rgba(self.colors().muted))
                    .child(playing.subtitle.clone()),
            )
    }

    fn transport(&self, playing: &Playing, cx: &mut Context<Self>) -> Div {
        let tab = playing.tab;
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(self.skip_button(tab, PREVIOUS, cx))
            .child(self.play_button(playing, cx))
            .child(self.skip_button(tab, NEXT, cx))
            .child(div().flex_1())
            .child(self.mute_button(playing, cx))
    }

    pub(super) fn mini_player(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let playing = self.playing(cx)?;
        let colors = self.colors();
        let favicon = site_icon(&self.favicons, &playing.page, rgba(colors.muted), cx);
        let paused = playing.playback == Playback::Paused;
        let heading = div()
            .flex()
            .items_center()
            .gap_2()
            .child(favicon)
            .child(self.track_details(&playing, cx))
            .when(paused, |heading| heading.child(self.dismiss_button(cx)));
        let card = div()
            .flex_none()
            .flex()
            .flex_col()
            .gap_1p5()
            .p_2()
            .rounded(px(PANEL_CORNER_PX))
            .bg(rgba(colors.row_active))
            .shadow_sm()
            .child(heading)
            .child(self.transport(&playing, cx));
        Some(card.into_any_element())
    }

    pub(super) fn mini_player_button(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let playing = self.playing(cx)?;
        let tab = playing.tab;
        let colors = self.colors();
        let button = icon_button("mini-player-button", toggle_icon(playing.playback), colors)
            .bg(tint(colors.text, BUTTON_HIGHLIGHT_ALPHA))
            .on_click(cx.listener(move |browser, _, _, cx| {
                browser.control_media(tab, Control::Toggle, cx)
            }));
        Some(button.into_any_element())
    }
}
