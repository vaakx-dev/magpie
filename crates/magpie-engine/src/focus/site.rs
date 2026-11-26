use crate::host::within;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Site {
    YouTube,
    Instagram,
    Facebook,
    X,
    TikTok,
    Reddit,
}

impl Site {
    pub const ALL: [Self; 6] = [
        Self::YouTube,
        Self::Instagram,
        Self::Facebook,
        Self::X,
        Self::TikTok,
        Self::Reddit,
    ];

    pub fn domains(self) -> &'static [&'static str] {
        match self {
            Self::YouTube => &["youtube.com"],
            Self::Instagram => &["instagram.com"],
            Self::Facebook => &["facebook.com"],
            Self::X => &["x.com", "twitter.com"],
            Self::TikTok => &["tiktok.com"],
            Self::Reddit => &["reddit.com"],
        }
    }

    pub fn of(host: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|site| site.covers(host))
    }

    fn covers(self, host: &str) -> bool {
        self.domains().iter().any(|domain| within(host, domain))
    }
}
