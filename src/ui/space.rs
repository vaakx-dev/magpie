use gpui::{Hsla, rgb};

use super::icon::Icon;

use crate::storage::session::{Link, Session, Space, SpaceId};

pub struct Theme {
    pub name: &'static str,
    pub icon: Icon,
}

pub const THEMES: [Theme; 6] = [
    Theme {
        name: "Personal",
        icon: Icon::Sparkle,
    },
    Theme {
        name: "Work",
        icon: Icon::Briefcase,
    },
    Theme {
        name: "Reading",
        icon: Icon::BookOpen,
    },
    Theme {
        name: "Studio",
        icon: Icon::Palette,
    },
    Theme {
        name: "Lab",
        icon: Icon::Flask,
    },
    Theme {
        name: "Desk",
        icon: Icon::Coffee,
    },
];

const SEEDED_SPACES: usize = 3;

const FAVORITES: [(&str, &str); 2] = [
    ("YouTube", "https://www.youtube.com/"),
    ("Magpie", "https://github.com/vaakx-dev/magpie"),
];

const FAVORITE_COLORS: [u32; 6] = [0x8c959f, 0xff6600, 0xe0703a, 0xe5484d, 0x4c8df6, 0x2a9d8f];

pub fn theme(space: &Space) -> &'static Theme {
    THEMES
        .iter()
        .find(|theme| theme.name == space.theme)
        .unwrap_or(&THEMES[0])
}

pub fn space(id: SpaceId, theme: &Theme) -> Space {
    Space::new(id, theme.name, theme.name)
}

pub fn favorite_color(link: &Link) -> Hsla {
    let hash = link.url.bytes().fold(0usize, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(usize::from(byte))
    });
    rgb(FAVORITE_COLORS[hash % FAVORITE_COLORS.len()]).into()
}

fn default_favorites() -> Vec<Link> {
    FAVORITES
        .iter()
        .map(|(title, url)| Link {
            url: (*url).to_owned(),
            title: (*title).to_owned(),
        })
        .collect()
}

fn seed(session: &mut Session) {
    session.spaces = THEMES
        .iter()
        .take(SEEDED_SPACES)
        .zip(1..)
        .map(|(theme, id)| Space {
            favorites: default_favorites(),
            ..space(SpaceId(id), theme)
        })
        .collect();
}

pub fn prepare(session: &mut Session) {
    if session.spaces.is_empty() {
        seed(session);
    }
    if session.active_space().is_none() {
        session.active = session.spaces.first().map(|space| space.id);
    }
    for space in &mut session.spaces {
        if space.active_tab().is_none() {
            space.active = space.today.first().map(|tab| tab.id);
        }
    }
}
