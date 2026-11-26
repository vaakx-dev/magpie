mod download;
mod install;
mod leftover;
mod phase;

use std::{env, path::PathBuf};

use magpie_engine::update::{Platform, Version};

pub use download::{Fault, download};
pub use install::{Handoff, command, install};
pub use leftover::{clear, failure};
pub use phase::Phase;

pub const CURRENT_VERSION: &str = match option_env!("MAGPIE_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};
pub const BUILD_COMMIT: Option<&str> = option_env!("MAGPIE_BUILD_SHA");
const FOLDER: &str = "magpie-update";
const SETUP_UNINSTALLER: &str = "unins000.exe";
const FAILURE_FILE: &str = "failed.txt";

pub fn current() -> Version {
    Version::parse(CURRENT_VERSION).expect("the package version is semantic")
}

fn installed_by_setup() -> bool {
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|folder| folder.join(SETUP_UNINSTALLER)))
        .is_some_and(|uninstaller| uninstaller.exists())
}

pub fn platform() -> Platform<'static> {
    match installed_by_setup() {
        true => Platform::WindowsInstalled,
        false => Platform::WindowsPortable,
    }
}

pub fn folder() -> PathBuf {
    env::temp_dir().join(FOLDER)
}
