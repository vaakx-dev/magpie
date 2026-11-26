use std::{
    fs::{self, File},
    time::{Duration, SystemTime},
};

use magpie::storage::Store;

const OPEN: &str = "https://example.com/open";
const RECENT: &str = "https://example.com/recent";
const OLD: &str = "https://example.com/old";
const TWO_DAYS: Duration = Duration::from_secs(2 * 24 * 60 * 60);

fn age_all_but(folder: &std::path::Path, keep_fresh: &[u8]) {
    let old = SystemTime::now() - TWO_DAYS;
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if fs::read(&path).unwrap() != keep_fresh {
            File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(old)
                .unwrap();
        }
    }
}

#[test]
fn sweeping_keeps_open_and_recent_previews_and_drops_old_orphans() {
    let home = tempfile::tempdir().unwrap();
    let cache = Store::open(home.path()).unwrap().thumbnails();
    cache.write(OPEN, b"open").unwrap();
    cache.write(RECENT, b"recent").unwrap();
    cache.write(OLD, b"old").unwrap();
    age_all_but(&home.path().join("thumbs"), b"recent");

    cache.sweep([OPEN]).unwrap();

    assert_eq!(cache.read(OPEN).as_deref(), Some(&b"open"[..]));
    assert_eq!(cache.read(RECENT).as_deref(), Some(&b"recent"[..]));
    assert_eq!(cache.read(OLD), None);
}

#[test]
fn clearing_browsing_data_removes_every_preview_from_disk() {
    let home = tempfile::tempdir().unwrap();
    let store = Store::open(home.path()).unwrap();
    let cache = store.thumbnails();
    cache.write(OPEN, b"bank statement").unwrap();

    store.clear_browsing_data().unwrap();

    assert_eq!(cache.read(OPEN), None);
    assert!(!home.path().join("thumbs").exists());
    store.clear_browsing_data().unwrap();
}

#[test]
fn private_windows_leave_no_previews_on_disk() {
    let home = tempfile::tempdir().unwrap();
    let cache = Store::open(home.path())
        .unwrap()
        .new_private_window()
        .thumbnails();

    cache.write(OPEN, b"open").unwrap();

    assert_eq!(cache.read(OPEN), None);
}
