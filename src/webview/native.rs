use std::future::Future;

use futures::channel::oneshot;
use url::Url;
use webview2_com::{
    CallDevToolsProtocolMethodCompletedHandler,
    Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC, COREWEBVIEW2_PRINT_DIALOG_KIND_BROWSER,
        ICoreWebView2, ICoreWebView2_3, ICoreWebView2_8, ICoreWebView2_16, ICoreWebView2_25,
        ICoreWebView2Controller, ICoreWebView2Controller2,
    },
    ShowSaveAsUICompletedHandler, TrySuspendCompletedHandler,
};
use windows::{
    Win32::{
        Foundation::{E_ABORT, HWND, RECT},
        Graphics::Gdi::{
            CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject, HRGN, InvalidateRect,
            RGN_AND, RGN_DIFF, SetWindowRgn,
        },
        UI::{
            Input::KeyboardAndMouse::GetFocus,
            WindowsAndMessaging::{
                DestroyWindow, IsChild, SHOW_WINDOW_CMD, SW_HIDE, SW_SHOWNA, SWP_NOACTIVATE,
                SWP_NOZORDER, SetWindowPos, ShowWindow,
            },
        },
    },
    core::{Error, HSTRING, Interface, Result},
};

use super::{backdrop::Backdrop, signal::Requested};
use crate::storage::permissions::Origin;

const CLEAR_DATA_METHOD: &str = "Storage.clearDataForOrigin";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rounded {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub corner: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub frame: Rounded,
    pub visible: RECT,
    pub cutouts: Vec<Rounded>,
}

pub struct Container(pub HWND);

pub struct Native {
    pub controller: ICoreWebView2Controller,
    pub webview: ICoreWebView2,
    pub container: Container,
    pub requested: Requested,
}

pub fn report(result: Result<()>) {
    if let Err(error) = result {
        eprintln!("magpie: WebView2 call failed: {error}");
    }
}

impl Rounded {
    fn region(&self) -> HRGN {
        let Rounded {
            x,
            y,
            width,
            height,
            corner,
        } = *self;
        unsafe { CreateRoundRectRgn(x, y, x + width + 1, y + height + 1, corner, corner) }
    }
}

fn subtract(region: HRGN, cutout: Rounded) {
    let hole = cutout.region();
    unsafe { CombineRgn(Some(region), Some(region), Some(hole), RGN_DIFF) };
    let _deleted = unsafe { DeleteObject(hole.into()) };
}

fn clip(region: HRGN, visible: RECT) {
    let RECT {
        left,
        top,
        right,
        bottom,
    } = visible;
    let shown = unsafe { CreateRectRgn(left, top, right, bottom) };
    unsafe { CombineRgn(Some(region), Some(region), Some(shown), RGN_AND) };
    let _deleted = unsafe { DeleteObject(shown.into()) };
}

fn outline(placement: &Placement) -> HRGN {
    let outline = Rounded {
        x: 0,
        y: 0,
        ..placement.frame
    };
    let region = outline.region();
    clip(region, placement.visible);
    for cutout in &placement.cutouts {
        subtract(region, *cutout);
    }
    region
}

impl Native {
    pub fn navigate(&self, url: &Url) {
        *self.requested.borrow_mut() = Some(url.clone());
        report(unsafe { self.webview.Navigate(&HSTRING::from(url.as_str())) });
    }

    pub fn back(&self) {
        report(unsafe { self.webview.GoBack() });
    }

    pub fn forward(&self) {
        report(unsafe { self.webview.GoForward() });
    }

    pub fn reload(&self) {
        report(unsafe { self.webview.Reload() });
    }

    pub fn stop(&self) {
        report(unsafe { self.webview.Stop() });
    }

    pub fn print(&self) {
        let webview = self.webview.cast::<ICoreWebView2_16>();
        report(webview.and_then(|webview| unsafe {
            webview.ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_BROWSER)
        }));
    }

    pub fn save_page(&self) {
        let completed = ShowSaveAsUICompletedHandler::create(Box::new(|result, _| result));
        let webview = self.webview.cast::<ICoreWebView2_25>();
        report(webview.and_then(|webview| unsafe { webview.ShowSaveAsUI(&completed) }));
    }

    pub fn post(&self, json: &str) {
        report(unsafe { self.webview.PostWebMessageAsJson(&HSTRING::from(json)) });
    }

    pub fn place(&self, placement: &Placement) {
        let Rounded {
            x,
            y,
            width,
            height,
            ..
        } = placement.frame;
        let container = self.container.0;
        let flags = SWP_NOZORDER | SWP_NOACTIVATE;
        report(unsafe { SetWindowPos(container, None, x, y, width, height, flags) });
        unsafe { SetWindowRgn(container, Some(outline(placement)), true) };
        let bounds = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        report(unsafe { self.controller.SetBounds(bounds) });
    }

    fn display(&self, command: SHOW_WINDOW_CMD) {
        let _was_visible = unsafe { ShowWindow(self.container.0, command) };
    }

    pub fn show(&self) {
        self.display(SW_SHOWNA);
        report(unsafe { self.controller.SetIsVisible(true) });
    }

    pub fn hide(&self) {
        report(unsafe { self.controller.SetIsVisible(false) });
        self.display(SW_HIDE);
    }

    pub fn paint(&self, backdrop: Backdrop) {
        let container = self.container.0;
        backdrop.keep_for(container);
        let _invalidated = unsafe { InvalidateRect(Some(container), None, true) };
        let controller = self.controller.cast::<ICoreWebView2Controller2>();
        report(controller.and_then(|controller| unsafe {
            controller.SetDefaultBackgroundColor(backdrop.page_color())
        }));
    }

    pub fn focus(&self) {
        report(unsafe {
            self.controller
                .MoveFocus(COREWEBVIEW2_MOVE_FOCUS_REASON_PROGRAMMATIC)
        });
    }

    fn set_muted(&self, muted: bool) {
        let webview = self.webview.cast::<ICoreWebView2_8>();
        report(webview.and_then(|webview| unsafe { webview.SetIsMuted(muted) }));
    }

    pub fn mute(&self) {
        self.set_muted(true);
    }

    pub fn unmute(&self) {
        self.set_muted(false);
    }

    pub fn suspend(&self) -> impl Future<Output = Result<bool>> + 'static {
        let (sender, receiver) = oneshot::channel();
        let completed = TrySuspendCompletedHandler::create(Box::new(move |result, suspended| {
            sender.send(result.map(|()| suspended)).ok();
            Ok(())
        }));
        let webview = self.webview.cast::<ICoreWebView2_3>();
        let started = webview.and_then(|webview| unsafe { webview.TrySuspend(&completed) });
        async move {
            started?;
            receiver.await.map_err(|_| Error::from(E_ABORT))?
        }
    }

    pub fn clear_site_data(&self, origin: &Origin) -> impl Future<Output = Result<()>> + use<> {
        let (sender, receiver) = oneshot::channel();
        let completed =
            CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result, _| {
                sender.send(result).ok();
                Ok(())
            }));
        let parameters = serde_json::json!({ "origin": origin, "storageTypes": "all" });
        let started = unsafe {
            self.webview.CallDevToolsProtocolMethod(
                &HSTRING::from(CLEAR_DATA_METHOD),
                &HSTRING::from(parameters.to_string()),
                &completed,
            )
        };
        async move {
            started?;
            receiver.await.map_err(|_| Error::from(E_ABORT))?
        }
    }

    pub fn resume(&self) {
        let webview = self.webview.cast::<ICoreWebView2_3>();
        report(webview.and_then(|webview| unsafe { webview.Resume() }));
    }

    pub fn focused(&self) -> bool {
        let focus = unsafe { GetFocus() };
        let container = self.container.0;
        focus == container || unsafe { IsChild(container, focus) }.as_bool()
    }
}

impl Drop for Native {
    fn drop(&mut self) {
        report(unsafe { self.controller.Close() });
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        report(unsafe { DestroyWindow(self.0) });
    }
}
