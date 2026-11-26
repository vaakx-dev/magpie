use super::category::Category;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Skip,
    Mute,
    Poi,
    Full,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub start_secs: f64,
    pub end_secs: f64,
    pub category: Category,
    pub action: Action,
    pub uuid: String,
    pub votes: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Skip {
    pub start_secs: f64,
    pub end_secs: f64,
    pub category: Category,
}

impl Action {
    pub const ALL: [Self; 4] = [Self::Skip, Self::Mute, Self::Poi, Self::Full];

    pub fn name(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Mute => "mute",
            Self::Poi => "poi",
            Self::Full => "full",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.name() == name)
    }
}

impl From<&Segment> for Skip {
    fn from(segment: &Segment) -> Self {
        Self {
            start_secs: segment.start_secs,
            end_secs: segment.end_secs,
            category: segment.category,
        }
    }
}

fn absorb(mut skips: Vec<Skip>, next: Skip) -> Vec<Skip> {
    match skips.last_mut() {
        Some(last) if next.start_secs <= last.end_secs => {
            last.end_secs = last.end_secs.max(next.end_secs);
        }
        _ => skips.push(next),
    }
    skips
}

pub fn skips(segments: &[Segment]) -> Vec<Skip> {
    let mut ranges: Vec<Skip> = segments
        .iter()
        .filter(|segment| segment.action == Action::Skip)
        .map(Skip::from)
        .collect();
    ranges.sort_by(|left, right| left.start_secs.total_cmp(&right.start_secs));
    ranges.into_iter().fold(Vec::new(), absorb)
}
