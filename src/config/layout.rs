use std::borrow::Cow;

use super::{Choosable, Switch};
use crate::locale::{Locale, Text};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sidebar {
    #[default]
    Shown,
    Hidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FavoritesSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FavoritesColumns {
    #[default]
    Auto,
    Four,
    Five,
    Six,
    Custom(u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlStyle {
    #[default]
    Windows,
    Mac,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Controls {
    pub side: Side,
    pub style: ControlStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarWidth(u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub top_bar: Switch,
    pub sidebar: Sidebar,
    pub sidebar_side: Side,
    pub sidebar_width: SidebarWidth,
    pub favorites_size: FavoritesSize,
    pub favorites_columns: FavoritesColumns,
    pub controls: Controls,
}

const LEGACY_SIDEBAR_NAMES: [(&str, Sidebar); 3] = [
    ("Full", Sidebar::Shown),
    ("Rail", Sidebar::Shown),
    ("Hidden", Sidebar::Hidden),
];

impl Choosable for Side {
    const ALL: &'static [Self] = &[Self::Left, Self::Right];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Left => "Left",
            Self::Right => "Right",
        })
    }
}

impl Choosable for Sidebar {
    const ALL: &'static [Self] = &[Self::Shown, Self::Hidden];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Shown => "Show",
            Self::Hidden => "Hide",
        })
    }

    fn parse(text: &str) -> Option<Self> {
        let legacy = LEGACY_SIDEBAR_NAMES
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(text))
            .map(|(_, sidebar)| sidebar);
        Self::ALL
            .iter()
            .copied()
            .find(|sidebar| sidebar.label().eq_ignore_ascii_case(text))
            .or(legacy)
    }
}

impl Choosable for FavoritesSize {
    const ALL: &'static [Self] = &[Self::Small, Self::Medium, Self::Large];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
        })
    }
}

impl Choosable for FavoritesColumns {
    const ALL: &'static [Self] = &[
        Self::Auto,
        Self::Custom(1),
        Self::Custom(2),
        Self::Custom(3),
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Custom(7),
        Self::Custom(8),
    ];

    fn label(self) -> Cow<'static, str> {
        match self.count() {
            Some(count) => Cow::Owned(count.to_string()),
            None => Cow::Borrowed(Self::AUTO_LABEL),
        }
    }

    fn name(self, locale: Locale) -> Cow<'static, str> {
        match self {
            Self::Auto => Cow::Borrowed(locale.text(Text::Auto)),
            manual => manual.label(),
        }
    }

    fn parse(text: &str) -> Option<Self> {
        if text.eq_ignore_ascii_case(Self::AUTO_LABEL) {
            return Some(Self::Auto);
        }
        let count: u32 = text.parse().ok()?;
        Some(Self::new(count))
    }
}

impl Choosable for ControlStyle {
    const ALL: &'static [Self] = &[Self::Windows, Self::Mac];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Windows => "Windows",
            Self::Mac => "Mac",
        })
    }
}

impl Sidebar {
    pub fn toggled(self) -> Self {
        match self {
            Self::Shown => Self::Hidden,
            Self::Hidden => Self::Shown,
        }
    }
}

impl FavoritesSize {
    pub const GAP_PX: f32 = 8.0;

    pub fn tile_height_px(self) -> f32 {
        match self {
            Self::Small => 36.0,
            Self::Medium => 44.0,
            Self::Large => 56.0,
        }
    }
}

impl FavoritesColumns {
    const MIN: u32 = 1;
    const MAX: u32 = 8;
    const AUTO_MIN: u16 = 2;
    const AUTO_MAX: u16 = 6;
    const AUTO_LABEL: &'static str = "Auto";
    const LEGIBLE_TILE_PX: f32 = 28.0;
    const COMFORTABLE_TILE_PX: f32 = 48.0;

    pub fn new(count: u32) -> Self {
        match count.clamp(Self::MIN, Self::MAX) {
            4 => Self::Four,
            5 => Self::Five,
            6 => Self::Six,
            custom => Self::Custom(custom as u8),
        }
    }

    pub fn count(self) -> Option<u16> {
        match self {
            Self::Auto => None,
            Self::Four => Some(4),
            Self::Five => Some(5),
            Self::Six => Some(6),
            Self::Custom(count) => Some(u16::from(count)),
        }
    }

    fn fit(row_px: f32, tile_px: f32) -> u16 {
        let gap_px = FavoritesSize::GAP_PX;
        let tiles = ((row_px + gap_px) / (tile_px + gap_px)).floor();
        tiles.max(1.0) as u16
    }

    fn comfortable(row_px: f32) -> u16 {
        Self::fit(row_px, Self::COMFORTABLE_TILE_PX).clamp(Self::AUTO_MIN, Self::AUTO_MAX)
    }

    pub fn fitting(self, row_px: f32) -> u16 {
        let wanted = self.count().unwrap_or_else(|| Self::comfortable(row_px));
        wanted.min(Self::fit(row_px, Self::LEGIBLE_TILE_PX))
    }
}

impl Default for Controls {
    fn default() -> Self {
        match cfg!(target_os = "macos") {
            true => Self {
                side: Side::Left,
                style: ControlStyle::Mac,
            },
            false => Self {
                side: Side::Right,
                style: ControlStyle::Windows,
            },
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        let controls = Controls::default();
        Self {
            top_bar: Switch::default(),
            sidebar: Sidebar::default(),
            sidebar_side: controls.side,
            sidebar_width: SidebarWidth::default(),
            favorites_size: FavoritesSize::default(),
            favorites_columns: FavoritesColumns::default(),
            controls,
        }
    }
}

impl SidebarWidth {
    pub const MIN_PX: f32 = 56.0;
    pub const MAX_PX: f32 = 360.0;
    const DEFAULT_PX: u16 = 240;

    pub fn new(width_px: f32) -> Self {
        Self(width_px.round().clamp(Self::MIN_PX, Self::MAX_PX) as u16)
    }

    pub fn parse(text: &str) -> Option<Self> {
        let width_px: u16 = text.parse().ok()?;
        Some(Self::new(f32::from(width_px)))
    }

    pub fn px(self) -> f32 {
        f32::from(self.0)
    }

    pub fn text(self) -> String {
        self.0.to_string()
    }
}

impl Default for SidebarWidth {
    fn default() -> Self {
        Self(Self::DEFAULT_PX)
    }
}
