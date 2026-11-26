use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{Browser, Profile};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roots {
    pub local: PathBuf,
    pub roaming: PathBuf,
}

#[derive(Deserialize)]
struct LocalState {
    profile: ProfileSection,
}

#[derive(Deserialize)]
struct ProfileSection {
    info_cache: BTreeMap<String, CachedProfile>,
}

#[derive(Deserialize)]
struct CachedProfile {
    name: String,
}

const CHROMIUM_FOLDERS: [(Browser, &str); 3] = [
    (Browser::Chrome, "Google/Chrome/User Data"),
    (Browser::Edge, "Microsoft/Edge/User Data"),
    (Browser::Brave, "BraveSoftware/Brave-Browser/User Data"),
];
const CHROMIUM_DATA_FILES: [&str; 2] = ["Bookmarks", "History"];
const LOCAL_STATE_FILE: &str = "Local State";
const FIREFOX_FOLDER: &str = "Mozilla/Firefox";
const FIREFOX_INDEX_FILE: &str = "profiles.ini";
const FIREFOX_DATA_FILE: &str = "places.sqlite";
const FIREFOX_PROFILE_SECTION: &str = "[Profile";
const RELATIVE_PATH: &str = "1";

fn local_state(user_data: &Path) -> Option<LocalState> {
    let text = fs::read(user_data.join(LOCAL_STATE_FILE)).ok()?;
    serde_json::from_slice(&text).ok()
}

fn holds_chromium_data(folder: &Path) -> bool {
    CHROMIUM_DATA_FILES
        .iter()
        .any(|file| folder.join(file).exists())
}

fn chromium_profiles(browser: Browser, user_data: &Path) -> Vec<Profile> {
    let Some(state) = local_state(user_data) else {
        return Vec::new();
    };
    state
        .profile
        .info_cache
        .into_iter()
        .map(|(folder, cached)| Profile {
            browser,
            name: cached.name,
            folder: user_data.join(folder),
        })
        .filter(|profile| holds_chromium_data(&profile.folder))
        .collect()
}

fn profile_sections(index: &str) -> Vec<HashMap<&str, &str>> {
    let mut sections: Vec<(&str, HashMap<&str, &str>)> = Vec::new();
    for line in index.lines().map(str::trim) {
        match (
            line.starts_with('['),
            line.split_once('='),
            sections.last_mut(),
        ) {
            (true, _, _) => sections.push((line, HashMap::new())),
            (false, Some((key, value)), Some((_, fields))) => {
                fields.insert(key.trim(), value.trim());
            }
            _ => {}
        }
    }
    sections
        .into_iter()
        .filter(|(header, _)| header.starts_with(FIREFOX_PROFILE_SECTION))
        .map(|(_, fields)| fields)
        .collect()
}

fn firefox_profile(base: &Path, fields: &HashMap<&str, &str>) -> Option<Profile> {
    let name = fields.get("Name")?;
    let path = fields.get("Path")?;
    let relative = fields.get("IsRelative") == Some(&RELATIVE_PATH);
    let folder = match relative {
        true => base.join(path),
        false => PathBuf::from(path),
    };
    Some(Profile {
        browser: Browser::Firefox,
        name: (*name).to_owned(),
        folder,
    })
}

fn firefox_profiles(base: &Path) -> Vec<Profile> {
    let Ok(index) = fs::read_to_string(base.join(FIREFOX_INDEX_FILE)) else {
        return Vec::new();
    };
    profile_sections(&index)
        .iter()
        .filter_map(|fields| firefox_profile(base, fields))
        .filter(|profile| profile.folder.join(FIREFOX_DATA_FILE).exists())
        .collect()
}

impl Roots {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            local: std::env::var_os("LOCALAPPDATA")?.into(),
            roaming: std::env::var_os("APPDATA")?.into(),
        })
    }

    pub fn profiles(&self) -> Vec<Profile> {
        let chromium = CHROMIUM_FOLDERS
            .iter()
            .flat_map(|(browser, folder)| chromium_profiles(*browser, &self.local.join(folder)));
        let firefox = firefox_profiles(&self.roaming.join(FIREFOX_FOLDER));
        chromium.chain(firefox).collect()
    }
}

pub fn detect() -> Vec<Profile> {
    Roots::from_env()
        .map(|roots| roots.profiles())
        .unwrap_or_default()
}
