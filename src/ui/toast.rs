use std::time::Duration;

use gpui::{
    ClickEvent, Context, Div, MouseButton, MouseDownEvent, Stateful, Task, Window, div, prelude::*,
    px, rgba,
};

use super::{
    browser::Browser,
    downloads::{Control, DownloadId, open_file, progress_bar, show_in_folder},
    icon::Icon,
    surface::{PANEL_CORNER_PX, surface, text_button},
    toolbar::icon_button,
};
use crate::{
    download::{Download, State},
    internal::Page,
    locale::Text,
};

pub struct Toast {
    id: DownloadId,
    lingering: Option<Task<()>>,
}

#[derive(Clone, Copy)]
enum Action {
    AllDownloads,
    Cancel,
    ShowInFolder,
    Open,
}

const WIDTH_PX: f32 = 360.0;
const LINGER: Duration = Duration::from_secs(6);

impl Action {
    fn look(self) -> (&'static str, Text) {
        match self {
            Action::AllDownloads => ("download-toast-all", Text::AllDownloads),
            Action::Cancel => ("download-toast-cancel", Text::Cancel),
            Action::ShowInFolder => ("download-toast-folder", Text::ShowInFolder),
            Action::Open => ("download-toast-open", Text::OpenFile),
        }
    }
}

fn actions(state: State) -> &'static [Action] {
    match state {
        State::InProgress | State::Paused => &[Action::AllDownloads, Action::Cancel],
        State::Completed => &[Action::AllDownloads, Action::ShowInFolder, Action::Open],
        State::Interrupted | State::Cancelled => &[Action::AllDownloads],
    }
}

impl Browser {
    pub(super) fn announce(&mut self, id: Option<DownloadId>, cx: &mut Context<Self>) {
        let Some(id) = id else {
            return;
        };
        self.toast = Some(Toast {
            id,
            lingering: None,
        });
        cx.notify();
    }

    pub(super) fn settle_toast(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self
            .toast
            .as_ref()
            .filter(|toast| toast.lingering.is_none())
            .map(|toast| toast.id)
        else {
            return;
        };
        if self
            .downloads
            .read(cx)
            .download(id)
            .is_some_and(Download::active)
        {
            return;
        }
        let lingering = cx.spawn(async move |browser, cx| {
            cx.background_executor().timer(LINGER).await;
            browser.update(cx, Self::dismiss_toast).ok();
        });
        self.toast = Some(Toast {
            id,
            lingering: Some(lingering),
        });
    }

    pub(super) fn dismiss_toast(&mut self, cx: &mut Context<Self>) {
        if self.toast.take().is_some() {
            cx.notify();
        }
    }

    pub(super) fn click_away_from_toast(
        &mut self,
        _: &MouseDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.dismiss_toast(cx);
    }

    fn act_on_toast(&mut self, id: DownloadId, action: Action, cx: &mut Context<Self>) {
        let downloads = self.downloads.read(cx);
        let Some(path) = downloads.download(id).map(|download| download.path.clone()) else {
            return;
        };
        match action {
            Action::Cancel => return downloads.control(id, Control::Cancel),
            Action::AllDownloads => self.show_page(Page::Downloads, cx),
            Action::ShowInFolder => show_in_folder(&path),
            Action::Open => open_file(&path),
        }
        self.dismiss_toast(cx);
    }

    fn toast_button(
        &self,
        id: DownloadId,
        action: Action,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (name, label) = action.look();
        text_button(name, self.text(label), self.colors()).on_click(
            cx.listener(move |browser, _: &ClickEvent, _, cx| browser.act_on_toast(id, action, cx)),
        )
    }

    fn toast_heading(&self, download: &Download, cx: &mut Context<Self>) -> Div {
        let colors = self.colors();
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(Icon::Download.tinted(rgba(colors.muted)))
            .child(div().flex_1().min_w_0().truncate().child(download.name()))
            .child(
                icon_button("download-toast-close", Icon::X, colors).on_click(
                    cx.listener(|browser, _: &ClickEvent, _, cx| browser.dismiss_toast(cx)),
                ),
            )
    }

    pub(super) fn download_toast(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let id = self.toast.as_ref()?.id;
        let download = self.downloads.read(cx).download(id)?.clone();
        let colors = self.colors();
        let progress = download.progress().filter(|_| download.active());
        let buttons = actions(download.state)
            .iter()
            .map(|action| self.toast_button(id, *action, cx))
            .collect::<Vec<_>>();
        let card = surface(colors)
            .w(px(WIDTH_PX))
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .rounded(px(PANEL_CORNER_PX))
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(self.toast_heading(&download, cx))
            .children(progress.map(|fraction| progress_bar(fraction, colors)))
            .child(
                div()
                    .text_xs()
                    .text_color(rgba(colors.muted))
                    .child(download.status()),
            )
            .child(div().flex().justify_end().gap_2().children(buttons));
        Some(self.cutouts.see_through(PANEL_CORNER_PX).child(card))
    }
}
