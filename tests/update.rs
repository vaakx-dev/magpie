use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use magpie::update::{self, Handoff};
use magpie_engine::update::Package;
use tempfile::TempDir;

const NEW_BUILD: &str = r"C:\Windows\System32\hostname.exe";
const OLD_BUILD: &str = r"C:\Windows\System32\whoami.exe";

struct Layout {
    _root: TempDir,
    downloads: PathBuf,
    update: PathBuf,
    target: PathBuf,
}

fn layout() -> Layout {
    let root = tempfile::tempdir().unwrap();
    let downloads = root.path().join("magpie update");
    let install = root.path().join("My Apps").join("Magpie Browser");
    fs::create_dir_all(&downloads).unwrap();
    fs::create_dir_all(&install).unwrap();
    let target = install.join("magpie.exe");
    fs::copy(OLD_BUILD, &target).unwrap();
    Layout {
        update: downloads.join("Magpie 0.2.0 windows-x64.exe"),
        downloads,
        target,
        _root: root,
    }
}

fn exited_process_id() -> u32 {
    let mut child = Command::new("cmd").args(["/c", "exit"]).spawn().unwrap();
    child.wait().unwrap();
    child.id()
}

fn apply_portable(update: &Path, target: &Path) {
    let handoff = Handoff {
        process_id: exited_process_id(),
        package: Package::Portable,
        update,
        target,
    };
    let status = update::command(&handoff).unwrap().status().unwrap();
    assert!(status.success());
}

#[test]
fn portable_update_replaces_the_running_exe_in_a_folder_with_spaces() {
    let layout = layout();
    fs::copy(NEW_BUILD, &layout.update).unwrap();

    apply_portable(&layout.update, &layout.target);

    assert_eq!(
        fs::read(&layout.target).unwrap(),
        fs::read(NEW_BUILD).unwrap()
    );
    assert_eq!(update::failure(&layout.downloads).unwrap(), None);
}

#[test]
fn failed_portable_update_keeps_the_old_exe_and_reports_why_on_the_next_launch() {
    let layout = layout();

    apply_portable(&layout.update, &layout.target);

    assert_eq!(
        fs::read(&layout.target).unwrap(),
        fs::read(OLD_BUILD).unwrap()
    );
    let failure = update::failure(&layout.downloads).unwrap().unwrap();
    assert!(
        failure.contains("Magpie 0.2.0 windows-x64.exe"),
        "{failure}"
    );
    update::clear(&layout.downloads).unwrap();
    assert!(!layout.downloads.exists());
}
