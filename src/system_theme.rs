use std::{ffi::c_void, thread};

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use windows::{
    Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_NOTIFY, KEY_QUERY_VALUE, REG_NOTIFY_CHANGE_LAST_SET,
        RRF_RT_REG_DWORD, RegCloseKey, RegGetValueW, RegNotifyChangeKeyValue, RegOpenKeyExW,
    },
    core::{PCWSTR, w},
};

use crate::config::Scheme;

const PERSONALIZE: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
const APPS_USE_LIGHT_THEME: PCWSTR = w!("AppsUseLightTheme");
const DWORD_BYTES: u32 = 4;

pub fn scheme() -> Scheme {
    let mut light = 1u32;
    let mut size = DWORD_BYTES;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PERSONALIZE,
            APPS_USE_LIGHT_THEME,
            RRF_RT_REG_DWORD,
            None,
            Some((&raw mut light).cast::<c_void>()),
            Some(&mut size),
        )
    };
    match (status.is_ok(), light) {
        (true, 0) => Scheme::Dark,
        _ => Scheme::Light,
    }
}

fn open() -> Option<HKEY> {
    let mut key = HKEY::default();
    let access = KEY_NOTIFY | KEY_QUERY_VALUE;
    let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, PERSONALIZE, None, access, &mut key) };
    status.is_ok().then_some(key)
}

fn changed(key: HKEY) -> bool {
    unsafe { RegNotifyChangeKeyValue(key, false, REG_NOTIFY_CHANGE_LAST_SET, None, false) }.is_ok()
}

fn watch(sender: &UnboundedSender<Scheme>) {
    let Some(key) = open() else {
        return;
    };
    while changed(key) && sender.unbounded_send(scheme()).is_ok() {}
    let _closed = unsafe { RegCloseKey(key) };
}

pub fn changes() -> UnboundedReceiver<Scheme> {
    let (sender, receiver) = unbounded();
    thread::spawn(move || watch(&sender));
    receiver
}
