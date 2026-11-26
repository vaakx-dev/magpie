mod job;
mod output;
mod process;
mod tool;

use std::{
    path::{Path, PathBuf},
    process::Child,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    thread,
};

pub use job::{Exit, Job};
pub use output::{Line, Progress};
pub use tool::{Tool, Toolbox, Tools};

use crate::download::Download;
use process::Request;

type Listener = dyn Fn(&Download) + Send + Sync;

struct Shared {
    job: Mutex<Job>,
    running: Mutex<Option<u32>>,
    on_progress: Box<Listener>,
}

#[derive(Clone)]
pub struct Stream(Arc<Shared>);

const THREAD_NAME: &str = "magpie-stream-download";

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Shared {
    fn update(&self, change: impl FnOnce(&mut Job)) {
        let download = {
            let mut job = locked(&self.job);
            change(&mut job);
            job.download().clone()
        };
        (self.on_progress)(&download);
    }

    fn active(&self) -> bool {
        locked(&self.job).active()
    }

    fn run(&self, page_url: &str, target_dir: &Path) {
        let prepared = Toolbox::system().prepare(|received_bytes, total_bytes| {
            self.update(|job| job.preparing(received_bytes, total_bytes));
        });
        match prepared {
            Ok(tools) => self.fetch(&tools, page_url, target_dir),
            Err(error) => self
                .update(|job| job.fail(format!("Couldn't set up the video downloader: {error}"))),
        }
    }

    fn fetch(&self, tools: &Tools, page_url: &str, target_dir: &Path) {
        if !self.active() {
            return;
        }
        let request = Request {
            tools,
            page_url,
            target_dir,
        };
        let mut child = match process::spawn(&request) {
            Ok(child) => child,
            Err(error) => {
                return self.update(|job| {
                    job.fail(format!("Couldn't start the video downloader: {error}"))
                });
            }
        };
        let lines = process::lines(&mut child);
        *locked(&self.running) = Some(child.id());
        if !self.active() {
            self.kill();
        }
        lines
            .iter()
            .for_each(|line| self.update(|job| job.read(&line)));
        self.settle(child);
    }

    fn settle(&self, mut child: Child) {
        locked(&self.running).take();
        let exit = match child.wait() {
            Ok(status) if status.success() => Exit::Succeeded,
            _ => Exit::Failed,
        };
        self.update(|job| {
            job.finish(exit);
            job.clean();
        });
    }

    fn kill(&self) {
        if let Some(process_id) = *locked(&self.running) {
            process::kill_tree(process_id);
        }
    }
}

impl Stream {
    pub fn download(&self) -> Download {
        locked(&self.0.job).download().clone()
    }

    pub fn cancel(&self) {
        self.0.update(|job| job.finish(Exit::Cancelled));
        self.0.kill();
    }
}

pub fn download_stream(
    page_url: &str,
    target_dir: &Path,
    on_progress: impl Fn(&Download) + Send + Sync + 'static,
) -> Stream {
    let shared = Arc::new(Shared {
        job: Mutex::new(Job::new(page_url)),
        running: Mutex::new(None),
        on_progress: Box::new(on_progress),
    });
    let worker = Arc::clone(&shared);
    let page_url = page_url.to_owned();
    let target_dir: PathBuf = target_dir.to_owned();
    let started = thread::Builder::new()
        .name(THREAD_NAME.to_owned())
        .spawn(move || worker.run(&page_url, &target_dir));
    if let Err(error) = started {
        shared.update(|job| job.fail(format!("Couldn't start the video downloader: {error}")));
    }
    Stream(shared)
}
