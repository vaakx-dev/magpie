use std::time::Duration;

use super::{Visibility, audio::Audio};

pub const IDLE_BEFORE_SLEEP: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Copy, Debug)]
pub struct Activity {
    pub visibility: Visibility,
    pub audio: Audio,
    pub unchanged_for: Duration,
}

impl Activity {
    pub fn drowsy(&self, idle: Duration) -> bool {
        self.visibility == Visibility::Hidden && !self.audio.playing && self.unchanged_for >= idle
    }
}
