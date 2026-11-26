use std::{
    fs,
    path::{Path, PathBuf},
};

use super::output::Line;
use crate::download::{Download, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct Job {
    download: Download,
    files: Vec<PathBuf>,
    saved: Option<PathBuf>,
    failure: Option<String>,
}

const PREPARING: &str = "Preparing video downloader…";
const NOTHING_SAVED: &str = "The video downloader stopped without saving a file.";
const LEFTOVER_PART: &str = ".part";
const LEFTOVER_STATE: &str = ".ytdl";

fn leftover(entry: &str, file: &str) -> bool {
    entry
        .strip_prefix(file)
        .is_some_and(|rest| rest.starts_with(LEFTOVER_PART) || rest == LEFTOVER_STATE)
}

fn remove_leftovers(file: &Path) {
    let (Some(folder), Some(name)) = (file.parent(), file.file_name()) else {
        return;
    };
    let name = name.to_string_lossy();
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    entries
        .flatten()
        .filter(|entry| leftover(&entry.file_name().to_string_lossy(), &name))
        .for_each(|entry| {
            if let Err(error) = fs::remove_file(entry.path()) {
                eprintln!(
                    "magpie: could not remove {}: {error}",
                    entry.path().display()
                );
            }
        });
}

fn file_size(path: &Path) -> Option<u64> {
    fs::metadata(path).map(|metadata| metadata.len()).ok()
}

impl Job {
    pub fn new(page_url: &str) -> Self {
        Self {
            download: Download {
                url: page_url.to_owned(),
                path: PathBuf::new(),
                received_bytes: 0,
                total_bytes: None,
                state: State::InProgress,
                pausable: false,
                speed: None,
                note: Some(PREPARING.to_owned()),
            },
            files: Vec::new(),
            saved: None,
            failure: None,
        }
    }

    pub fn download(&self) -> &Download {
        &self.download
    }

    pub fn active(&self) -> bool {
        self.download.active()
    }

    pub fn preparing(&mut self, received_bytes: u64, total_bytes: Option<u64>) {
        self.download.received_bytes = received_bytes;
        self.download.total_bytes = total_bytes;
    }

    pub fn read(&mut self, text: &str) {
        match Line::parse(text) {
            Some(Line::Progress(progress)) => {
                self.download.note = None;
                self.download.received_bytes = progress.received_bytes;
                self.download.total_bytes = progress.total_bytes;
                self.download.path = progress.file.clone();
                if !self.files.contains(&progress.file) {
                    self.files.push(progress.file);
                }
            }
            Some(Line::Saved(path)) => self.saved = Some(path),
            Some(Line::Failure(reason)) => self.failure = Some(reason),
            None => {}
        }
    }

    pub fn fail(&mut self, reason: String) {
        self.failure = Some(reason);
        self.finish(Exit::Failed);
    }

    fn complete(&mut self, saved: PathBuf) {
        let size = file_size(&saved).unwrap_or(self.download.received_bytes);
        self.download.received_bytes = size;
        self.download.total_bytes = Some(size);
        self.download.path = saved;
        self.download.note = None;
        self.download.state = State::Completed;
    }

    fn interrupt(&mut self) {
        let reason = self
            .failure
            .take()
            .unwrap_or_else(|| NOTHING_SAVED.to_owned());
        self.download.note = Some(reason);
        self.download.state = State::Interrupted;
    }

    fn cancel(&mut self) {
        self.download.note = None;
        self.download.state = State::Cancelled;
    }

    pub fn finish(&mut self, exit: Exit) {
        if !self.active() {
            return;
        }
        match (exit, self.saved.take()) {
            (Exit::Succeeded, Some(saved)) => self.complete(saved),
            (Exit::Succeeded | Exit::Failed, _) => self.interrupt(),
            (Exit::Cancelled, _) => self.cancel(),
        }
    }

    pub fn clean(&self) {
        if self.download.state == State::Completed {
            return;
        }
        self.files.iter().for_each(|file| remove_leftovers(file));
    }
}
