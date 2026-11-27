use std::{mem::size_of_val, ptr, time::Duration};

use gpui::App;
use windows::{
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW,
                SetClipboardData,
            },
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
        },
    },
    core::{Error, PCWSTR, Result, w},
};
use zeroize::Zeroizing;

const CLEAR_AFTER: Duration = Duration::from_secs(30);
const UNICODE_TEXT_FORMAT: u32 = 13;
const PRIVATE_MARKS: [PCWSTR; 3] = [
    w!("ExcludeClipboardContentFromMonitorProcessing"),
    w!("CanIncludeInClipboardHistory"),
    w!("CanUploadToCloudClipboard"),
];
const DENIED: u32 = 0;

struct Opened;

impl Opened {
    fn open() -> Result<Self> {
        unsafe { OpenClipboard(None) }?;
        Ok(Self)
    }
}

impl Drop for Opened {
    fn drop(&mut self) {
        unsafe { CloseClipboard() }.ok();
    }
}

fn global<T>(data: &[T]) -> Result<HGLOBAL> {
    let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, size_of_val(data)) }?;
    let target = unsafe { GlobalLock(memory) };
    if target.is_null() {
        let error = Error::from_thread();
        unsafe { GlobalFree(Some(memory)) }.ok();
        return Err(error);
    }
    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr(), target.cast(), data.len());
        GlobalUnlock(memory).ok();
    }
    Ok(memory)
}

fn set<T>(format: u32, data: &[T]) -> Result<()> {
    let memory = global(data)?;
    unsafe { SetClipboardData(format, Some(HANDLE(memory.0))) }
        .inspect_err(|_| {
            unsafe { GlobalFree(Some(memory)) }.ok();
        })
        .map(drop)
}

fn mark_private() -> Result<()> {
    PRIVATE_MARKS.into_iter().try_for_each(|mark| {
        let format = unsafe { RegisterClipboardFormatW(mark) };
        set(format, &[DENIED])
    })
}

fn write(secret: &str) -> Result<()> {
    let text = Zeroizing::new(secret.encode_utf16().chain([0]).collect::<Vec<u16>>());
    let _opened = Opened::open()?;
    unsafe { EmptyClipboard() }?;
    set(UNICODE_TEXT_FORMAT, &text)?;
    mark_private()
}

fn clear() -> Result<()> {
    let _opened = Opened::open()?;
    unsafe { EmptyClipboard() }
}

fn still_holds(secret: &str, cx: &App) -> bool {
    cx.read_from_clipboard()
        .and_then(|item| item.text())
        .map(Zeroizing::new)
        .is_some_and(|text| text.as_str() == secret)
}

fn clear_if_unchanged(secret: &str, cx: &App) {
    if !still_holds(secret, cx) {
        return;
    }
    if let Err(error) = clear() {
        eprintln!("magpie: could not clear the clipboard: {error}");
    }
}

pub fn copy_secret(secret: Zeroizing<String>, cx: &mut App) {
    if let Err(error) = write(&secret) {
        eprintln!("magpie: could not copy to the clipboard: {error}");
        return;
    }
    cx.spawn(async move |cx| {
        cx.background_executor().timer(CLEAR_AFTER).await;
        cx.update(|cx| clear_if_unchanged(&secret, cx)).ok();
    })
    .detach();
}
