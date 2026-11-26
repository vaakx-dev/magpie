use gpui::{App, DisplayId};
use windows::Win32::{
    Foundation::{LPARAM, RECT},
    Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
    UI::WindowsAndMessaging::MONITORINFOF_PRIMARY,
};
use windows::core::BOOL;

use crate::placement::{Rect, Screen};

unsafe extern "system" fn collect(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    monitors: LPARAM,
) -> BOOL {
    let monitors = unsafe { &mut *(monitors.0 as *mut Vec<HMONITOR>) };
    monitors.push(monitor);
    BOOL::from(true)
}

fn monitors() -> Vec<HMONITOR> {
    let mut monitors = Vec::new();
    let collected = LPARAM(&mut monitors as *mut Vec<HMONITOR> as isize);
    match unsafe { EnumDisplayMonitors(None, None, Some(collect), collected) }.as_bool() {
        true => monitors,
        false => Vec::new(),
    }
}

fn info(monitor: HMONITOR) -> Option<MONITORINFO> {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..MONITORINFO::default()
    };
    unsafe { GetMonitorInfoW(monitor, &mut info) }
        .as_bool()
        .then_some(info)
}

fn logical(rect: RECT, scale: f32) -> Rect {
    Rect {
        x: rect.left as f32 / scale,
        y: rect.top as f32 / scale,
        width: (rect.right - rect.left) as f32 / scale,
        height: (rect.bottom - rect.top) as f32 / scale,
    }
}

fn screen(info: &MONITORINFO, logical_width_px: f32) -> Screen {
    let physical_width_px = (info.rcMonitor.right - info.rcMonitor.left) as f32;
    let scale = physical_width_px / logical_width_px;
    Screen {
        work_area: logical(info.rcWork, scale),
        primary: info.dwFlags & MONITORINFOF_PRIMARY != 0,
    }
}

pub fn screens(cx: &App) -> Vec<(DisplayId, Screen)> {
    cx.displays()
        .into_iter()
        .zip(monitors())
        .filter_map(|(display, monitor)| {
            let width_px = f32::from(display.bounds().size.width);
            Some((display.id(), screen(&info(monitor)?, width_px)))
        })
        .collect()
}
