use std::{cmp::Ordering, fmt};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Identifier {
    Numeric(u64),
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    core: [u64; 3],
    pre_release: Vec<Identifier>,
}

const NIGHTLY_LABEL: &str = "nightly";

impl Identifier {
    fn parse(text: &str) -> Option<Self> {
        if text.is_empty() {
            return None;
        }
        if text.chars().all(|character| character.is_ascii_digit()) {
            return text.parse().ok().map(Self::Numeric);
        }
        let word = text
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-');
        word.then(|| Self::Text(text.to_owned()))
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Numeric(number) => write!(formatter, "{number}"),
            Self::Text(text) => write!(formatter, "{text}"),
        }
    }
}

fn core(text: &str) -> Option<[u64; 3]> {
    let numbers: Vec<u64> = text
        .split('.')
        .map(|part| part.parse().ok())
        .collect::<Option<_>>()?;
    numbers.try_into().ok()
}

fn pre_release(text: Option<&str>) -> Option<Vec<Identifier>> {
    text.map_or(Some(Vec::new()), |text| {
        text.split('.').map(Identifier::parse).collect()
    })
}

impl Version {
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let unprefixed = text.strip_prefix(['v', 'V']).unwrap_or(text);
        let without_build = unprefixed.split('+').next()?;
        let (numbers, label) = match without_build.split_once('-') {
            Some((numbers, label)) => (numbers, Some(label)),
            None => (without_build, None),
        };
        Some(Self {
            core: core(numbers)?,
            pre_release: pre_release(label)?,
        })
    }

    pub fn nightly(&self) -> bool {
        matches!(self.pre_release.first(), Some(Identifier::Text(label)) if label == NIGHTLY_LABEL)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let [major, minor, patch] = self.core;
        write!(formatter, "{major}.{minor}.{patch}")?;
        let labels: Vec<String> = self.pre_release.iter().map(Identifier::to_string).collect();
        match labels.is_empty() {
            true => Ok(()),
            false => write!(formatter, "-{}", labels.join(".")),
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let released = (self.pre_release.is_empty(), other.pre_release.is_empty());
        let pre_release = match released {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => self.pre_release.cmp(&other.pre_release),
        };
        self.core.cmp(&other.core).then(pre_release)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
