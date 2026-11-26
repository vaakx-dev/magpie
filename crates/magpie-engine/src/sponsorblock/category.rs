#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    Sponsor,
    SelfPromo,
    Interaction,
    Intro,
    Outro,
    Preview,
    MusicOfftopic,
    Filler,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Categories(u16);

impl Category {
    pub const ALL: [Self; 8] = [
        Self::Sponsor,
        Self::SelfPromo,
        Self::Interaction,
        Self::Intro,
        Self::Outro,
        Self::Preview,
        Self::MusicOfftopic,
        Self::Filler,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Sponsor => "sponsor",
            Self::SelfPromo => "selfpromo",
            Self::Interaction => "interaction",
            Self::Intro => "intro",
            Self::Outro => "outro",
            Self::Preview => "preview",
            Self::MusicOfftopic => "music_offtopic",
            Self::Filler => "filler",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sponsor => "Sponsor",
            Self::SelfPromo => "Self-promotion",
            Self::Interaction => "Interaction reminder",
            Self::Intro => "Intro",
            Self::Outro => "Outro",
            Self::Preview => "Preview/recap",
            Self::MusicOfftopic => "Non-music section",
            Self::Filler => "Filler",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Sponsor => "Paid promotions and sponsored segments",
            Self::SelfPromo => "Unpaid plugs like merch, donations or other channels",
            Self::Interaction => "Reminders to like, subscribe or follow",
            Self::Intro => "Intro animations and title cards",
            Self::Outro => "End cards and credits",
            Self::Preview => "Previews of what's coming up and recaps of earlier parts",
            Self::MusicOfftopic => "Talking and other non-music parts of music videos",
            Self::Filler => "Tangents and jokes the video doesn't need",
        }
    }

    pub fn color(self) -> &'static str {
        match self {
            Self::Sponsor => "#00d400",
            Self::SelfPromo => "#ffff00",
            Self::Interaction => "#cc00ff",
            Self::Intro => "#00ffff",
            Self::Outro => "#0202ed",
            Self::Preview => "#008fd6",
            Self::MusicOfftopic => "#ff9900",
            Self::Filler => "#7300ff",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == name)
    }

    fn bit(self) -> u16 {
        1 << self as u16
    }
}

impl Categories {
    pub const NONE: Self = Self(0);

    pub fn contains(self, category: Category) -> bool {
        self.0 & category.bit() != 0
    }

    pub fn insert(&mut self, category: Category) {
        self.0 |= category.bit();
    }

    pub fn remove(&mut self, category: Category) {
        self.0 &= !category.bit();
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = Category> {
        Category::ALL
            .into_iter()
            .filter(move |category| self.contains(*category))
    }
}

impl FromIterator<Category> for Categories {
    fn from_iter<I: IntoIterator<Item = Category>>(categories: I) -> Self {
        Self(
            categories
                .into_iter()
                .map(Category::bit)
                .fold(0, |bits, bit| bits | bit),
        )
    }
}

impl Default for Categories {
    fn default() -> Self {
        [
            Category::Sponsor,
            Category::SelfPromo,
            Category::Interaction,
        ]
        .into_iter()
        .collect()
    }
}
