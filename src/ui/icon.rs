use std::borrow::Cow;

use gpui::{AssetSource, Hsla, SharedString, Svg, prelude::*, px, svg};

macro_rules! icons {
    ($($variant:ident => $name:literal,)*) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum Icon {
            $($variant,)*
        }

        impl Icon {
            const ALL: &[Icon] = &[$(Icon::$variant,)*];

            fn path(self) -> &'static str {
                match self {
                    $(Icon::$variant => concat!("icons/", $name, ".svg"),)*
                }
            }

            fn bytes(self) -> &'static [u8] {
                match self {
                    $(Icon::$variant => include_bytes!(concat!("../../assets/icons/", $name, ".svg")),)*
                }
            }
        }
    };
}

icons! {
    ArrowLeft => "arrow-left",
    ArrowRight => "arrow-right",
    RotateCw => "rotate-cw",
    X => "x",
    Home => "home",
    PanelLeft => "panel-left",
    PanelRight => "panel-right",
    PanelLeftClose => "panel-left-close",
    Columns2 => "columns-2",
    Plus => "plus",
    Minus => "minus",
    Printer => "printer",
    Bookmark => "bookmark",
    ZoomIn => "zoom-in",
    Folder => "folder",
    FolderOpen => "folder-open",
    Pin => "pin",
    Star => "star",
    Archive => "archive",
    Layers => "layers",
    Globe => "globe",
    Shield => "shield",
    Lock => "lock",
    LockOpen => "lock-open",
    Camera => "camera",
    Microphone => "mic",
    Bell => "bell",
    MapPin => "map-pin",
    Clipboard => "clipboard",
    Search => "search",
    Settings => "settings",
    History => "history",
    Download => "download",
    Play => "play",
    Pause => "pause",
    SkipBack => "skip-back",
    SkipForward => "skip-forward",
    ChevronDown => "chevron-down",
    ChevronUp => "chevron-up",
    ChevronRight => "chevron-right",
    ChevronLeft => "chevron-left",
    Check => "check",
    Trash => "trash",
    ExternalLink => "external-link",
    Command => "command",
    Sparkle => "sparkle",
    Briefcase => "briefcase",
    BookOpen => "book-open",
    Palette => "palette",
    Flask => "flask",
    Coffee => "coffee",
    MoreHorizontal => "more-horizontal",
    Volume => "volume-2",
    VolumeMuted => "volume-x",
    Menu => "menu",
    AppWindow => "app-window",
    EyeOff => "eye-off",
    Copy => "copy",
    Link => "link",
    SquarePlus => "square-plus",
    FileSearch => "file-search",
    FileDown => "file-down",
    Code => "code",
    Maximize => "maximize",
    KeyRound => "key-round",
    ScanFace => "scan-face",
    Pencil => "pencil",
    Moon => "moon",
    User => "user",
}

pub const ICON_PX: f32 = 16.0;

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let icon = Icon::ALL.iter().find(|icon| icon.path() == path);
        Ok(icon.map(|icon| Cow::Borrowed(icon.bytes())))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let paths = Icon::ALL
            .iter()
            .map(|icon| icon.path())
            .filter(|icon| icon.starts_with(path));
        Ok(paths.map(SharedString::new_static).collect())
    }
}

impl Icon {
    pub fn tinted(self, color: impl Into<Hsla>) -> Svg {
        svg()
            .path(self.path())
            .size(px(ICON_PX))
            .flex_none()
            .text_color(color)
    }
}
