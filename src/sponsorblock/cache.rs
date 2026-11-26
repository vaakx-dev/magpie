use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use super::{category::Categories, segment::Segment, video::VideoId};

type Key = (VideoId, Categories);

struct Entry {
    segments: Vec<Segment>,
    stored: Instant,
    used: u64,
}

pub struct Cache {
    entries: HashMap<Key, Entry>,
    capacity: usize,
    lifetime: Duration,
    clock: u64,
}

impl Cache {
    pub fn new(capacity: usize, lifetime: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            capacity,
            lifetime,
            clock: 0,
        }
    }

    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    pub fn get(&mut self, key: &Key, now: Instant) -> Option<Vec<Segment>> {
        let used = self.tick();
        let lifetime = self.lifetime;
        let entry = self
            .entries
            .get_mut(key)
            .filter(|entry| now.duration_since(entry.stored) < lifetime)?;
        entry.used = used;
        Some(entry.segments.clone())
    }

    fn evict_least_recent(&mut self) {
        let oldest = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.used)
            .map(|(key, _)| key.clone());
        if let Some(key) = oldest {
            self.entries.remove(&key);
        }
    }

    pub fn insert(&mut self, key: Key, segments: Vec<Segment>, now: Instant) {
        if !self.entries.contains_key(&key) && self.entries.len() >= self.capacity {
            self.evict_least_recent();
        }
        let used = self.tick();
        self.entries.insert(
            key,
            Entry {
                segments,
                stored: now,
                used,
            },
        );
    }
}
