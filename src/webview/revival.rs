use std::time::{Duration, Instant};

use super::Visibility;

const CRASH_LOOP: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Revive {
    Now,
    WhenShown,
}

#[derive(Debug, Default)]
pub struct Revival {
    last_end: Option<Instant>,
}

impl Revival {
    pub fn ended(&mut self, at: Instant, visibility: Visibility) -> Revive {
        let looping = self
            .last_end
            .is_some_and(|last| at.duration_since(last) < CRASH_LOOP);
        self.last_end = Some(at);
        match (visibility, looping) {
            (Visibility::Hidden, _) | (_, true) => Revive::WhenShown,
            (Visibility::Shown | Visibility::Focused, false) => Revive::Now,
        }
    }
}
