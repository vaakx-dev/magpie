use std::{
    borrow::Borrow,
    fmt,
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    InProgress,
    Paused,
    Completed,
    Interrupted,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Speed {
    pub bytes_per_sec: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Download {
    pub url: String,
    pub path: PathBuf,
    pub received_bytes: u64,
    pub total_bytes: Option<u64>,
    pub state: State,
    pub pausable: bool,
    pub speed: Option<Speed>,
    pub note: Option<String>,
}

#[derive(Clone, Copy)]
struct Sample {
    bytes: u64,
    at: Instant,
}

#[derive(Default)]
pub struct Meter {
    last: Option<Sample>,
    bytes_per_sec: Option<f64>,
}

const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
const UNIT_BYTES: u64 = 1024;
const UNNAMED: &str = "download";
const SEPARATOR: &str = " · ";
const SECS_PER_MINUTE: u64 = 60;
const SECS_PER_HOUR: u64 = 60 * SECS_PER_MINUTE;
const SAMPLE_INTERVAL: Duration = Duration::from_millis(250);
const SMOOTHING_SECS: f64 = 2.0;

pub fn size(bytes: u64) -> String {
    let exponent = (1..=UNITS.len() as u32)
        .take_while(|power| bytes >= UNIT_BYTES.pow(*power))
        .count();
    match exponent {
        0 => format!("{bytes} B"),
        _ => {
            let scaled = bytes as f64 / UNIT_BYTES.pow(exponent as u32) as f64;
            format!("{scaled:.1} {}", UNITS[exponent - 1])
        }
    }
}

pub fn time_left(remaining: Duration) -> String {
    let secs = remaining.as_secs();
    let (hours, minutes) = (secs / SECS_PER_HOUR, secs % SECS_PER_HOUR / SECS_PER_MINUTE);
    let seconds = secs % SECS_PER_MINUTE;
    match (hours, minutes) {
        (0, 0) => format!("{seconds}s left"),
        (0, _) => format!("{minutes}m {seconds}s left"),
        _ => format!("{hours}h {minutes}m left"),
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}/s", size(self.bytes_per_sec))
    }
}

impl Download {
    pub fn name(&self) -> String {
        let from_path = self.path.file_name().map(|name| name.to_string_lossy());
        let from_url = || {
            let last = self.url.rsplit('/').find(|part| !part.is_empty());
            last.unwrap_or(UNNAMED).into()
        };
        from_path.unwrap_or_else(from_url).into_owned()
    }

    pub fn progress(&self) -> Option<f32> {
        let total = self.total_bytes.filter(|total| *total > 0)?;
        Some((self.received_bytes as f64 / total as f64).min(1.0) as f32)
    }

    pub fn eta(&self) -> Option<Duration> {
        let speed = self.speed.filter(|speed| speed.bytes_per_sec > 0)?;
        let remaining = self.total_bytes?.saturating_sub(self.received_bytes);
        Some(Duration::from_secs(remaining.div_ceil(speed.bytes_per_sec)))
    }

    fn amount(&self) -> String {
        let received = size(self.received_bytes);
        match self.total_bytes {
            Some(total) => format!("{received} of {}", size(total)),
            None => received,
        }
    }

    fn transfer(&self) -> String {
        let speed = self.speed.map(|speed| speed.to_string());
        let eta = self.eta().map(time_left);
        [Some(self.amount()), speed, eta]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(SEPARATOR)
    }

    pub fn status(&self) -> String {
        if let Some(note) = &self.note {
            return note.clone();
        }
        match self.state {
            State::InProgress => self.transfer(),
            State::Paused => format!("Paused{SEPARATOR}{}", self.amount()),
            State::Completed => format!("Done{SEPARATOR}{}", size(self.received_bytes)),
            State::Interrupted => "Failed".to_owned(),
            State::Cancelled => "Cancelled".to_owned(),
        }
    }

    pub fn active(&self) -> bool {
        matches!(self.state, State::InProgress | State::Paused)
    }

    fn named(&self, query: &str) -> bool {
        self.name().to_lowercase().contains(&query.to_lowercase())
    }
}

impl Sample {
    fn bytes_per_sec_since(&self, earlier: Sample) -> f64 {
        let gained = self.bytes.saturating_sub(earlier.bytes) as f64;
        gained / self.at.duration_since(earlier.at).as_secs_f64()
    }
}

impl Meter {
    pub fn measure(&mut self, download: Download, now: Instant) -> Download {
        if download.state != State::InProgress {
            *self = Self::default();
            return Download {
                speed: None,
                ..download
            };
        }
        self.sample(Sample {
            bytes: download.received_bytes,
            at: now,
        });
        Download {
            speed: self.speed(),
            ..download
        }
    }

    fn sample(&mut self, sample: Sample) {
        match self.last {
            Some(last) if sample.at.duration_since(last.at) < SAMPLE_INTERVAL => {}
            Some(last) => {
                self.blend(last, sample);
                self.last = Some(sample);
            }
            None => self.last = Some(sample),
        }
    }

    fn blend(&mut self, last: Sample, sample: Sample) {
        let latest = sample.bytes_per_sec_since(last);
        let elapsed_secs = sample.at.duration_since(last.at).as_secs_f64();
        let weight = 1.0 - (-elapsed_secs / SMOOTHING_SECS).exp();
        let smoothed = match self.bytes_per_sec {
            Some(previous) => previous + weight * (latest - previous),
            None => latest,
        };
        self.bytes_per_sec = Some(smoothed);
    }

    fn speed(&self) -> Option<Speed> {
        let bytes_per_sec = self.bytes_per_sec?.round() as u64;
        Some(Speed { bytes_per_sec })
    }
}

pub fn shelf<'a, T: Borrow<Download>>(entries: &'a [T], query: &str) -> Vec<&'a T> {
    let (active, finished): (Vec<&T>, Vec<&T>) = entries
        .iter()
        .filter(|entry| T::borrow(entry).named(query))
        .partition(|entry| T::borrow(entry).active());
    active.into_iter().chain(finished).collect()
}
