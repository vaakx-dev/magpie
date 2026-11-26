use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_COLOR;
use windows::Win32::{
    Foundation::{COLORREF, HWND, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{DC_BRUSH, FillRect, GetStockObject, HBRUSH, HDC, SetDCBrushColor},
    UI::WindowsAndMessaging::{GWLP_USERDATA, GetClientRect, GetWindowLongPtrW, SetWindowLongPtrW},
};

const OPAQUE: u8 = 0xff;
const ERASED: LRESULT = LRESULT(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backdrop {
    red: u8,
    green: u8,
    blue: u8,
}

impl Backdrop {
    pub const fn from_rgba(rgba: u32) -> Self {
        let [red, green, blue, _] = rgba.to_be_bytes();
        Self { red, green, blue }
    }

    pub(super) fn page_color(self) -> COREWEBVIEW2_COLOR {
        COREWEBVIEW2_COLOR {
            A: OPAQUE,
            R: self.red,
            G: self.green,
            B: self.blue,
        }
    }

    fn colorref(self) -> COLORREF {
        COLORREF(u32::from_le_bytes([self.red, self.green, self.blue, 0]))
    }

    pub(super) fn keep_for(self, window: HWND) {
        unsafe { SetWindowLongPtrW(window, GWLP_USERDATA, self.colorref().0 as isize) };
    }
}

fn kept_color(window: HWND) -> COLORREF {
    COLORREF(unsafe { GetWindowLongPtrW(window, GWLP_USERDATA) } as u32)
}

pub(super) fn erase(window: HWND, context: WPARAM) -> LRESULT {
    let context = HDC(context.0 as _);
    let mut area = RECT::default();
    if unsafe { GetClientRect(window, &mut area) }.is_ok() {
        unsafe { SetDCBrushColor(context, kept_color(window)) };
        unsafe { FillRect(context, &area, HBRUSH(GetStockObject(DC_BRUSH).0)) };
    }
    ERASED
}
