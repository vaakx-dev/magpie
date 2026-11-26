#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Primary,
    Secondary,
    Private,
}

impl Role {
    pub fn private(self) -> bool {
        self == Self::Private
    }

    pub fn keeps_session(self) -> bool {
        self == Self::Primary
    }
}
