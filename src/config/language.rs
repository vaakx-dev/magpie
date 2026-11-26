use std::borrow::Cow;

use super::Choosable;
use crate::locale::{Locale, Text};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    System,
    English,
    Spanish,
    French,
    German,
    Japanese,
    ChineseSimplified,
}

impl Choosable for Language {
    const ALL: &'static [Self] = &[
        Self::System,
        Self::English,
        Self::Spanish,
        Self::French,
        Self::German,
        Self::Japanese,
        Self::ChineseSimplified,
    ];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::System => "System",
            Self::English => "English",
            Self::Spanish => "Spanish",
            Self::French => "French",
            Self::German => "German",
            Self::Japanese => "Japanese",
            Self::ChineseSimplified => "Chinese (Simplified)",
        })
    }

    fn name(self, locale: Locale) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::System => locale.text(Text::System),
            Self::English => "English",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::German => "Deutsch",
            Self::Japanese => "日本語",
            Self::ChineseSimplified => "简体中文",
        })
    }
}

impl Language {
    pub fn chosen(self) -> Option<Locale> {
        match self {
            Self::System => None,
            Self::English => Some(Locale::English),
            Self::Spanish => Some(Locale::Spanish),
            Self::French => Some(Locale::French),
            Self::German => Some(Locale::German),
            Self::Japanese => Some(Locale::Japanese),
            Self::ChineseSimplified => Some(Locale::ChineseSimplified),
        }
    }

    pub fn resolve(self, system: Option<Locale>) -> Locale {
        self.chosen().or(system).unwrap_or(Locale::English)
    }

    pub fn locale(self) -> Locale {
        self.resolve(Locale::system())
    }
}
