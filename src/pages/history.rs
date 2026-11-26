use serde::Serialize;
use serde_json::{Value, json};

use super::{link::host, local_time::LocalTime};
use crate::storage::history::{History, Visit};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum Period {
    Today,
    Yesterday,
    #[serde(rename = "This Week")]
    ThisWeek,
    Older,
}

#[derive(Serialize)]
struct Entry<'a> {
    url: &'a str,
    title: &'a str,
    host: String,
    visited: u64,
}

#[derive(Serialize)]
struct Group<'a> {
    period: Period,
    visits: Vec<Entry<'a>>,
}

const WEEK_DAYS: i64 = 7;

impl Period {
    fn of(days_ago: i64) -> Self {
        match days_ago {
            ..=0 => Self::Today,
            1 => Self::Yesterday,
            2..WEEK_DAYS => Self::ThisWeek,
            _ => Self::Older,
        }
    }
}

fn entry(visit: &Visit) -> Entry<'_> {
    Entry {
        url: &visit.url,
        title: &visit.title,
        host: host(&visit.url),
        visited: visit.last_visit.0,
    }
}

pub fn history_payload(history: &History, now: LocalTime) -> Value {
    let period = |visit: &Visit| Period::of(now.days_ago(visit.last_visit));
    let visits = history.recent(usize::MAX);
    let groups: Vec<Group> = visits
        .chunk_by(|left, right| period(left) == period(right))
        .map(|chunk| Group {
            period: period(chunk[0]),
            visits: chunk.iter().copied().map(entry).collect(),
        })
        .collect();
    json!({ "groups": groups })
}
