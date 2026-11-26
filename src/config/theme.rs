use std::borrow::Cow;

use serde::Serialize;

use super::Choosable;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    #[default]
    Light,
    Dark,
}

impl Choosable for Theme {
    const ALL: &'static [Self] = &[Self::System, Self::Light, Self::Dark];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        })
    }
}

impl Theme {
    pub fn resolve(self, system: Scheme) -> Scheme {
        match self {
            Self::System => system,
            Self::Light => Scheme::Light,
            Self::Dark => Scheme::Dark,
        }
    }
}
