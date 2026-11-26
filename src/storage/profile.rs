use std::{collections::HashSet, iter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const BUILT_IN_ID: &str = "default";
const CUSTOM_PREFIX: &str = "magpie-";
const MAX_ID_CHARS: usize = 64;
const MAX_NAME_CHARS: usize = 40;
const ID_SYMBOLS: [char; 11] = ['#', '@', '$', '(', ')', '+', '-', '_', '~', '.', ' '];
const ID_FORBIDDEN_ENDINGS: [char; 2] = ['.', ' '];

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProfileId(String);

#[derive(Deserialize)]
struct UncheckedProfile {
    id: ProfileId,
    name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "UncheckedProfile")]
pub struct Profile {
    pub id: ProfileId,
    name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<Profile>", into = "Vec<Profile>")]
pub struct Profiles(Vec<Profile>);

fn webview_safe(id: &str) -> bool {
    let allowed =
        |character: char| character.is_ascii_alphanumeric() || ID_SYMBOLS.contains(&character);
    (1..=MAX_ID_CHARS).contains(&id.len())
        && id.chars().all(allowed)
        && !id.ends_with(ID_FORBIDDEN_ENDINGS)
        && !id.eq_ignore_ascii_case(BUILT_IN_ID)
}

fn sanitized(name: &str) -> Option<String> {
    let spaced: String = name
        .chars()
        .map(|character| match character.is_control() {
            true => ' ',
            false => character,
        })
        .collect();
    let words: Vec<&str> = spaced.split_whitespace().collect();
    let clipped: String = words.join(" ").chars().take(MAX_NAME_CHARS).collect();
    Some(clipped.trim_end().to_owned()).filter(|name| !name.is_empty())
}

impl ProfileId {
    fn generate() -> Self {
        Self(format!("{CUSTOM_PREFIX}{}", Uuid::new_v4().simple()))
    }

    pub fn built_in(&self) -> bool {
        self.0 == BUILT_IN_ID
    }

    pub fn webview_name(&self) -> Option<&str> {
        (!self.built_in()).then_some(&self.0)
    }
}

impl Default for ProfileId {
    fn default() -> Self {
        Self(BUILT_IN_ID.to_owned())
    }
}

impl TryFrom<String> for ProfileId {
    type Error = &'static str;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        match id == BUILT_IN_ID || webview_safe(&id) {
            true => Ok(Self(id)),
            false => Err("profile id is not a valid WebView2 profile name"),
        }
    }
}

impl From<ProfileId> for String {
    fn from(id: ProfileId) -> Self {
        id.0
    }
}

impl TryFrom<UncheckedProfile> for Profile {
    type Error = &'static str;

    fn try_from(unchecked: UncheckedProfile) -> Result<Self, Self::Error> {
        if unchecked.id.built_in() {
            return Err("the built-in profile is not stored");
        }
        let name = sanitized(&unchecked.name).ok_or("profile name is empty")?;
        Ok(Self {
            id: unchecked.id,
            name,
        })
    }
}

impl Profile {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<Vec<Profile>> for Profiles {
    type Error = &'static str;

    fn try_from(profiles: Vec<Profile>) -> Result<Self, Self::Error> {
        let mut seen = HashSet::new();
        let unique = profiles.iter().all(|profile| seen.insert(&profile.id));
        match unique {
            true => Ok(Self(profiles)),
            false => Err("two profiles share an id"),
        }
    }
}

impl From<Profiles> for Vec<Profile> {
    fn from(Profiles(profiles): Profiles) -> Self {
        profiles
    }
}

impl Profiles {
    pub fn custom(&self) -> &[Profile] {
        &self.0
    }

    pub fn ids(&self) -> Vec<ProfileId> {
        let custom = self.0.iter().map(|profile| profile.id.clone());
        iter::once(ProfileId::default()).chain(custom).collect()
    }

    pub fn named(&self, built_in_name: &str) -> Vec<Profile> {
        let built_in = Profile {
            id: ProfileId::default(),
            name: built_in_name.to_owned(),
        };
        iter::once(built_in).chain(self.0.iter().cloned()).collect()
    }

    pub fn custom_name(&self, id: &ProfileId) -> Option<&str> {
        self.find(id).map(Profile::name)
    }

    pub fn contains(&self, id: &ProfileId) -> bool {
        id.built_in() || self.find(id).is_some()
    }

    fn find(&self, id: &ProfileId) -> Option<&Profile> {
        self.0.iter().find(|profile| &profile.id == id)
    }

    pub fn create(&mut self, name: &str) -> Option<ProfileId> {
        let name = sanitized(name)?;
        let id = ProfileId::generate();
        self.0.push(Profile {
            id: id.clone(),
            name,
        });
        Some(id)
    }

    pub fn rename(&mut self, id: &ProfileId, name: &str) -> bool {
        let Some(name) = sanitized(name) else {
            return false;
        };
        let Some(profile) = self.0.iter_mut().find(|profile| &profile.id == id) else {
            return false;
        };
        let changed = profile.name != name;
        profile.name = name;
        changed
    }

    pub fn delete(&mut self, id: &ProfileId) -> bool {
        let before = self.0.len();
        self.0.retain(|profile| &profile.id != id);
        self.0.len() < before
    }
}
