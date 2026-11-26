use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::clock::Epoch;

const RECENCY_WEIGHTS: [(Duration, f64); 4] = [
    (Duration::from_secs(4 * 24 * 60 * 60), 100.0),
    (Duration::from_secs(14 * 24 * 60 * 60), 70.0),
    (Duration::from_secs(31 * 24 * 60 * 60), 50.0),
    (Duration::from_secs(90 * 24 * 60 * 60), 30.0),
];
const OLD_WEIGHT: f64 = 10.0;
pub const MAX_VISITS: usize = 20_000;
const PRUNED_VISITS: usize = MAX_VISITS * 9 / 10;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Visit {
    pub url: String,
    pub title: String,
    pub last_visit: Epoch,
    pub count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct History {
    visits: HashMap<String, Visit>,
}

fn recency_weight(age: Duration) -> f64 {
    RECENCY_WEIGHTS
        .iter()
        .find(|(limit, _)| age < *limit)
        .map_or(OLD_WEIGHT, |(_, weight)| *weight)
}

impl Visit {
    pub fn frecency(&self, now: Epoch) -> f64 {
        f64::from(self.count) * recency_weight(now.since(self.last_visit))
    }
}

impl History {
    pub fn record(&mut self, url: &str, title: &str, now: Epoch) {
        let visit = self.visits.entry(url.to_owned()).or_insert_with(|| Visit {
            url: url.to_owned(),
            title: String::new(),
            last_visit: now,
            count: 0,
        });
        visit.count = visit.count.saturating_add(1);
        visit.last_visit = now;
        if !title.is_empty() {
            visit.title = title.to_owned();
        }
        self.trim(now);
    }

    pub fn merge(&mut self, visits: impl IntoIterator<Item = Visit>, now: Epoch) {
        for visit in visits {
            self.absorb(visit);
        }
        self.trim(now);
    }

    fn absorb(&mut self, visit: Visit) {
        let Some(known) = self.visits.get_mut(&visit.url) else {
            self.visits.insert(visit.url.clone(), visit);
            return;
        };
        known.count = known.count.max(visit.count);
        known.last_visit = known.last_visit.max(visit.last_visit);
        if known.title.is_empty() {
            known.title = visit.title;
        }
    }

    fn trim(&mut self, now: Epoch) {
        if self.visits.len() > MAX_VISITS {
            self.prune(now);
        }
    }

    fn prune(&mut self, now: Epoch) {
        let mut ranked: Vec<(f64, String)> = self
            .visits
            .values()
            .map(|visit| (visit.frecency(now), visit.url.clone()))
            .collect();
        ranked.sort_by(|left, right| right.0.total_cmp(&left.0));
        for (_, url) in ranked.split_off(PRUNED_VISITS) {
            self.visits.remove(&url);
        }
    }

    pub fn forget(&mut self, url: &str) {
        self.visits.remove(url);
    }

    pub fn clear(&mut self) {
        self.visits.clear();
    }

    pub fn visit(&self, url: &str) -> Option<&Visit> {
        self.visits.get(url)
    }

    pub fn visits(&self) -> impl Iterator<Item = &Visit> {
        self.visits.values()
    }

    pub fn recent(&self, count: usize) -> Vec<&Visit> {
        let newest = |visit: &&Visit| Reverse(visit.last_visit);
        let mut visits: Vec<&Visit> = self.visits.values().collect();
        if count < visits.len() {
            visits.select_nth_unstable_by_key(count, newest);
            visits.truncate(count);
        }
        visits.sort_by_key(newest);
        visits
    }
}
