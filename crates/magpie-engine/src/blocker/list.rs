use std::{
    fs,
    path::Path,
    thread,
    time::{Duration, SystemTime},
};

const SUBSCRIPTIONS: [(&str, &str); 4] = [
    ("easylist.txt", "https://easylist.to/easylist/easylist.txt"),
    (
        "easyprivacy.txt",
        "https://easylist.to/easylist/easyprivacy.txt",
    ),
    (
        "ublock-filters.txt",
        "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt",
    ),
    (
        "ublock-quick-fixes.txt",
        "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/quick-fixes.txt",
    ),
];
const FRESH_FOR: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_LIST_BYTES: u64 = 16 * 1024 * 1024;

pub fn subscriptions(folder: &Path) -> Vec<String> {
    thread::scope(|scope| {
        let loading: Vec<_> = SUBSCRIPTIONS
            .iter()
            .map(|(file, address)| {
                let path = folder.join(file);
                scope.spawn(move || subscription(&path, address))
            })
            .collect();
        loading
            .into_iter()
            .filter_map(|load| load.join().ok().flatten())
            .collect()
    })
}

fn subscription(path: &Path, address: &str) -> Option<String> {
    let cached = fs::read_to_string(path).ok();
    if cached.is_some() && fresh(path) {
        return cached;
    }
    download(address)
        .inspect(|list| save(path, list))
        .or(cached)
}

pub(super) fn fresh(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age < FRESH_FOR)
}

fn download(address: &str) -> Option<String> {
    ureq::get(address)
        .call()
        .ok()?
        .body_mut()
        .with_config()
        .limit(MAX_LIST_BYTES)
        .read_to_string()
        .ok()
}

pub(super) fn save(path: &Path, contents: impl AsRef<[u8]>) {
    let written = path
        .parent()
        .map(fs::create_dir_all)
        .transpose()
        .and_then(|_| fs::write(path, contents));
    if let Err(error) = written {
        eprintln!(
            "magpie: could not cache filters {}: {error}",
            path.display()
        );
    }
}
