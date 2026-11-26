use std::{
    fs, io,
    path::Path,
    thread,
    time::{Duration, SystemTime},
};

use ureq::Agent;

use crate::file;

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
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60);
const MARKUP_START: char = '<';

pub fn filter_list(text: &str) -> bool {
    let start = text.trim_start_matches('\u{feff}').trim_start();
    !start.is_empty() && !start.starts_with(MARKUP_START)
}

pub(super) fn subscriptions(folder: &Path) -> Vec<String> {
    let agent = agent();
    thread::scope(|scope| {
        let loading: Vec<_> = SUBSCRIPTIONS
            .iter()
            .map(|(file, address)| {
                let path = folder.join(file);
                let agent = &agent;
                scope.spawn(move || subscription(agent, &path, address))
            })
            .collect();
        loading
            .into_iter()
            .filter_map(|load| load.join().ok().flatten())
            .collect()
    })
}

fn agent() -> Agent {
    Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(DOWNLOAD_TIMEOUT))
        .build()
        .new_agent()
}

fn subscription(agent: &Agent, path: &Path, address: &str) -> Option<String> {
    let cached = fs::read_to_string(path)
        .ok()
        .filter(|list| filter_list(list));
    if cached.is_some() && fresh(path) {
        return cached;
    }
    download(agent, address)
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

fn download(agent: &Agent, address: &str) -> Option<String> {
    agent
        .get(address)
        .call()
        .ok()?
        .body_mut()
        .with_config()
        .limit(MAX_LIST_BYTES)
        .read_to_string()
        .ok()
        .filter(|list| filter_list(list))
}

fn write(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    file::replace(path, contents)
}

pub(super) fn save(path: &Path, contents: impl AsRef<[u8]>) {
    if let Err(error) = write(path, contents.as_ref()) {
        eprintln!(
            "magpie: could not cache filters {}: {error}",
            path.display()
        );
    }
}
