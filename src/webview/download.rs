use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use webview2_com::{
    BytesReceivedChangedEventHandler, DownloadStartingEventHandler,
    Microsoft::Web::WebView2::Win32::*, SaveAsUIShowingEventHandler, StateChangedEventHandler,
};
use windows::core::{BOOL, HSTRING, Interface, Result};

use super::{
    native::report,
    runtime::Runtime,
    signal::{Signal, Signals, text},
};
use crate::download::{Download, State};

#[derive(Clone)]
pub struct Transfer(ICoreWebView2DownloadOperation);

#[derive(Clone, Default)]
struct Chosen(Rc<RefCell<Option<PathBuf>>>);

enum Request {
    Download {
        args: ICoreWebView2DownloadStartingEventArgs,
        transfer: Transfer,
    },
    SaveAs {
        args: ICoreWebView2SaveAsUIShowingEventArgs,
        chosen: Chosen,
    },
}

pub struct Prompt {
    request: Request,
    deferral: ICoreWebView2Deferral,
}

impl Chosen {
    fn remember(&self, path: PathBuf) {
        self.0.replace(Some(path));
    }

    fn claim(&self, path: &Path) -> bool {
        self.0
            .borrow_mut()
            .take_if(|chosen| chosen == path)
            .is_some()
    }
}

fn path_text(path: &Path) -> HSTRING {
    HSTRING::from(path.as_os_str())
}

fn state(operation: &ICoreWebView2DownloadOperation) -> Result<State> {
    let mut state = COREWEBVIEW2_DOWNLOAD_STATE::default();
    let mut reason = COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON::default();
    let mut resumable = BOOL::default();
    unsafe { operation.State(&mut state) }?;
    unsafe { operation.InterruptReason(&mut reason) }?;
    unsafe { operation.CanResume(&mut resumable) }?;
    Ok(match (state, reason) {
        (COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED, _) => State::Completed,
        (
            COREWEBVIEW2_DOWNLOAD_STATE_INTERRUPTED,
            COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_PAUSED,
        ) if resumable.as_bool() => State::Paused,
        (
            COREWEBVIEW2_DOWNLOAD_STATE_INTERRUPTED,
            COREWEBVIEW2_DOWNLOAD_INTERRUPT_REASON_USER_CANCELED,
        ) => State::Cancelled,
        (COREWEBVIEW2_DOWNLOAD_STATE_INTERRUPTED, _) => State::Interrupted,
        _ => State::InProgress,
    })
}

impl Transfer {
    pub fn download(&self) -> Result<Download> {
        let operation = &self.0;
        let mut received_bytes = 0;
        let mut total_bytes = 0;
        unsafe { operation.BytesReceived(&mut received_bytes) }?;
        unsafe { operation.TotalBytesToReceive(&mut total_bytes) }?;
        Ok(Download {
            url: text(|value| unsafe { operation.Uri(value) })?,
            path: PathBuf::from(text(|value| unsafe { operation.ResultFilePath(value) })?),
            received_bytes: u64::try_from(received_bytes).unwrap_or_default(),
            total_bytes: u64::try_from(total_bytes).ok(),
            state: state(operation)?,
            pausable: true,
            speed: None,
            note: None,
        })
    }

    pub fn pause(&self) {
        report(unsafe { self.0.Pause() });
    }

    pub fn resume(&self) {
        report(unsafe { self.0.Resume() });
    }

    pub fn cancel(&self) {
        report(unsafe { self.0.Cancel() });
    }

    pub fn watch(&self, changed: impl Fn() + Clone + 'static) -> Result<()> {
        let received = changed.clone();
        let progressed = BytesReceivedChangedEventHandler::create(Box::new(move |_, _| {
            received();
            Ok(())
        }));
        let finished = StateChangedEventHandler::create(Box::new(move |_, _| {
            changed();
            Ok(())
        }));
        unsafe { self.0.add_BytesReceivedChanged(&progressed, &mut 0) }?;
        unsafe { self.0.add_StateChanged(&finished, &mut 0) }
    }
}

fn choose_download(
    args: &ICoreWebView2DownloadStartingEventArgs,
    path: Option<&Path>,
) -> Result<()> {
    match path {
        Some(path) => unsafe { args.SetResultFilePath(&path_text(path)) },
        None => unsafe { args.SetCancel(true) },
    }
}

fn choose_save_as(
    args: &ICoreWebView2SaveAsUIShowingEventArgs,
    chosen: &Chosen,
    path: Option<PathBuf>,
) -> Result<()> {
    match path {
        Some(path) => {
            unsafe { args.SetSaveAsFilePath(&path_text(&path)) }?;
            unsafe { args.SetAllowReplace(true) }?;
            chosen.remember(path);
            Ok(())
        }
        None => unsafe { args.SetCancel(true) },
    }
}

impl Prompt {
    pub fn suggested(&self) -> PathBuf {
        let path = match &self.request {
            Request::Download { args, .. } => text(|value| unsafe { args.ResultFilePath(value) }),
            Request::SaveAs { args, .. } => text(|value| unsafe { args.SaveAsFilePath(value) }),
        };
        PathBuf::from(path.unwrap_or_default())
    }

    pub fn finish(self, path: Option<PathBuf>) -> Option<Transfer> {
        let transfer = match self.request {
            Request::Download { args, transfer } => {
                report(choose_download(&args, path.as_deref()));
                path.map(|_| transfer)
            }
            Request::SaveAs { args, chosen } => {
                report(choose_save_as(&args, &chosen, path));
                None
            }
        };
        report(unsafe { self.deferral.Complete() });
        transfer
    }
}

fn starting(
    runtime: &Runtime,
    chosen: &Chosen,
    args: ICoreWebView2DownloadStartingEventArgs,
) -> Result<Signal> {
    unsafe { args.SetHandled(true) }?;
    let transfer = Transfer(unsafe { args.DownloadOperation() }?);
    let path = PathBuf::from(text(|value| unsafe { args.ResultFilePath(value) })?);
    if chosen.claim(&path) || !runtime.asks_where_to_save() {
        return Ok(Signal::Download(transfer));
    }
    let deferral = unsafe { args.GetDeferral() }?;
    Ok(Signal::Prompt(Prompt {
        request: Request::Download { args, transfer },
        deferral,
    }))
}

fn saving_as(chosen: &Chosen, args: ICoreWebView2SaveAsUIShowingEventArgs) -> Result<Signal> {
    unsafe { args.SetSuppressDefaultDialog(true) }?;
    let deferral = unsafe { args.GetDeferral() }?;
    Ok(Signal::Prompt(Prompt {
        request: Request::SaveAs {
            args,
            chosen: chosen.clone(),
        },
        deferral,
    }))
}

fn wire_downloads(
    webview: &ICoreWebView2,
    runtime: Rc<Runtime>,
    chosen: Chosen,
    signals: Signals,
) -> Result<()> {
    let handler = DownloadStartingEventHandler::create(Box::new(move |_, args| {
        if let Some(args) = args {
            signals.send(starting(&runtime, &chosen, args)?);
        }
        Ok(())
    }));
    let webview = webview.cast::<ICoreWebView2_4>()?;
    unsafe { webview.add_DownloadStarting(&handler, &mut 0) }
}

fn wire_save_as(webview: &ICoreWebView2, chosen: Chosen, signals: Signals) -> Result<()> {
    let handler = SaveAsUIShowingEventHandler::create(Box::new(move |_, args| {
        if let Some(args) = args {
            signals.send(saving_as(&chosen, args)?);
        }
        Ok(())
    }));
    let webview = webview.cast::<ICoreWebView2_25>()?;
    unsafe { webview.add_SaveAsUIShowing(&handler, &mut 0) }
}

pub fn wire(webview: &ICoreWebView2, runtime: Rc<Runtime>, signals: Signals) -> Result<()> {
    let chosen = Chosen::default();
    wire_save_as(webview, chosen.clone(), signals.clone())?;
    wire_downloads(webview, runtime, chosen, signals)
}
