use std::{
    collections::BTreeMap,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::locale::Text;
use magpie_engine::file;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "&'static str", try_from = "String")]
pub enum Kind {
    Camera,
    Microphone,
    Notifications,
    Location,
    Clipboard,
    Popups,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "&'static str", try_from = "String")]
pub enum State {
    #[default]
    Ask,
    Allow,
    Block,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct Origin(String);

#[derive(Debug)]
pub struct Unknown(String);

pub type Rules = BTreeMap<Kind, State>;

#[derive(Clone, Debug, Default)]
pub struct Permissions {
    path: Option<PathBuf>,
    sites: BTreeMap<Origin, Rules>,
}

const FILE: &str = "permissions.conf";
const SCHEME_SEPARATOR: &str = "://";

impl Kind {
    pub const ALL: [Self; 6] = [
        Self::Camera,
        Self::Microphone,
        Self::Notifications,
        Self::Location,
        Self::Clipboard,
        Self::Popups,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::Notifications => "notifications",
            Self::Location => "location",
            Self::Clipboard => "clipboard",
            Self::Popups => "popups",
        }
    }

    pub fn label(self) -> Text {
        match self {
            Self::Camera => Text::Camera,
            Self::Microphone => Text::Microphone,
            Self::Notifications => Text::Notifications,
            Self::Location => Text::Location,
            Self::Clipboard => Text::Clipboard,
            Self::Popups => Text::Popups,
        }
    }

    pub fn request(self) -> Text {
        match self {
            Self::Camera => Text::WantsCamera,
            Self::Microphone => Text::WantsMicrophone,
            Self::Notifications => Text::WantsNotifications,
            Self::Location => Text::WantsLocation,
            Self::Clipboard => Text::WantsClipboard,
            Self::Popups => Text::WantsPopups,
        }
    }

    fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }
}

impl State {
    pub const ALL: [Self; 3] = [Self::Ask, Self::Allow, Self::Block];

    pub fn name(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Allow => "allow",
            Self::Block => "block",
        }
    }

    pub fn label(self) -> Text {
        match self {
            Self::Ask => Text::Ask,
            Self::Allow => Text::Allow,
            Self::Block => Text::Block,
        }
    }

    fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|state| state.name() == name)
    }
}

impl Origin {
    pub fn of(url: &Url) -> Option<Self> {
        let origin = url.origin();
        origin
            .is_tuple()
            .then(|| Self(origin.ascii_serialization()))
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::of(&Url::parse(text).ok()?)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn host(&self) -> &str {
        self.0
            .split_once(SCHEME_SEPARATOR)
            .map_or(self.as_str(), |(_, host)| host)
    }
}

impl From<Kind> for &'static str {
    fn from(kind: Kind) -> Self {
        kind.name()
    }
}

impl From<State> for &'static str {
    fn from(state: State) -> Self {
        state.name()
    }
}

impl From<Origin> for String {
    fn from(origin: Origin) -> Self {
        origin.0
    }
}

impl TryFrom<String> for Kind {
    type Error = Unknown;

    fn try_from(name: String) -> Result<Self, Unknown> {
        Self::parse(&name).ok_or(Unknown(name))
    }
}

impl TryFrom<String> for State {
    type Error = Unknown;

    fn try_from(name: String) -> Result<Self, Unknown> {
        Self::parse(&name).ok_or(Unknown(name))
    }
}

impl TryFrom<String> for Origin {
    type Error = Unknown;

    fn try_from(text: String) -> Result<Self, Unknown> {
        Self::parse(&text).ok_or(Unknown(text))
    }
}

impl fmt::Display for Unknown {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "unknown site permission value: {}", self.0)
    }
}

fn rule(line: &str) -> Option<(Origin, Kind, State)> {
    let (key, state) = line.split_once('=')?;
    let (origin, kind) = key.trim().rsplit_once(' ')?;
    Some((
        Origin::parse(origin.trim())?,
        Kind::parse(kind)?,
        State::parse(state.trim())?,
    ))
}

fn parse(text: &str) -> BTreeMap<Origin, Rules> {
    text.lines()
        .filter_map(rule)
        .filter(|(_, _, state)| *state != State::Ask)
        .fold(BTreeMap::new(), |mut sites, (origin, kind, state)| {
            sites.entry(origin).or_default().insert(kind, state);
            sites
        })
}

fn read(path: &Path) -> io::Result<BTreeMap<Origin, Rules>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(parse(&text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(error) => Err(error),
    }
}

fn write(path: &Path, text: &str) -> io::Result<()> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    file::replace(path, text.as_bytes())
}

impl Permissions {
    pub fn open(folder: &Path) -> Self {
        let path = folder.join(FILE);
        let sites = read(&path).unwrap_or_else(|error| {
            eprintln!("magpie: could not read the site permissions: {error}");
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

    pub fn sites(&self) -> &BTreeMap<Origin, Rules> {
        &self.sites
    }

    pub fn rules(&self, origin: &Origin) -> Rules {
        self.sites.get(origin).cloned().unwrap_or_default()
    }

    pub fn state(&self, origin: &Origin, kind: Kind) -> State {
        self.sites
            .get(origin)
            .and_then(|rules| rules.get(&kind))
            .copied()
            .unwrap_or_default()
    }

    pub fn set(&mut self, origin: &Origin, kind: Kind, state: State) -> io::Result<()> {
        if self.state(origin, kind) == state {
            return Ok(());
        }
        let rules = self.sites.entry(origin.clone()).or_default();
        match state {
            State::Ask => rules.remove(&kind),
            State::Allow | State::Block => rules.insert(kind, state),
        };
        self.sites.retain(|_, rules| !rules.is_empty());
        self.save()
    }

    fn text(&self) -> String {
        self.sites
            .iter()
            .flat_map(|(origin, rules)| {
                rules.iter().map(move |(kind, state)| {
                    format!("{} {} = {}\n", origin.as_str(), kind.name(), state.name())
                })
            })
            .collect()
    }

    fn save(&self) -> io::Result<()> {
        match &self.path {
            Some(path) => write(path, &self.text()),
            None => Ok(()),
        }
    }
}
