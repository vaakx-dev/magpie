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

pub fn theme(space: &Space) -> &'static Theme {
    THEMES
        .iter()
        .find(|theme| theme.name == space.theme)
        .unwrap_or(&THEMES[0])
}

pub fn space(id: SpaceId, theme: &Theme) -> Space {
    Space::new(id, theme.name, theme.name)
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
