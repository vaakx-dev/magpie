use std::time::{Duration, Instant};

use crate::config::Sidebar;

const DURATION: Duration = Duration::from_millis(150);

pub struct Slide {
    target: Sidebar,
    from: f32,
    started: Instant,
}

fn openness(sidebar: Sidebar) -> f32 {
    match sidebar {
        Sidebar::Shown => 1.0,
        Sidebar::Hidden => 0.0,
    }
}

fn ease_out(progress: f32) -> f32 {
    1.0 - (1.0 - progress).powi(3)
}

impl Slide {
    pub fn new(sidebar: Sidebar) -> Self {
        Self {
            target: sidebar,
            from: openness(sidebar),
            started: Instant::now(),
        }
    }

    fn progress(&self) -> f32 {
        let elapsed = self.started.elapsed().as_secs_f32();
        (elapsed / DURATION.as_secs_f32()).min(1.0)
    }

    pub fn moving(&self) -> bool {
        self.progress() < 1.0 && self.from != openness(self.target)
    }

    pub fn openness(&self) -> f32 {
        let to = openness(self.target);
        match self.moving() {
            true => self.from + (to - self.from) * ease_out(self.progress()),
            false => to,
        }
    }

    pub fn follow(&mut self, sidebar: Sidebar) {
        if sidebar == self.target {
            return;
        }
        self.from = self.openness();
        self.target = sidebar;
        self.started = Instant::now();
    }
}
