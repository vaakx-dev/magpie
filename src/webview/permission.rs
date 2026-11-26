use std::rc::Rc;

use url::Url;
use webview2_com::{Microsoft::Web::WebView2::Win32::*, PermissionRequestedEventHandler};
use windows::core::{Interface, Result};

use super::{
    Disposition,
    native::report,
    runtime::Runtime,
    signal::{Signal, Signals, address},
};
use crate::storage::permissions::{Kind, Origin, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Granted,
    Denied,
}

enum Waiting {
    Permission {
        args: ICoreWebView2PermissionRequestedEventArgs,
        deferral: ICoreWebView2Deferral,
    },
    Popup(Url),
}

pub struct Prompt {
    pub origin: Origin,
    pub kind: Kind,
    waiting: Waiting,
}

fn kind(kind: COREWEBVIEW2_PERMISSION_KIND) -> Option<Kind> {
    match kind {
        COREWEBVIEW2_PERMISSION_KIND_CAMERA => Some(Kind::Camera),
        COREWEBVIEW2_PERMISSION_KIND_MICROPHONE => Some(Kind::Microphone),
        COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS => Some(Kind::Notifications),
        COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION => Some(Kind::Location),
        COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ => Some(Kind::Clipboard),
        _ => None,
    }
}

fn permission_state(access: Access) -> COREWEBVIEW2_PERMISSION_STATE {
    match access {
        Access::Granted => COREWEBVIEW2_PERMISSION_STATE_ALLOW,
        Access::Denied => COREWEBVIEW2_PERMISSION_STATE_DENY,
    }
}

fn decide(args: &ICoreWebView2PermissionRequestedEventArgs, access: Access) -> Result<()> {
    unsafe { args.SetState(permission_state(access)) }
}

fn request(args: &ICoreWebView2PermissionRequestedEventArgs) -> Result<Option<(Origin, Kind)>> {
    let mut requested = COREWEBVIEW2_PERMISSION_KIND::default();
    unsafe { args.PermissionKind(&mut requested) }?;
    let origin = address(|value| unsafe { args.Uri(value) })?
        .as_ref()
        .and_then(Origin::of);
    Ok(origin.zip(kind(requested)))
}

fn keep_out_of_profile(args: &ICoreWebView2PermissionRequestedEventArgs) -> Result<()> {
    let args = args.cast::<ICoreWebView2PermissionRequestedEventArgs3>()?;
    unsafe { args.SetSavesInProfile(false) }
}

fn requested(
    runtime: &Runtime,
    args: ICoreWebView2PermissionRequestedEventArgs,
    signals: &Signals,
) -> Result<()> {
    let Some((origin, kind)) = request(&args)? else {
        return Ok(());
    };
    keep_out_of_profile(&args)?;
    match runtime.permission(&origin, kind) {
        State::Allow => decide(&args, Access::Granted),
        State::Block => decide(&args, Access::Denied),
        State::Ask => {
            let deferral = unsafe { args.GetDeferral() }?;
            let waiting = Waiting::Permission { args, deferral };
            signals.send(Signal::Permission(Prompt {
                origin,
                kind,
                waiting,
            }));
            Ok(())
        }
    }
}

pub fn open_popup(runtime: &Runtime, opener: Origin, url: Url, signals: &Signals) {
    match runtime.permission(&opener, Kind::Popups) {
        State::Allow => signals.send(Signal::Opened(url, Disposition::Foreground)),
        State::Block => {}
        State::Ask => signals.send(Signal::Permission(Prompt {
            origin: opener,
            kind: Kind::Popups,
            waiting: Waiting::Popup(url),
        })),
    }
}

impl Prompt {
    pub fn settle(self, access: Access) -> Option<Url> {
        match self.waiting {
            Waiting::Permission { args, deferral } => {
                report(decide(&args, access));
                report(unsafe { deferral.Complete() });
                None
            }
            Waiting::Popup(url) => (access == Access::Granted).then_some(url),
        }
    }
}

pub fn wire(webview: &ICoreWebView2, runtime: Rc<Runtime>, signals: Signals) -> Result<()> {
    let handler = PermissionRequestedEventHandler::create(Box::new(move |_, args| match args {
        Some(args) => requested(&runtime, args, &signals),
        None => Ok(()),
    }));
    unsafe { webview.add_PermissionRequested(&handler, &mut 0) }
}
