mod text;

use windows::Win32::Globalization::GetUserDefaultUILanguage;

pub use text::Text;

const PRIMARY_LANGUAGE_MASK: u16 = 0x3ff;
const SUBLANGUAGE_SHIFT: u16 = 10;
const LANG_CHINESE: u16 = 0x04;
const LANG_GERMAN: u16 = 0x07;
const LANG_ENGLISH: u16 = 0x09;
const LANG_SPANISH: u16 = 0x0a;
const LANG_FRENCH: u16 = 0x0c;
const LANG_JAPANESE: u16 = 0x11;
const SUBLANG_CHINESE_SIMPLIFIED: u16 = 0x02;
const SUBLANG_CHINESE_SINGAPORE: u16 = 0x04;
const SLOT: &str = "{}";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    English,
    Spanish,
    French,
    German,
    Japanese,
    ChineseSimplified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LanguageId(pub u16);

impl Locale {
    const COUNT: usize = 6;

    pub fn from_language_id(LanguageId(id): LanguageId) -> Option<Self> {
        let primary = id & PRIMARY_LANGUAGE_MASK;
        let sublanguage = id >> SUBLANGUAGE_SHIFT;
        match (primary, sublanguage) {
            (LANG_ENGLISH, _) => Some(Self::English),
            (LANG_SPANISH, _) => Some(Self::Spanish),
            (LANG_FRENCH, _) => Some(Self::French),
            (LANG_GERMAN, _) => Some(Self::German),
            (LANG_JAPANESE, _) => Some(Self::Japanese),
            (LANG_CHINESE, SUBLANG_CHINESE_SIMPLIFIED | SUBLANG_CHINESE_SINGAPORE) => {
                Some(Self::ChineseSimplified)
            }
            _ => None,
        }
    }

    pub fn system() -> Option<Self> {
        Self::from_language_id(LanguageId(unsafe { GetUserDefaultUILanguage() }))
    }

    pub fn tag(self) -> &'static str {
        match self {
            Self::English => "en-US",
            Self::Spanish => "es-ES",
            Self::French => "fr-FR",
            Self::German => "de-DE",
            Self::Japanese => "ja-JP",
            Self::ChineseSimplified => "zh-CN",
        }
    }

    pub fn accept_language(self) -> &'static str {
        match self {
            Self::English => "en-US,en;q=0.9",
            Self::Spanish => "es-ES,es;q=0.9",
            Self::French => "fr-FR,fr;q=0.9",
            Self::German => "de-DE,de;q=0.9",
            Self::Japanese => "ja-JP,ja;q=0.9",
            Self::ChineseSimplified => "zh-CN,zh;q=0.9",
        }
    }

    pub fn text(self, text: Text) -> &'static str {
        text.translations()[self as usize]
    }

    pub fn fill(self, text: Text, value: &str) -> String {
        self.text(text).replacen(SLOT, value, 1)
    }
}
