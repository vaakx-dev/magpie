use std::collections::VecDeque;

use super::session::{SpaceId, Tab};

pub const CAPACITY: usize = 25;

#[derive(Clone, Debug, PartialEq)]
pub struct Closed {
    pub tab: Tab,
    pub space: SpaceId,
}

#[derive(Debug, Default)]
pub struct Stack {
    entries: VecDeque<Closed>,
}

impl Stack {
    pub fn push(&mut self, closed: Closed) {
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(closed);
    }

    pub fn take(&mut self, newest_index: usize) -> Option<Closed> {
        let index = self.entries.len().checked_sub(newest_index + 1)?;
        self.entries.remove(index)
    }

    pub fn drain_oldest_first(&mut self) -> impl Iterator<Item = Closed> + '_ {
        self.entries.drain(..)
    }

    pub fn newest_first(&self) -> impl Iterator<Item = &Closed> {
        self.entries.iter().rev()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
