use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use magpie::update::{self, Fault, Handoff};
use magpie_engine::update::{Download, Package};
use tempfile::TempDir;

const NEW_BUILD: &str = r"C:\Windows\System32\hostname.exe";
const OLD_BUILD: &str = r"C:\Windows\System32\whoami.exe";
const CHECKSUM: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

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

fn offered(url: &str, sha256: Option<&str>) -> Download {
    Download {
        name: "Magpie-9.9.9-setup.exe".to_owned(),
        url: url.to_owned(),
        size: 1,
        sha256: sha256.map(str::to_owned),
        package: Package::Installer,
    }
}

#[test]
fn updates_only_download_from_github_release_hosts() {
    let folder = TempDir::new().unwrap();
    let untrusted = [
        "https://github.com.example.net/vaakx-dev/magpie/releases/download/v9.9.9/setup.exe",
        "http://github.com/vaakx-dev/magpie/releases/download/v9.9.9/setup.exe",
        "https://github.com:8443/vaakx-dev/magpie/releases/download/v9.9.9/setup.exe",
        "https://user@objects.githubusercontent.com/setup.exe",
        "https://example.com/setup.exe",
    ];
    for url in untrusted {
        let fault = update::download(&offered(url, Some(CHECKSUM)), folder.path(), drop);
        assert!(matches!(fault, Err(Fault::Untrusted)), "{url} was trusted");
    }
}

#[test]
fn updates_without_a_checksum_are_refused() {
    let folder = TempDir::new().unwrap();
    let official = "https://github.com/vaakx-dev/magpie/releases/download/v9.9.9/setup.exe";
    let fault = update::download(&offered(official, None), folder.path(), drop);
    assert!(matches!(fault, Err(Fault::Unverified)));
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
}
