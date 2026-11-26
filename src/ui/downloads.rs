use std::{
    borrow::Borrow, os::windows::process::CommandExt, path::Path, process::Command, time::Instant,
};

use futures::{
    StreamExt,
    channel::mpsc::{UnboundedSender, unbounded},
};
use gpui::{ClickEvent, Context, Div, Stateful, Task, div, prelude::*, px, relative, rgba};
use serde_json::Value;

use super::{
    browser::Browser,
    icon::Icon,
    surface::{Colors, tint},
    toolbar::icon_button,
};
use crate::{
    download::{Download, Meter, shelf},
    pages::downloads_payload,
    stream_download::{Stream, download_stream},
    webview::Transfer,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DownloadId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Pause,
    Resume,
    Cancel,
}

enum Source {
    Page(Transfer),
    Stream(Stream),
}

struct Entry {
    id: DownloadId,
    source: Source,
    download: Download,
    meter: Meter,
}

pub struct Downloads {
    entries: Vec<Entry>,
    next_id: u64,
    changes: UnboundedSender<()>,
    _pump: Task<()>,
}

const PROGRESS_HEIGHT_PX: f32 = 4.0;

fn launch(command: &mut Command) {
    if let Err(error) = command.spawn() {
        eprintln!("magpie: could not open Explorer: {error}");
    }
}

pub fn open_file(path: &Path) {
    launch(Command::new("explorer").arg(path));
}

pub fn show_in_folder(path: &Path) {
    let select = format!("/select,\"{}\"", path.display());
    launch(Command::new("explorer").raw_arg(select));
}

impl Source {
    fn snapshot(&self) -> Option<Download> {
        match self {
            Source::Page(transfer) => transfer
                .download()
                .inspect_err(|error| eprintln!("magpie: could not read a download: {error}"))
                .ok(),
            Source::Stream(stream) => Some(stream.download()),
        }
    }

    fn control(&self, control: Control) {
        match (self, control) {
            (Source::Page(transfer), Control::Pause) => transfer.pause(),
            (Source::Page(transfer), Control::Resume) => transfer.resume(),
            (Source::Page(transfer), Control::Cancel) => transfer.cancel(),
            (Source::Stream(stream), Control::Cancel) => stream.cancel(),
            (Source::Stream(_), Control::Pause | Control::Resume) => {}
        }
    }
}

impl Borrow<Download> for Entry {
    fn borrow(&self) -> &Download {
        &self.download
    }
}

impl Downloads {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (changes, mut received) = unbounded();
        let pump = cx.spawn(async move |downloads, cx| {
            while received.next().await.is_some() {
                if downloads.update(cx, Self::refresh).is_err() {
                    return;
                }
            }
        });
        Self {
            entries: Vec::new(),
            next_id: 0,
            changes,
            _pump: pump,
        }
    }

    pub fn track(&mut self, transfer: Transfer, cx: &mut Context<Self>) {
        let changes = self.changes.clone();
        let watched = transfer.watch(move || {
            changes.unbounded_send(()).ok();
        });
        if let Err(error) = watched {
            eprintln!("magpie: could not follow a download: {error}");
        }
        self.insert(Source::Page(transfer), cx);
    }

    pub fn stream(&mut self, page_url: &str, target_dir: &Path, cx: &mut Context<Self>) {
        let changes = self.changes.clone();
        let stream = download_stream(page_url, target_dir, move |_| {
            changes.unbounded_send(()).ok();
        });
        self.insert(Source::Stream(stream), cx);
    }

    fn insert(&mut self, source: Source, cx: &mut Context<Self>) {
        let Some(download) = source.snapshot() else {
            return;
        };
        let id = DownloadId(self.next_id);
        self.next_id += 1;
        self.entries.insert(
            0,
            Entry {
                id,
                source,
                download,
                meter: Meter::default(),
            },
        );
        cx.notify();
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        let now = Instant::now();
        for entry in self
            .entries
            .iter_mut()
            .filter(|entry| entry.download.active())
        {
            if let Some(download) = entry.source.snapshot() {
                entry.download = entry.meter.measure(download, now);
            }
        }
        cx.notify();
    }

    pub fn shelf(&self, query: &str) -> Vec<(DownloadId, Download)> {
        shelf(&self.entries, query)
            .into_iter()
            .map(|entry| (entry.id, entry.download.clone()))
            .collect()
    }

    pub fn payload(&self) -> Value {
        downloads_payload(&self.entries)
    }

    pub fn find(&self, path: &Path) -> Option<DownloadId> {
        self.entries
            .iter()
            .find(|entry| entry.download.path == path)
            .map(|entry| entry.id)
    }

    pub fn control(&self, id: DownloadId, control: Control) {
        if let Some(entry) = self.entries.iter().find(|entry| entry.id == id) {
            entry.source.control(control);
        }
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.entries.retain(|entry| entry.download.active());
        cx.notify();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn active(&self) -> bool {
        self.entries.iter().any(|entry| entry.download.active())
    }

    pub fn finished(&self) -> bool {
        self.entries.iter().any(|entry| !entry.download.active())
    }
}

pub fn progress_bar(fraction: f32, colors: &Colors) -> impl IntoElement {
    div()
        .h(px(PROGRESS_HEIGHT_PX))
        .w_full()
        .rounded_full()
        .bg(rgba(colors.overlay))
        .child(
            div()
                .h_full()
                .w(relative(fraction))
                .rounded_full()
                .bg(rgba(colors.muted)),
        )
}

impl Browser {
    pub(super) fn downloads_button(&self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        let downloads = self.downloads.read(cx);
        if downloads.is_empty() {
            return None;
        }
        let ink = self.colors().text;
        let button = icon_button("downloads", Icon::Download, self.colors())
            .when(downloads.active(), |button| button.bg(tint(ink, 0x2e)))
            .on_click(cx.listener(|browser, _: &ClickEvent, window, cx| {
                cx.stop_propagation();
                browser.toggle_downloads(window, cx);
            }));
        Some(button)
    }
}
