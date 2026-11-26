use std::{collections::HashSet, future::Future, mem, ptr};

use futures::channel::oneshot;
use webview2_com::{
    GetProcessExtendedInfosCompletedHandler,
    Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_PROCESS_KIND, COREWEBVIEW2_PROCESS_KIND_RENDERER, ICoreWebView2,
        ICoreWebView2_20, ICoreWebView2Environment13, ICoreWebView2FrameInfo2,
        ICoreWebView2FrameInfoCollection, ICoreWebView2ProcessExtendedInfo,
        ICoreWebView2ProcessExtendedInfoCollection,
    },
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, E_ABORT, E_POINTER},
        System::{
            ProcessStatus::{
                GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
            },
            Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
        },
    },
    core::{BOOL, Error, Interface, Result},
};

const BYTES_PER_MEGABYTE: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameId(u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Memory {
    bytes: u64,
}

impl Memory {
    pub fn megabytes(self) -> u64 {
        self.bytes / BYTES_PER_MEGABYTE
    }
}

pub(super) fn main_frame(webview: &ICoreWebView2) -> Option<FrameId> {
    let mut id = 0;
    let webview = webview.cast::<ICoreWebView2_20>().ok()?;
    unsafe { webview.FrameId(&mut id) }.ok()?;
    (id != 0).then_some(FrameId(id))
}

fn frame_ids(frames: &ICoreWebView2FrameInfoCollection) -> Result<Vec<FrameId>> {
    let iterator = unsafe { frames.GetIterator() }?;
    let mut ids = Vec::new();
    let mut more = BOOL::default();
    unsafe { iterator.HasCurrent(&mut more) }?;
    while more.as_bool() {
        let mut id = 0;
        unsafe {
            iterator
                .GetCurrent()?
                .cast::<ICoreWebView2FrameInfo2>()?
                .FrameId(&mut id)
        }?;
        ids.push(FrameId(id));
        unsafe { iterator.MoveNext(&mut more) }?;
    }
    Ok(ids)
}

fn renderer(process: &ICoreWebView2ProcessExtendedInfo) -> Result<Option<u32>> {
    let info = unsafe { process.ProcessInfo() }?;
    let mut kind = COREWEBVIEW2_PROCESS_KIND::default();
    let mut id = 0;
    unsafe { info.Kind(&mut kind) }?;
    unsafe { info.ProcessId(&mut id) }?;
    Ok((kind == COREWEBVIEW2_PROCESS_KIND_RENDERER).then_some(id as u32))
}

fn hosts(process: &ICoreWebView2ProcessExtendedInfo, frames: &HashSet<FrameId>) -> Result<bool> {
    let associated = unsafe { process.AssociatedFrameInfos() }?;
    Ok(frame_ids(&associated)?.iter().any(|id| frames.contains(id)))
}

fn hosting_renderer(
    process: &ICoreWebView2ProcessExtendedInfo,
    frames: &HashSet<FrameId>,
) -> Result<Option<u32>> {
    match renderer(process)? {
        Some(id) if hosts(process, frames)? => Ok(Some(id)),
        _ => Ok(None),
    }
}

fn renderers(
    processes: &ICoreWebView2ProcessExtendedInfoCollection,
    frames: &HashSet<FrameId>,
) -> Result<Vec<u32>> {
    let mut count = 0;
    unsafe { processes.Count(&mut count) }?;
    let hosting = (0..count).map(|index| {
        let process = unsafe { processes.GetValueAtIndex(index) }?;
        hosting_renderer(&process, frames)
    });
    hosting
        .filter_map(Result::transpose)
        .collect::<Result<HashSet<u32>>>()
        .map(|ids| ids.into_iter().collect())
}

fn private_bytes(process_id: u32) -> Result<u64> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }?;
    let mut counters = PROCESS_MEMORY_COUNTERS_EX::default();
    let size = mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    let counted = unsafe {
        GetProcessMemoryInfo(
            process,
            ptr::from_mut(&mut counters).cast::<PROCESS_MEMORY_COUNTERS>(),
            size,
        )
    };
    let _closed = unsafe { CloseHandle(process) };
    counted.map(|()| counters.PrivateUsage as u64)
}

fn usage(
    processes: Option<ICoreWebView2ProcessExtendedInfoCollection>,
    frames: &HashSet<FrameId>,
) -> Result<Memory> {
    let processes = processes.ok_or_else(|| Error::from(E_POINTER))?;
    let bytes = renderers(&processes, frames)?
        .into_iter()
        .map(private_bytes)
        .sum::<Result<u64>>()?;
    Ok(Memory { bytes })
}

pub(super) fn measure(
    environment: &ICoreWebView2Environment13,
    frames: Vec<FrameId>,
) -> impl Future<Output = Result<Memory>> + 'static {
    let frames: HashSet<FrameId> = frames.into_iter().collect();
    let (sender, receiver) = oneshot::channel();
    let completed =
        GetProcessExtendedInfosCompletedHandler::create(Box::new(move |result, processes| {
            sender
                .send(result.and_then(|()| usage(processes, &frames)))
                .ok();
            Ok(())
        }));
    let started = unsafe { environment.GetProcessExtendedInfos(&completed) };
    async move {
        started?;
        receiver.await.map_err(|_| Error::from(E_ABORT))?
    }
}
