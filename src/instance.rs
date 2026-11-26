use std::{ffi::c_void, slice, sync::Once};

use futures::{
    StreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded},
};
use windows::{
    Win32::{
        Foundation::{
            ERROR_ALREADY_EXISTS, ERROR_INVALID_DATA, ERROR_TIMEOUT, GetLastError, HANDLE, HWND,
            LPARAM, LRESULT, WAIT_OBJECT_0, WIN32_ERROR, WPARAM,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{CreateEventW, CreateMutexW, SetEvent, WaitForSingleObject},
        },
        UI::WindowsAndMessaging::{
            AllowSetForegroundWindow, CREATESTRUCTW, CreateWindowExW, DefWindowProcW,
            FindWindowExW, GWLP_USERDATA, GetWindowLongPtrW, GetWindowThreadProcessId,
            HWND_MESSAGE, RegisterClassW, SMTO_ABORTIFHUNG, SendMessageTimeoutW, SetWindowLongPtrW,
            WINDOW_EX_STYLE, WINDOW_STYLE, WM_COPYDATA, WM_NCCREATE, WNDCLASSW,
        },
    },
    core::{Error, HRESULT, Owned, PCWSTR, Result, w},
};

const LOCK: PCWSTR = w!("Local\\Magpie.Instance");
const READY: PCWSTR = w!("Local\\Magpie.Instance.Ready");
const LISTENER_CLASS: PCWSTR = w!("MagpieInstance");
const HANDOFF_TAG: usize = 0x4d41_4750;
const READY_TIMEOUT_MS: u32 = 10_000;
const SEND_TIMEOUT_MS: u32 = 5_000;
const ACCEPTED: usize = 1;

#[repr(C)]
struct CopyData {
    tag: usize,
    length: u32,
    bytes: *const c_void,
}

pub struct Instance {
    _lock: Owned<HANDLE>,
    ready: Owned<HANDLE>,
}

pub struct Running {
    ready: Owned<HANDLE>,
}

pub enum Claim {
    First(Instance),
    Running(Running),
}

pub struct Listener {
    _instance: Instance,
    addresses: UnboundedReceiver<Vec<String>>,
}

type Sender = UnboundedSender<Vec<String>>;

pub fn addresses(arguments: impl IntoIterator<Item = String>) -> Vec<String> {
    arguments
        .into_iter()
        .map(|argument| argument.trim().to_owned())
        .filter(|argument| !argument.is_empty())
        .collect()
}

pub fn encode(addresses: &[String]) -> Vec<u8> {
    serde_json::to_vec(addresses).expect("addresses serialize to JSON")
}

pub fn decode(bytes: &[u8]) -> Option<Vec<String>> {
    serde_json::from_slice(bytes).ok()
}

fn ready_event() -> Result<Owned<HANDLE>> {
    let event = unsafe { CreateEventW(None, true, false, READY) }?;
    Ok(unsafe { Owned::new(event) })
}

fn received(copied: &CopyData) -> Option<Vec<String>> {
    if copied.tag != HANDOFF_TAG || copied.bytes.is_null() {
        return None;
    }
    let bytes = unsafe { slice::from_raw_parts(copied.bytes.cast::<u8>(), copied.length as usize) };
    decode(bytes)
}

fn failure(error: WIN32_ERROR) -> Error {
    HRESULT::from_win32(error.0).into()
}

fn keep_sender(window: HWND, created: LPARAM) {
    let created = unsafe { &*(created.0 as *const CREATESTRUCTW) };
    unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, created.lpCreateParams as isize) };
}

fn accept(window: HWND, copied: LPARAM) -> LRESULT {
    let Some(addresses) = received(unsafe { &*(copied.0 as *const CopyData) }) else {
        return LRESULT(0);
    };
    let sender = unsafe { &*(GetWindowLongPtrW(window, GWLP_USERDATA) as *const Sender) };
    sender.unbounded_send(addresses).ok();
    LRESULT(ACCEPTED as isize)
}

unsafe extern "system" fn listener_procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            keep_sender(window, lparam);
            unsafe { DefWindowProcW(window, message, wparam, lparam) }
        }
        WM_COPYDATA => accept(window, lparam),
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn register_listener_class() -> Result<()> {
    static REGISTERED: Once = Once::new();
    let instance = unsafe { GetModuleHandleW(None) }?;
    REGISTERED.call_once(|| {
        let class = WNDCLASSW {
            lpfnWndProc: Some(listener_procedure),
            hInstance: instance.into(),
            lpszClassName: LISTENER_CLASS,
            ..WNDCLASSW::default()
        };
        unsafe { RegisterClassW(&class) };
    });
    Ok(())
}

fn listener_window(sender: Sender) -> Result<()> {
    register_listener_class()?;
    let instance = unsafe { GetModuleHandleW(None) }?;
    let sender = Box::into_raw(Box::new(sender));
    let created = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            LISTENER_CLASS,
            PCWSTR::null(),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance.into()),
            Some(sender.cast_const().cast()),
        )
    };
    if created.is_err() {
        drop(unsafe { Box::from_raw(sender) });
    }
    created.map(drop)
}

fn send(window: HWND, payload: &[u8]) -> Result<()> {
    let copied = CopyData {
        tag: HANDOFF_TAG,
        length: payload.len() as u32,
        bytes: payload.as_ptr().cast(),
    };
    let mut answer = 0;
    let sent = unsafe {
        SendMessageTimeoutW(
            window,
            WM_COPYDATA,
            WPARAM(0),
            LPARAM(&copied as *const CopyData as isize),
            SMTO_ABORTIFHUNG,
            SEND_TIMEOUT_MS,
            Some(&mut answer),
        )
    };
    match (sent.0, answer) {
        (0, _) => Err(Error::from_thread()),
        (_, ACCEPTED) => Ok(()),
        _ => Err(failure(ERROR_INVALID_DATA)),
    }
}

impl Claim {
    pub fn take() -> Result<Self> {
        let lock = unsafe { CreateMutexW(None, false, LOCK) }?;
        let already_running = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        let lock = unsafe { Owned::new(lock) };
        let ready = ready_event()?;
        Ok(match already_running {
            true => Self::Running(Running { ready }),
            false => Self::First(Instance { _lock: lock, ready }),
        })
    }
}

impl Instance {
    pub fn listen(self) -> Result<Listener> {
        let (sender, addresses) = unbounded();
        listener_window(sender)?;
        unsafe { SetEvent(*self.ready) }?;
        Ok(Listener {
            _instance: self,
            addresses,
        })
    }
}

impl Listener {
    pub async fn next(&mut self) -> Option<Vec<String>> {
        self.addresses.next().await
    }
}

impl Running {
    pub fn hand_off(&self, addresses: &[String]) -> Result<()> {
        if unsafe { WaitForSingleObject(*self.ready, READY_TIMEOUT_MS) } != WAIT_OBJECT_0 {
            return Err(failure(ERROR_TIMEOUT));
        }
        let window =
            unsafe { FindWindowExW(Some(HWND_MESSAGE), None, LISTENER_CLASS, PCWSTR::null()) }?;
        let mut process = 0;
        unsafe { GetWindowThreadProcessId(window, Some(&mut process)) };
        unsafe { AllowSetForegroundWindow(process) }.ok();
        send(window, &encode(addresses))
    }
}
