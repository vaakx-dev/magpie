use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::clock::Epoch;
use super::query::Query;
use super::session::{Session, Space, SpaceId, Tab, TabId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Archived {
    pub tab: Tab,
    pub space: SpaceId,
    pub archived: Epoch,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Archive {
    tabs: Vec<Archived>,
}

fn idle_tabs(space: &mut Space, now: Epoch, idle: Duration) -> Vec<Tab> {
    let active = space.active;
    let (stale, fresh) = space
        .today
        .drain(..)
        .partition(|tab| Some(tab.id) != active && now.since(tab.last_active) >= idle);
    space.today = fresh;
    stale
}

impl Archive {
    pub fn sweep(&mut self, session: &mut Session, now: Epoch, idle: Duration) -> usize {
        let before = self.tabs.len();
        for space in &mut session.spaces {
            let space_id = space.id;
            let archived = idle_tabs(space, now, idle).into_iter().map(|tab| Archived {
                tab,
                space: space_id,
                archived: now,
            });
            self.tabs.extend(archived);
        }
        self.tabs.len() - before
    }

    pub fn list(&self) -> impl Iterator<Item = &Archived> {
        self.tabs.iter().rev()
    }

    pub fn search(&self, query: &str) -> Vec<&Archived> {
        let query = Query::new(query);
        self.list()
            .filter(|entry| query.matches(entry.tab.title(), entry.tab.url()))
            .collect()
    }

    pub fn restore(&mut self, id: TabId, session: &mut Session, now: Epoch) -> Option<SpaceId> {
        let index = self.tabs.iter().position(|entry| entry.tab.id == id)?;
        let space = session.home(self.tabs[index].space)?;
        let entry = self.tabs.remove(index);
        Some(space.reopen(entry.tab, now))
    }
}
