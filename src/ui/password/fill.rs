use gpui::{App, Entity};

use crate::webview::{
    WebView,
    autofill::{Fill, Purpose},
};
use magpie_engine::bitwarden::{Login, Page};

fn belongs(login: &Login, view: &WebView) -> bool {
    view.url()
        .and_then(|url| Page::parse(url.as_str()))
        .is_some_and(|page| login.matches(&page))
}

pub fn sign_in(view: &Entity<WebView>, login: &Login, cx: &mut App) {
    if !belongs(login, view.read(cx)) {
        return;
    }
    let fill = Fill {
        username: login.username.as_deref(),
        password: login.password.as_deref(),
        purpose: Purpose::SignIn,
    };
    view.update(cx, |view, cx| view.autofill(&fill, cx));
}

pub fn new_password(view: &Entity<WebView>, password: &str, cx: &mut App) {
    let fill = Fill {
        username: None,
        password: Some(password),
        purpose: Purpose::NewPassword,
    };
    view.update(cx, |view, cx| view.autofill(&fill, cx));
}
