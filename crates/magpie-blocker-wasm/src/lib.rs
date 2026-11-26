use std::cell::RefCell;

use adblock::{Engine, request::Request};

const FIELD_SEPARATOR: char = '\n';

thread_local! {
    static ENGINE: RefCell<Engine> = RefCell::new(Engine::default());
    static EXCHANGE: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn blocks(fields: &str) -> bool {
    let mut fields = fields.split(FIELD_SEPARATOR);
    let (Some(url), Some(source), Some(kind), Some(method)) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return false;
    };
    let Ok(request) = Request::new(url, source, kind, method) else {
        return false;
    };
    ENGINE.with_borrow(|engine| engine.check_network_request(&request).should_block())
}

fn stylesheet(page: &str) -> String {
    let resources = ENGINE.with_borrow(|engine| engine.url_cosmetic_resources(page));
    resources
        .hide_selectors
        .iter()
        .map(|selector| format!("{selector}{{display:none !important}}\n"))
        .collect()
}

#[unsafe(no_mangle)]
pub extern "C" fn exchange(length: usize) -> *mut u8 {
    EXCHANGE.with_borrow_mut(|bytes| {
        bytes.resize(length, 0);
        bytes.as_mut_ptr()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn load_snapshot(length: usize) -> bool {
    let snapshot = EXCHANGE.take();
    ENGINE.with_borrow_mut(|engine| engine.deserialize(&snapshot[..length]).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn should_block(length: usize) -> bool {
    EXCHANGE.with_borrow(|bytes| str::from_utf8(&bytes[..length]).is_ok_and(blocks))
}

#[unsafe(no_mangle)]
pub extern "C" fn cosmetic_css(length: usize) -> usize {
    let css = EXCHANGE.with_borrow(|bytes| {
        str::from_utf8(&bytes[..length])
            .map(stylesheet)
            .unwrap_or_default()
    });
    let length = css.len();
    EXCHANGE.set(css.into_bytes());
    length
}
