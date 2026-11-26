use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    config::Choosable,
    locale::{Locale, Text},
};

const SCRIPT: &str = include_str!("reader.js");
const STYLE_KIND: &str = "magpie_reader";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Font {
    #[default]
    Sans,
    Serif,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextSize(pub u8);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub font: Font,
    pub size: TextSize,
}

#[derive(Deserialize)]
struct Incoming {
    kind: String,
    font: String,
    size: u8,
}

#[derive(Serialize)]
struct Labels {
    exit: &'static str,
    sans: &'static str,
    serif: &'static str,
    smaller: &'static str,
    larger: &'static str,
}

impl Choosable for Font {
    const ALL: &'static [Self] = &[Self::Sans, Self::Serif];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Sans => "Sans",
            Self::Serif => "Serif",
        })
    }

    fn name(self, locale: Locale) -> Cow<'static, str> {
        Cow::Borrowed(locale.text(match self {
            Self::Sans => Text::SansSerif,
            Self::Serif => Text::Serif,
        }))
    }
}

impl Default for TextSize {
    fn default() -> Self {
        Self(18)
    }
}

impl Choosable for TextSize {
    const ALL: &'static [Self] = &[
        Self(14),
        Self(16),
        Self(18),
        Self(20),
        Self(22),
        Self(24),
        Self(28),
    ];

    fn label(self) -> Cow<'static, str> {
        Cow::Owned(self.0.to_string())
    }

    fn name(self, _: Locale) -> Cow<'static, str> {
        Cow::Owned(format!("{} px", self.0))
    }
}

fn labels(locale: Locale) -> Labels {
    Labels {
        exit: locale.text(Text::ExitReader),
        sans: locale.text(Text::SansSerif),
        serif: locale.text(Text::Serif),
        smaller: locale.text(Text::SmallerText),
        larger: locale.text(Text::LargerText),
    }
}

impl Config {
    pub fn parse(json: &str) -> Option<Self> {
        let incoming: Incoming = serde_json::from_str(json).ok()?;
        if incoming.kind != STYLE_KIND {
            return None;
        }
        let size = TextSize(incoming.size);
        Some(Self {
            font: Font::parse(&incoming.font)?,
            size: TextSize::ALL.contains(&size).then_some(size)?,
        })
    }

    pub fn script(&self, locale: Locale) -> String {
        let sizes: Vec<u8> = TextSize::ALL.iter().map(|size| size.0).collect();
        let options = json!({
            "font": self.font.label(),
            "size": self.size.0,
            "sizes": sizes,
            "text": labels(locale),
        });
        format!("({SCRIPT})({options});")
    }
}
