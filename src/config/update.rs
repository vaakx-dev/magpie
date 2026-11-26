use std::borrow::Cow;

use magpie_engine::update::{Channel, Repo};

use super::{Choosable, Switch};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Updates {
    pub automatic: Switch,
    pub channel: Channel,
    pub repo: Option<Repo>,
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            automatic: Switch::On,
            channel: Channel::default(),
            repo: None,
        }
    }
}

impl Updates {
    pub fn source(&self) -> Repo {
        self.repo.clone().unwrap_or_default()
    }
}

impl Choosable for Channel {
    const ALL: &'static [Self] = &Channel::ALL;

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(Channel::label(self))
    }
}
