use super::Version;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Channel {
    #[default]
    Stable,
    Nightly,
}

impl Channel {
    pub const ALL: [Self; 2] = [Self::Stable, Self::Nightly];

    pub fn label(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Nightly => "Nightly",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|channel| channel.label().eq_ignore_ascii_case(text.trim()))
    }

    pub fn offers(self, candidate: &Version, current: &Version) -> bool {
        match self {
            Self::Stable => candidate > current,
            Self::Nightly => !current.nightly() || candidate > current,
        }
    }
}
