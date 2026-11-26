use std::{ffi::OsString, fs, path::PathBuf};

use magpie::{
    download::State,
    stream_download::{Exit, Job, Line, Progress, Tool, Toolbox},
};

const PAGE: &str = "https://video.example.com/watch?v=abc123";

fn progress_line(received: &str, total: &str, estimate: &str, file: &str) -> String {
    format!("magpie-progress|{received}|{total}|{estimate}|{file}")
}

#[test]
fn progress_lines_read_bytes_total_and_file() {
    let file = r"C:\Videos\Cats | Dogs.f137.mp4";
    let parsed = |total, estimate| Line::parse(&progress_line("2048", total, estimate, file));
    let expected = |total_bytes| {
        Some(Line::Progress(Progress {
            received_bytes: 2048,
            total_bytes,
            file: PathBuf::from(file),
        }))
    };
    assert_eq!(parsed("8192", "9000.5"), expected(Some(8192)));
    assert_eq!(parsed("NA", "9000.5"), expected(Some(9000)));
    assert_eq!(parsed("NA", "NA"), expected(None));
    assert_eq!(
        Line::parse(&progress_line("NA", "NA", "NA", file)),
        Some(Line::Progress(Progress {
            received_bytes: 0,
            total_bytes: None,
            file: PathBuf::from(file),
        }))
    );
}

#[test]
fn saved_paths_and_errors_are_recognised_and_other_output_is_ignored() {
    assert_eq!(
        Line::parse("magpie-saved|C:\\Videos\\Clip.mp4\r"),
        Some(Line::Saved(PathBuf::from(r"C:\Videos\Clip.mp4")))
    );
    assert_eq!(
        Line::parse("ERROR: [youtube] abc123: Video unavailable"),
        Some(Line::Failure("Video unavailable".to_owned()))
    );
    assert_eq!(
        Line::parse("ERROR: [Netflix] 81: This video is DRM protected"),
        Some(Line::Failure(
            "This video is DRM-protected, so it can't be saved.".to_owned()
        ))
    );
    assert_eq!(
        Line::parse("ERROR: Unsupported URL: https://example.com/"),
        Some(Line::Failure(
            "Magpie couldn't find a video on this page.".to_owned()
        ))
    );
    assert_eq!(Line::parse("WARNING: something minor"), None);
    assert_eq!(Line::parse("magpie-progress|12|NA"), None);
}

#[test]
fn a_job_shows_preparation_then_progress_then_the_saved_file() {
    let folder = tempfile::tempdir().unwrap();
    let saved = folder.path().join("Clip.mp4");
    fs::write(&saved, [0; 5000]).unwrap();
    let mut job = Job::new(PAGE);
    job.preparing(100, Some(400));
    assert_eq!(job.download().status(), "Preparing video downloader…");
    assert_eq!(job.download().progress(), Some(0.25));

    job.read(&progress_line(
        "1024",
        "4096",
        "NA",
        &saved.to_string_lossy(),
    ));
    assert_eq!(job.download().status(), "1.0 KB of 4.0 KB");
    assert_eq!(job.download().name(), "Clip.mp4");

    job.read(&format!("magpie-saved|{}", saved.display()));
    job.finish(Exit::Succeeded);
    job.finish(Exit::Cancelled);
    let download = job.download();
    assert_eq!(download.state, State::Completed);
    assert_eq!(download.path, saved);
    assert_eq!(download.status(), "Done · 4.9 KB");
}

#[test]
fn failures_explain_themselves_and_a_clean_exit_without_a_file_is_a_failure() {
    let mut failed = Job::new(PAGE);
    failed.read("ERROR: Unsupported URL: https://example.com/");
    failed.finish(Exit::Failed);
    assert_eq!(failed.download().state, State::Interrupted);
    assert_eq!(
        failed.download().status(),
        "Magpie couldn't find a video on this page."
    );

    let mut silent = Job::new(PAGE);
    silent.finish(Exit::Succeeded);
    assert_eq!(silent.download().state, State::Interrupted);
}

#[test]
fn cancelling_is_final_and_cleans_only_partial_files() {
    let folder = tempfile::tempdir().unwrap();
    let video = folder.path().join("Clip.f137.mp4");
    let leftovers = [
        "Clip.f137.mp4.part",
        "Clip.f137.mp4.part-Frag3",
        "Clip.f137.mp4.ytdl",
    ];
    let kept = ["Clip.f140.m4a", "Clip.f137.mp4.notes", "Other.mp4.part"];
    leftovers
        .iter()
        .chain(&kept)
        .for_each(|name| fs::write(folder.path().join(name), b"x").unwrap());

    let mut job = Job::new(PAGE);
    job.read(&progress_line("10", "NA", "NA", &video.to_string_lossy()));
    job.finish(Exit::Cancelled);
    job.finish(Exit::Failed);
    assert_eq!(job.download().state, State::Cancelled);
    assert_eq!(job.download().status(), "Cancelled");

    job.clean();
    let remaining = |name: &&str| folder.path().join(name).exists();
    assert!(!leftovers.iter().any(remaining));
    assert!(kept.iter().all(remaining));
}

#[test]
fn tools_are_found_in_the_magpie_folder_before_the_path() {
    let local = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    fs::write(local.path().join("yt-dlp.exe"), b"").unwrap();
    fs::write(second.path().join("yt-dlp.exe"), b"").unwrap();
    let search_path: OsString = std::env::join_paths([first.path(), second.path()]).unwrap();
    let toolbox = Toolbox::new(local.path().to_owned(), search_path);
    assert_eq!(
        toolbox.find(Tool::YtDlp),
        Some(local.path().join("yt-dlp.exe"))
    );
    assert_eq!(toolbox.find(Tool::Ffmpeg), None);

    fs::write(second.path().join("ffmpeg.exe"), b"").unwrap();
    assert_eq!(
        toolbox.find(Tool::Ffmpeg),
        Some(second.path().join("ffmpeg.exe"))
    );

    let installed = local.path().join("ffmpeg");
    fs::create_dir(&installed).unwrap();
    fs::write(installed.join("ffmpeg.exe"), b"").unwrap();
    assert_eq!(
        toolbox.find(Tool::Ffmpeg),
        Some(installed.join("ffmpeg.exe"))
    );
}
