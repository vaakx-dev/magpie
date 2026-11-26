use std::{cell::RefCell, rc::Rc};

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use gpui::Keystroke;
use url::Url;
use webview2_com::take_pwstr;
use windows::core::{PWSTR, Result};

use super::{
    Disposition,
    audio::Audio,
    autofill::{Sighting, Submission},
    download::{Prompt, Transfer},
    media::Track,
    permission::Prompt as PermissionPrompt,
};
use crate::{find::Matches, internal, reader, sponsorblock::Request, zoom::Step};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct History {
    pub back: bool,
    pub forward: bool,
}

pub enum Signal {
    Started,
    Rewritten(Url),
    Prompt(Prompt),
    Download(Transfer),
    Zoom(Step),
    ZoomDrifted,
    Matches(Matches),
    Source(Url),
    Title(String),
    History(History),
    Finished,
    Favicon(Vec<u8>),
    Opened(Url, Disposition),
    Copied(String),
    DownloadVideo(Url),
    Audio(Audio),
    Track(Track),
    Shortcut(Keystroke),
    Focused,
    SponsorBlock(Request),
    Reader(reader::Config),
    Fullscreen(bool),
    Internal(internal::Message),
    Permission(PermissionPrompt),
    Login(Sighting),
    Submitted(Submission),
}

#[derive(Clone)]
pub struct Signals(UnboundedSender<Signal>);

pub type CurrentPage = Rc<RefCell<Option<Url>>>;
pub type Requested = Rc<RefCell<Option<Url>>>;

impl Signals {
    pub fn send(&self, signal: Signal) {
        self.0.unbounded_send(signal).ok();
    }
}

pub fn channel() -> (Signals, UnboundedReceiver<Signal>) {
    let (sender, receiver) = unbounded();
    (Signals(sender), receiver)
}

pub fn text(read: impl FnOnce(*mut PWSTR) -> Result<()>) -> Result<String> {
    let mut value = PWSTR::null();
    read(&mut value)?;
    Ok(take_pwstr(value))
}

pub fn address(read: impl FnOnce(*mut PWSTR) -> Result<()>) -> Result<Option<Url>> {
    Ok(Url::parse(&text(read)?).ok())
}
