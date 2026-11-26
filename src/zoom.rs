use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::config::Choosable;
use magpie_engine::file;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Zoom(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Step {
    In,
    Out,
    Reset,
}

#[derive(Clone)]
pub struct Zooms {
    path: Option<PathBuf>,
    sites: BTreeMap<String, Zoom>,
}

const FILE: &str = "zoom.json";
const STEPS: [u16; 15] = [
    25, 33, 50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300,
];

impl Zoom {
    pub fn step(self, step: Step, default: Self) -> Self {
        match step {
            Step::In => self.larger(),
            Step::Out => self.smaller(),
            Step::Reset => default,
        }
    }

    fn larger(self) -> Self {
        let next = STEPS.into_iter().find(|step| *step > self.0);
        Self(next.unwrap_or(STEPS[STEPS.len() - 1]))
    }

    fn smaller(self) -> Self {
        let previous = STEPS.into_iter().rev().find(|step| *step < self.0);
        Self(previous.unwrap_or(STEPS[0]))
    }

    pub fn factor(self) -> f64 {
        f64::from(self.0) / 100.0
    }
}

impl Default for Zoom {
    fn default() -> Self {
        Self(80)
    }
}

impl Choosable for Zoom {
    const ALL: &'static [Self] = &[
        Self(67),
        Self(75),
        Self(80),
        Self(90),
        Self(100),
        Self(110),
        Self(125),
        Self(150),
    ];

    fn label(self) -> Cow<'static, str> {
        self.to_string().into()
    }
}

impl fmt::Display for Zoom {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}%", self.0)
    }
}

impl Zooms {
    pub fn open(folder: &Path) -> Self {
        let path = folder.join(FILE);
        let sites = file::load(&path).unwrap_or_else(|error| {
            eprintln!("magpie: could not read the zoom levels: {error}");
            BTreeMap::new()
        });
        Self {
            path: Some(path),
            sites,
        }
    }

    pub fn in_memory(&self) -> Self {
        Self {
            path: None,
            ..self.clone()
        }
    }

    pub fn sites(&self) -> &BTreeMap<String, Zoom> {
        &self.sites
    }

    pub fn of(&self, site: &str, default: Zoom) -> Zoom {
        self.sites.get(site).copied().unwrap_or(default)
    }

    pub fn set(&mut self, site: &str, zoom: Zoom, default: Zoom) -> io::Result<()> {
        let changed = match zoom == default {
            true => self.sites.remove(site).is_some(),
            false => self.sites.insert(site.to_owned(), zoom) != Some(zoom),
        };
        match (&self.path, changed) {
            (Some(path), true) => file::save(path, &self.sites),
            _ => Ok(()),
        }
    }
}
