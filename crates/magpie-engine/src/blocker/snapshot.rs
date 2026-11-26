use std::{fs, path::Path};

use adblock::Engine;

use super::{
    BASELINE, engine,
    list::{fresh, save, subscriptions},
};

const FILE: &str = "engine.dat";

fn serialized(lists: impl IntoIterator<Item = String>) -> Vec<u8> {
    engine(lists).serialize()
}

fn readable(path: &Path) -> bool {
    fs::read(path).is_ok_and(|snapshot| Engine::default().deserialize(&snapshot).is_ok())
}

pub fn cached(folder: &Path) -> Option<Vec<u8>> {
    fs::read(folder.join(FILE)).ok()
}

pub fn refreshed(folder: &Path) -> Option<Vec<u8>> {
    let path = folder.join(FILE);
    if fresh(&path) && readable(&path) {
        return None;
    }
    let lists = subscriptions(folder);
    if lists.is_empty() {
        return Some(serialized([BASELINE.to_owned()]));
    }
    let snapshot = serialized(lists);
    save(&path, &snapshot);
    Some(snapshot)
}
