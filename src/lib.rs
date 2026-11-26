pub mod address;
pub mod autocomplete;
pub mod backup;
pub mod config;
pub mod darkmode;
pub mod download;
pub mod find;
pub mod import;
pub mod instance;
pub mod internal;
pub mod locale;
pub mod pages;
pub mod placement;
pub mod reader;
pub mod role;
pub mod storage;
pub mod stream_download;
pub mod system_theme;
pub mod ui;
pub mod update;
pub mod vault_key;
pub mod webview;
pub mod zoom;

use std::ops::ControlFlow;

use instance::{Claim, Instance};

const DIRECT_COMPOSITION_SWITCH: &str = "GPUI_DISABLE_DIRECT_COMPOSITION";

fn draw_beneath_child_windows() {
    unsafe { std::env::set_var(DIRECT_COMPOSITION_SWITCH, "1") };
}

fn claim(addresses: &[String]) -> ControlFlow<(), Option<Instance>> {
    let running = match Claim::take() {
        Ok(Claim::First(instance)) => return ControlFlow::Continue(Some(instance)),
        Ok(Claim::Running(running)) => running,
        Err(error) => {
            eprintln!("magpie: could not check for a running Magpie: {error}");
            return ControlFlow::Continue(None);
        }
    };
    match running.hand_off(addresses) {
        Ok(()) => ControlFlow::Break(()),
        Err(error) => {
            eprintln!("magpie: could not reach the running Magpie: {error}");
            ControlFlow::Continue(None)
        }
    }
}

pub fn run() {
    let addresses = instance::addresses(std::env::args().skip(1));
    let ControlFlow::Continue(instance) = claim(&addresses) else {
        return;
    };
    draw_beneath_child_windows();
    gpui::Application::new()
        .with_assets(ui::Assets)
        .run(move |cx| ui::open(&addresses, instance, cx));
}
