use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use magpie_engine::file;

const DELAY: Duration = Duration::from_secs(2);

enum Request {
    Save,
    Flush(Sender<io::Result<()>>),
}

pub struct Writer {
    requests: Sender<Request>,
}

struct Worker<T> {
    path: PathBuf,
    value: Arc<RwLock<T>>,
    due: Option<Instant>,
}

fn write<T: Serialize>(path: &Path, value: &RwLock<T>) -> io::Result<()> {
    let bytes = serde_json::to_vec(&*value.read().expect("saved value lock poisoned"))
        .map_err(io::Error::other)?;
    file::replace(path, &bytes)
}

impl<T: Serialize> Worker<T> {
    fn next(&self, requests: &Receiver<Request>) -> Result<Request, RecvTimeoutError> {
        match self.due {
            Some(due) => requests.recv_timeout(due.saturating_duration_since(Instant::now())),
            None => requests.recv().map_err(|_| RecvTimeoutError::Disconnected),
        }
    }

    fn write_pending(&mut self) -> io::Result<()> {
        match self.due.take() {
            Some(_) => write(&self.path, &self.value),
            None => Ok(()),
        }
    }

    fn report_pending(&mut self) {
        if let Err(error) = self.write_pending() {
            eprintln!("magpie: could not save {}: {error}", self.path.display());
        }
    }

    fn run(mut self, requests: Receiver<Request>) {
        loop {
            match self.next(&requests) {
                Ok(Request::Save) => {
                    self.due.get_or_insert_with(|| Instant::now() + DELAY);
                }
                Ok(Request::Flush(done)) => {
                    done.send(self.write_pending()).ok();
                }
                Err(RecvTimeoutError::Timeout) => self.report_pending(),
                Err(RecvTimeoutError::Disconnected) => {
                    self.report_pending();
                    return;
                }
            }
        }
    }
}

impl Writer {
    pub fn spawn<T: Serialize + Send + Sync + 'static>(
        path: PathBuf,
        value: Arc<RwLock<T>>,
    ) -> io::Result<Self> {
        let (requests, received) = mpsc::channel();
        let worker = Worker {
            path,
            value,
            due: None,
        };
        thread::Builder::new()
            .name("magpie-writer".to_owned())
            .spawn(move || worker.run(received))?;
        Ok(Self { requests })
    }

    pub fn save(&self) {
        self.requests.send(Request::Save).ok();
    }

    pub fn save_now(&self) -> io::Result<()> {
        self.save();
        self.flush()
    }

    pub fn flush(&self) -> io::Result<()> {
        let (done, finished) = mpsc::channel();
        self.requests
            .send(Request::Flush(done))
            .map_err(|_| io::Error::other("the writer stopped"))?;
        finished
            .recv()
            .map_err(|_| io::Error::other("the writer stopped"))?
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        if let Err(error) = self.flush() {
            eprintln!("magpie: could not save on exit: {error}");
        }
    }
}
