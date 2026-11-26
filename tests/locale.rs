use magpie::{
    config::{Choosable, Language},
    locale::{LanguageId, Locale, Text},
};

#[test]
fn windows_ui_languages_map_to_supported_locales() {
    let locale = |id| Locale::from_language_id(LanguageId(id));

    assert_eq!(locale(0x0409), Some(Locale::English));
    assert_eq!(locale(0x0809), Some(Locale::English));
    assert_eq!(locale(0x080a), Some(Locale::Spanish));
    assert_eq!(locale(0x0c0c), Some(Locale::French));
    assert_eq!(locale(0x0c07), Some(Locale::German));
    assert_eq!(locale(0x0411), Some(Locale::Japanese));
    assert_eq!(locale(0x0804), Some(Locale::ChineseSimplified));
    assert_eq!(locale(0x1004), Some(Locale::ChineseSimplified));
    assert_eq!(locale(0x0404), None);
    assert_eq!(locale(0x0410), None);
}

#[test]
fn system_language_follows_windows_and_falls_back_to_english() {
    assert_eq!(
        Language::System.resolve(Some(Locale::French)),
        Locale::French
    );
    assert_eq!(Language::System.resolve(None), Locale::English);
    assert_eq!(
        Language::Japanese.resolve(Some(Locale::French)),
        Locale::Japanese
    );
}

#[test]
fn filled_translations_keep_the_value_in_every_language() {
    let locales = Language::ALL
        .iter()
        .filter_map(|language| language.chosen());
    for locale in locales {
        let filled_texts = [
            Text::SearchFor,
            Text::OpenAddress,
            Text::ClearFailed,
            Text::WantsCamera,
            Text::WantsMicrophone,
            Text::WantsNotifications,
            Text::WantsLocation,
            Text::WantsClipboard,
            Text::WantsPopups,
        ];
        for text in filled_texts {
            let filled = locale.fill(text, "magpie");

            assert!(filled.contains("magpie"), "{locale:?} {text:?}");
            assert!(!filled.contains("{}"), "{locale:?} {text:?}");
        }
    }
}
