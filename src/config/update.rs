use magpie_engine::update::Repo;

use super::Switch;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Updates {
    pub automatic: Switch,
    pub repo: Option<Repo>,
}

impl Default for Updates {
    fn default() -> Self {
        Self {
            automatic: Switch::On,
            repo: None,
        }
    }
}

impl Updates {
    pub fn source(&self) -> Repo {
        self.repo.clone().unwrap_or_default()
    }
}
