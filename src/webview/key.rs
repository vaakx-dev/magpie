use gpui::{Keystroke, Modifiers};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_CONTROL, VK_DOWN, VK_ESCAPE, VK_F1, VK_F24, VK_LEFT, VK_MENU,
    VK_OEM_2, VK_OEM_5, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_RIGHT, VK_SHIFT,
    VK_TAB, VK_UP,
};

const NAMED_KEYS: [(VIRTUAL_KEY, &str); 12] = [
    (VK_LEFT, "left"),
    (VK_RIGHT, "right"),
    (VK_UP, "up"),
    (VK_DOWN, "down"),
    (VK_TAB, "tab"),
    (VK_ESCAPE, "escape"),
    (VK_OEM_COMMA, ","),
    (VK_OEM_PERIOD, "."),
    (VK_OEM_2, "/"),
    (VK_OEM_5, "\\"),
    (VK_OEM_PLUS, "="),
    (VK_OEM_MINUS, "-"),
];

fn held(key: VIRTUAL_KEY) -> bool {
    unsafe { GetKeyState(i32::from(key.0)) < 0 }
}

fn modifiers() -> Modifiers {
    Modifiers {
        control: held(VK_CONTROL),
        alt: held(VK_MENU),
        shift: held(VK_SHIFT),
        ..Modifiers::default()
    }
}

fn named_key(key: VIRTUAL_KEY) -> Option<String> {
    NAMED_KEYS
        .iter()
        .find(|(code, _)| *code == key)
        .map(|(_, name)| (*name).to_owned())
}

fn function_key(key: VIRTUAL_KEY) -> Option<String> {
    (VK_F1.0..=VK_F24.0)
        .contains(&key.0)
        .then(|| format!("f{}", key.0 - VK_F1.0 + 1))
}

fn character_key(key: VIRTUAL_KEY) -> Option<String> {
    char::from_u32(key.0.into())
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase().to_string())
}

pub fn keystroke(virtual_key: u32) -> Option<Keystroke> {
    let key = VIRTUAL_KEY(u16::try_from(virtual_key).ok()?);
    let name = named_key(key)
        .or_else(|| function_key(key))
        .or_else(|| character_key(key))?;
    Some(Keystroke {
        modifiers: modifiers(),
        key: name,
        key_char: None,
    })
}
