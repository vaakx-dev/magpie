pub mod address;
pub mod autocomplete;
pub mod config;
pub mod darkmode;
pub mod download;
pub mod find;
pub mod import;
pub mod internal;
pub mod locale;
pub mod pages;
pub mod reader;
pub mod role;
pub mod storage;
pub mod stream_download;
pub mod system_theme;
pub mod ui;
pub mod update;
pub mod webview;
pub mod zoom;

const DIRECT_COMPOSITION_SWITCH: &str = "GPUI_DISABLE_DIRECT_COMPOSITION";

fn draw_beneath_child_windows() {
    unsafe { std::env::set_var(DIRECT_COMPOSITION_SWITCH, "1") };
}

pub fn run() {
    draw_beneath_child_windows();
    let addresses: Vec<String> = std::env::args().skip(1).collect();
    gpui::Application::new()
        .with_assets(ui::Assets)
        .run(move |cx| ui::open(&addresses, cx));
}
