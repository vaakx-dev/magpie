use crate::storage::session::TabId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub tab: TabId,
    pub playing: bool,
}

#[derive(Debug, Default)]
pub struct Player {
    held: Option<TabId>,
}

impl Player {
    pub fn shown(&self, candidates: &[Candidate], active: Option<TabId>) -> Option<TabId> {
        let mut background = candidates
            .iter()
            .filter(|candidate| Some(candidate.tab) != active);
        let held = background
            .clone()
            .find(|candidate| Some(candidate.tab) == self.held);
        let playing = background.find(|candidate| candidate.playing);
        match (held, playing) {
            (Some(held), _) if held.playing => Some(held.tab),
            (_, Some(playing)) => Some(playing.tab),
            (held, None) => held.map(|held| held.tab),
        }
    }

    pub fn hold(&mut self, tab: TabId) {
        self.held = Some(tab);
    }

    pub fn release(&mut self) {
        self.held = None;
    }

    pub fn activated(&mut self, tab: TabId) {
        if self.held == Some(tab) {
            self.release();
        }
    }
}
