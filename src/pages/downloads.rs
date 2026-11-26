use std::borrow::Borrow;

use serde::Serialize;
use serde_json::{Value, json};

use crate::download::{Download, State, shelf, size};

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Active,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Serialize)]
struct Entry<'a> {
    name: String,
    url: &'a str,
    path: String,
    status: Status,
    pausable: bool,
    progress: Option<f32>,
    readout: String,
    size: String,
}

fn status(state: State) -> Status {
    match state {
        State::InProgress => Status::Active,
        State::Paused => Status::Paused,
        State::Completed => Status::Completed,
        State::Interrupted => Status::Failed,
        State::Cancelled => Status::Cancelled,
    }
}

fn entry(download: &Download) -> Entry<'_> {
    Entry {
        name: download.name(),
        url: &download.url,
        path: download.path.to_string_lossy().into_owned(),
        status: status(download.state),
        pausable: download.pausable,
        progress: download.progress(),
        readout: download.status(),
        size: size(download.total_bytes.unwrap_or(download.received_bytes)),
    }
}

pub fn downloads_payload<T: Borrow<Download>>(downloads: &[T]) -> Value {
    let entries: Vec<Entry> = shelf(downloads, "")
        .into_iter()
        .map(|download| entry(download.borrow()))
        .collect();
    json!({ "downloads": entries })
}
