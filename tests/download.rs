use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use magpie::download::{Download, Meter, Speed, State, shelf, size, time_left};

fn download(received_bytes: u64, total_bytes: Option<u64>, state: State) -> Download {
    Download {
        url: "https://files.example.com/releases/tool.zip".to_owned(),
        path: PathBuf::from(r"C:\Users\someone\Downloads\tool (1).zip"),
        received_bytes,
        total_bytes,
        state,
        pausable: true,
        speed: None,
        note: None,
    }
}

const MEGABYTE: u64 = 1024 * 1024;

#[test]
fn sizes_use_binary_units_with_one_decimal() {
    assert_eq!(size(0), "0 B");
    assert_eq!(size(1023), "1023 B");
    assert_eq!(size(1024), "1.0 KB");
    assert_eq!(size(1536), "1.5 KB");
    assert_eq!(size(5 * 1024 * 1024), "5.0 MB");
    assert_eq!(size(3 * 1024 * 1024 * 1024 / 2), "1.5 GB");
}

#[test]
fn progress_is_the_received_share_of_a_known_total() {
    assert_eq!(
        download(256, Some(1024), State::InProgress).progress(),
        Some(0.25)
    );
    assert_eq!(
        download(2048, Some(1024), State::InProgress).progress(),
        Some(1.0)
    );
    assert_eq!(download(256, None, State::InProgress).progress(), None);
    assert_eq!(download(0, Some(0), State::InProgress).progress(), None);
}

#[test]
fn status_describes_each_state() {
    let megabyte = 1024 * 1024;
    let partial = download(megabyte, Some(4 * megabyte), State::InProgress);
    assert_eq!(partial.status(), "1.0 MB of 4.0 MB");
    assert_eq!(download(2048, None, State::InProgress).status(), "2.0 KB");
    let done = download(4 * megabyte, Some(4 * megabyte), State::Completed);
    assert_eq!(done.status(), "Done · 4.0 MB");
    assert_eq!(download(0, None, State::Interrupted).status(), "Failed");
    assert_eq!(download(0, None, State::Cancelled).status(), "Cancelled");
    let paused = download(megabyte, Some(4 * megabyte), State::Paused);
    assert_eq!(paused.status(), "Paused · 1.0 MB of 4.0 MB");
}

#[test]
fn transfers_show_speed_and_time_left() {
    let fast = Download {
        speed: Some(Speed {
            bytes_per_sec: MEGABYTE / 2,
        }),
        ..download(MEGABYTE, Some(4 * MEGABYTE), State::InProgress)
    };
    assert_eq!(fast.status(), "1.0 MB of 4.0 MB · 512.0 KB/s · 6s left");
    let open_ended = Download {
        speed: Some(Speed { bytes_per_sec: 300 }),
        ..download(MEGABYTE, None, State::InProgress)
    };
    assert_eq!(open_ended.status(), "1.0 MB · 300 B/s");
    let stalled = Download {
        speed: Some(Speed { bytes_per_sec: 0 }),
        ..download(MEGABYTE, Some(4 * MEGABYTE), State::InProgress)
    };
    assert_eq!(stalled.eta(), None);
}

#[test]
fn time_left_uses_the_two_largest_units() {
    assert_eq!(time_left(Duration::from_secs(0)), "0s left");
    assert_eq!(time_left(Duration::from_secs(12)), "12s left");
    assert_eq!(time_left(Duration::from_secs(135)), "2m 15s left");
    assert_eq!(time_left(Duration::from_secs(3600)), "1h 0m left");
    assert_eq!(
        time_left(Duration::from_secs(5 * 3600 + 7 * 60 + 9)),
        "5h 7m left"
    );
}

#[test]
fn meter_smooths_speed_and_resets_while_paused() {
    let start = Instant::now();
    let mut meter = Meter::default();
    let measure = |meter: &mut Meter, millis: u64, received_bytes: u64, state: State| {
        let snapshot = download(received_bytes, Some(100 * MEGABYTE), state);
        let now = start + Duration::from_millis(millis);
        meter
            .measure(snapshot, now)
            .speed
            .map(|speed| speed.bytes_per_sec)
    };
    assert_eq!(measure(&mut meter, 0, 0, State::InProgress), None);
    assert_eq!(
        measure(&mut meter, 500, MEGABYTE, State::InProgress),
        Some(2 * MEGABYTE)
    );
    assert_eq!(
        measure(&mut meter, 1000, 2 * MEGABYTE, State::InProgress),
        Some(2 * MEGABYTE)
    );
    assert_eq!(
        measure(&mut meter, 1100, 50 * MEGABYTE, State::InProgress),
        Some(2 * MEGABYTE)
    );
    let burst = measure(&mut meter, 2000, 50 * MEGABYTE, State::InProgress).unwrap();
    assert!(burst > 2 * MEGABYTE && burst < 48 * MEGABYTE);
    assert_eq!(
        measure(&mut meter, 2500, 50 * MEGABYTE, State::Paused),
        None
    );
    assert_eq!(
        measure(&mut meter, 60_000, 50 * MEGABYTE, State::InProgress),
        None
    );
    assert_eq!(
        measure(&mut meter, 61_000, 51 * MEGABYTE, State::InProgress),
        Some(MEGABYTE)
    );
}

#[test]
fn name_comes_from_the_saved_file_and_falls_back_to_the_url() {
    assert_eq!(download(0, None, State::InProgress).name(), "tool (1).zip");
    let unsaved = Download {
        path: PathBuf::new(),
        ..download(0, None, State::InProgress)
    };
    assert_eq!(unsaved.name(), "tool.zip");
}

#[test]
fn shelf_lists_matching_downloads_with_active_ones_first() {
    let named = |name: &str, state: State| Download {
        path: PathBuf::from(format!(r"C:\Users\someone\Downloads\{name}")),
        ..download(0, None, state)
    };
    let entries = [
        named("Report-final.pdf", State::Completed),
        named("photos.zip", State::InProgress),
        named("report-draft.pdf", State::Cancelled),
        named("big-report.iso", State::InProgress),
    ];
    let names = |query: &str| -> Vec<String> {
        shelf(&entries, query)
            .into_iter()
            .map(Download::name)
            .collect()
    };
    assert_eq!(
        names(""),
        [
            "photos.zip",
            "big-report.iso",
            "Report-final.pdf",
            "report-draft.pdf"
        ]
    );
    assert_eq!(
        names("REPORT"),
        ["big-report.iso", "Report-final.pdf", "report-draft.pdf"]
    );
    assert!(names("movie").is_empty());
}
