use url::Url;
use webview2_com::{
    ContextMenuRequestedEventHandler, CustomItemSelectedEventHandler,
    ExecuteScriptCompletedHandler, Microsoft::Web::WebView2::Win32::*,
};
use windows::{
    Win32::{Foundation::POINT, System::Com::IStream},
    core::{BOOL, HSTRING, Interface, PWSTR, Result},
};

use super::{
    Disposition,
    link::web,
    native::{Native, report},
    signal::{Signal, Signals, address, text},
    zoom,
};

const PICTURE_IN_PICTURE: &str = include_str!("picture_in_picture.js");

#[derive(Clone, Copy)]
enum Choice {
    Open(Disposition),
    Copy,
    PictureInPicture,
    DownloadVideo,
}

struct Entry {
    label: &'static str,
    choice: Choice,
}

struct Swap {
    stock: &'static str,
    entry: Entry,
}

struct Section {
    top: &'static [Entry],
    swaps: &'static [Swap],
    dropped: &'static [&'static str],
    sourced: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

struct Subject {
    section: &'static Section,
    url: Option<Url>,
    page: Option<Url>,
    point: Point,
}

#[derive(Clone, Copy)]
enum Probe {
    Find,
    Toggle,
}

const PICTURE_IN_PICTURE_ENTRY: Entry = Entry {
    label: "Picture-in-Picture",
    choice: Choice::PictureInPicture,
};

const DOWNLOAD_VIDEO_ENTRY: Entry = Entry {
    label: "Download Video",
    choice: Choice::DownloadVideo,
};

const LINK: Section = Section {
    top: &[
        Entry {
            label: "Open Link in New Tab",
            choice: Choice::Open(Disposition::Foreground),
        },
        Entry {
            label: "Open Link in Background Tab",
            choice: Choice::Open(Disposition::Background),
        },
        Entry {
            label: "Open Link in Split View",
            choice: Choice::Open(Disposition::Split),
        },
    ],
    swaps: &[Swap {
        stock: "copyLinkLocation",
        entry: Entry {
            label: "Copy Link Address",
            choice: Choice::Copy,
        },
    }],
    dropped: &["openLinkInNewWindow"],
    sourced: &[],
};

const IMAGE: Section = Section {
    top: &[Entry {
        label: "Open Image in New Tab",
        choice: Choice::Open(Disposition::Foreground),
    }],
    swaps: &[Swap {
        stock: "copyImageLocation",
        entry: Entry {
            label: "Copy Image Address",
            choice: Choice::Copy,
        },
    }],
    dropped: &[],
    sourced: &[],
};

const VIDEO: Section = Section {
    top: &[PICTURE_IN_PICTURE_ENTRY, DOWNLOAD_VIDEO_ENTRY],
    swaps: &[Swap {
        stock: "copyLink",
        entry: Entry {
            label: "Copy Video Address",
            choice: Choice::Copy,
        },
    }],
    dropped: &["pictureInPciture", "pictureInPicture"],
    sourced: &["saveMediaAs"],
};

const VIDEO_UNDER_PAGE: Section = Section {
    top: &[PICTURE_IN_PICTURE_ENTRY, DOWNLOAD_VIDEO_ENTRY],
    swaps: &[],
    dropped: &[],
    sourced: &[],
};

#[derive(Clone)]
struct Menu {
    environment: ICoreWebView2Environment9,
    webview: ICoreWebView2,
    controller: ICoreWebView2Controller,
    signals: Signals,
}

fn flag(read: impl FnOnce(*mut BOOL) -> Result<()>) -> Result<bool> {
    let mut value = BOOL::default();
    read(&mut value)?;
    Ok(value.as_bool())
}

fn web_address(read: impl FnOnce(*mut PWSTR) -> Result<()>) -> Result<Option<Url>> {
    Ok(address(read)?.filter(web))
}

fn link(target: &ICoreWebView2ContextMenuTarget) -> Result<Option<Url>> {
    if !flag(|value| unsafe { target.HasLinkUri(value) })? {
        return Ok(None);
    }
    web_address(|value| unsafe { target.LinkUri(value) })
}

fn source(target: &ICoreWebView2ContextMenuTarget) -> Result<Option<Url>> {
    web_address(|value| unsafe { target.SourceUri(value) })
}

fn page(target: &ICoreWebView2ContextMenuTarget) -> Result<Option<Url>> {
    web_address(|value| unsafe { target.PageUri(value) })
}

fn kind(target: &ICoreWebView2ContextMenuTarget) -> Result<COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND> {
    let mut kind = COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND::default();
    unsafe { target.Kind(&mut kind) }?;
    Ok(kind)
}

fn media_section(kind: COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND) -> Option<&'static Section> {
    match kind {
        COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND_IMAGE => Some(&IMAGE),
        COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND_VIDEO => Some(&VIDEO),
        _ => None,
    }
}

fn position(items: &ICoreWebView2ContextMenuItemCollection, stock: &str) -> Result<Option<u32>> {
    let mut count = 0;
    unsafe { items.Count(&mut count) }?;
    for index in 0..count {
        let item = unsafe { items.GetValueAtIndex(index) }?;
        if text(|value| unsafe { item.Name(value) })? == stock {
            return Ok(Some(index));
        }
    }
    Ok(None)
}

fn probe_script(point: Point, probe: Probe) -> HSTRING {
    let action = match probe {
        Probe::Find => "find",
        Probe::Toggle => "toggle",
    };
    let source = PICTURE_IN_PICTURE.trim_end();
    HSTRING::from(format!(
        "({source})({}, {}, \"{action}\")",
        point.x, point.y
    ))
}

fn probe(
    webview: &ICoreWebView2,
    point: Point,
    probe: Probe,
    answered: impl FnOnce(bool) -> Result<()> + 'static,
) -> Result<()> {
    let completed = ExecuteScriptCompletedHandler::create(Box::new(move |result, json| {
        answered(result.is_ok() && json == "true")
    }));
    unsafe { webview.ExecuteScript(&probe_script(point, probe), &completed) }
}

impl Subject {
    fn removed(&self) -> impl Iterator<Item = &'static str> {
        let section = self.section;
        let unsourced = self.url.is_none().then_some(section.sourced);
        let stock = section
            .dropped
            .iter()
            .chain(unsourced.into_iter().flatten());
        stock.copied()
    }
}

impl Menu {
    fn create(
        &self,
        label: &str,
        kind: COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND,
    ) -> Result<ICoreWebView2ContextMenuItem> {
        let label = HSTRING::from(label);
        unsafe {
            self.environment
                .CreateContextMenuItem(&label, None::<&IStream>, kind)
        }
    }

    fn on_selected(&self, choice: Choice, subject: &Subject) -> Option<Box<dyn Fn()>> {
        let signals = self.signals.clone();
        match (choice, subject.url.clone()) {
            (Choice::Open(disposition), Some(url)) => Some(Box::new(move || {
                signals.send(Signal::Opened(url.clone(), disposition))
            })),
            (Choice::Copy, Some(url)) => Some(Box::new(move || {
                signals.send(Signal::Copied(url.to_string()))
            })),
            (Choice::PictureInPicture, _) => {
                let (webview, point) = (self.webview.clone(), subject.point);
                Some(Box::new(move || {
                    report(probe(&webview, point, Probe::Toggle, |_| Ok(())))
                }))
            }
            (Choice::DownloadVideo, url) => {
                let video = url.or_else(|| subject.page.clone())?;
                Some(Box::new(move || {
                    signals.send(Signal::DownloadVideo(video.clone()))
                }))
            }
            (Choice::Open(_) | Choice::Copy, None) => None,
        }
    }

    fn entry(
        &self,
        entry: &Entry,
        subject: &Subject,
    ) -> Result<Option<ICoreWebView2ContextMenuItem>> {
        let Some(selected) = self.on_selected(entry.choice, subject) else {
            return Ok(None);
        };
        let item = self.create(entry.label, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND)?;
        let handler = CustomItemSelectedEventHandler::create(Box::new(move |_, _| {
            selected();
            Ok(())
        }));
        unsafe { item.add_CustomItemSelected(&handler, &mut 0) }?;
        Ok(Some(item))
    }

    fn top(&self, subject: &Subject) -> Result<Vec<ICoreWebView2ContextMenuItem>> {
        let entries = subject.section.top.iter();
        let created = entries.map(|entry| self.entry(entry, subject).transpose());
        created.flatten().collect()
    }

    fn apply(
        &self,
        items: &ICoreWebView2ContextMenuItemCollection,
        subject: &Subject,
    ) -> Result<()> {
        let section = subject.section;
        for stock in subject.removed() {
            if let Some(index) = position(items, stock)? {
                unsafe { items.RemoveValueAtIndex(index) }?;
            }
        }
        let mut top = self.top(subject)?;
        for swap in section.swaps {
            let Some(item) = self.entry(&swap.entry, subject)? else {
                continue;
            };
            match position(items, swap.stock)? {
                Some(index) => unsafe {
                    items.RemoveValueAtIndex(index)?;
                    items.InsertValueAtIndex(index, &item)?;
                },
                None => top.push(item),
            }
        }
        if top.is_empty() {
            return Ok(());
        }
        top.push(self.create("", COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SEPARATOR)?);
        for (index, item) in (0..).zip(top) {
            unsafe { items.InsertValueAtIndex(index, &item) }?;
        }
        Ok(())
    }

    fn point(&self, args: &ICoreWebView2ContextMenuRequestedEventArgs) -> Result<Point> {
        let mut location = POINT::default();
        unsafe { args.Location(&mut location) }?;
        let zoom = zoom::factor(&self.controller)?;
        Ok(Point {
            x: f64::from(location.x) / zoom,
            y: f64::from(location.y) / zoom,
        })
    }

    fn offer_video_under_page(
        &self,
        args: &ICoreWebView2ContextMenuRequestedEventArgs,
        subject: Subject,
    ) -> Result<()> {
        let deferral = unsafe { args.GetDeferral() }?;
        let items = unsafe { args.MenuItems() }?;
        let (menu, pending) = (self.clone(), deferral.clone());
        let started = probe(&self.webview, subject.point, Probe::Find, move |found| {
            if found {
                report(menu.apply(&items, &subject));
            }
            unsafe { pending.Complete() }
        });
        if started.is_err() {
            unsafe { deferral.Complete() }?;
        }
        started
    }

    fn customize(&self, args: &ICoreWebView2ContextMenuRequestedEventArgs) -> Result<()> {
        let target = unsafe { args.ContextMenuTarget() }?;
        let items = unsafe { args.MenuItems() }?;
        let point = self.point(args)?;
        let page = page(&target)?;
        let kind = kind(&target)?;
        if let Some(section) = media_section(kind) {
            let url = source(&target)?;
            self.apply(
                &items,
                &Subject {
                    section,
                    url,
                    page: page.clone(),
                    point,
                },
            )?;
        }
        if let Some(url) = link(&target)? {
            let subject = Subject {
                section: &LINK,
                url: Some(url),
                page,
                point,
            };
            return self.apply(&items, &subject);
        }
        let on_page = kind == COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND_PAGE;
        if !on_page || flag(|value| unsafe { target.IsEditable(value) })? {
            return Ok(());
        }
        let subject = Subject {
            section: &VIDEO_UNDER_PAGE,
            url: None,
            page,
            point,
        };
        self.offer_video_under_page(args, subject)
    }
}

pub fn wire(
    native: &Native,
    environment: &ICoreWebView2Environment,
    signals: Signals,
) -> Result<()> {
    let menu = Menu {
        environment: environment.cast()?,
        webview: native.webview.clone(),
        controller: native.controller.clone(),
        signals,
    };
    let handler = ContextMenuRequestedEventHandler::create(Box::new(move |_, args| {
        if let Some(args) = args {
            report(menu.customize(&args));
        }
        Ok(())
    }));
    let webview = native.webview.cast::<ICoreWebView2_11>()?;
    unsafe { webview.add_ContextMenuRequested(&handler, &mut 0) }
}
