use std::collections::HashSet;

use rusqlite::{Connection, Row};

use super::{Failure, importable};
use crate::storage::clock::Epoch;
use crate::storage::history::{MAX_VISITS, Visit};

pub struct Table {
    pub query: &'static str,
    pub epoch: fn(i64) -> Epoch,
}

fn count(raw: i64) -> u32 {
    u32::try_from(raw.max(0)).unwrap_or(u32::MAX)
}

fn visit(row: &Row, table: &Table) -> rusqlite::Result<Visit> {
    Ok(Visit {
        url: row.get(0)?,
        title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
        count: count(row.get(2)?),
        last_visit: (table.epoch)(row.get(3)?),
    })
}

pub fn recent(connection: &Connection, table: &Table) -> Result<Vec<Visit>, Failure> {
    let mut statement = connection.prepare(table.query)?;
    let limit = i64::try_from(MAX_VISITS)?;
    let rows = statement.query_map([limit], |row| visit(row, table))?;
    let newest_first = rows.collect::<rusqlite::Result<Vec<Visit>>>()?;
    let mut seen = HashSet::new();
    Ok(newest_first
        .into_iter()
        .filter(|visit| importable(&visit.url))
        .filter(|visit| seen.insert(visit.url.clone()))
        .collect())
}
