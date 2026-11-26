use std::{
    io::{self, BufRead, BufReader, Read},
    os::windows::process::CommandExt,
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use super::{output, tool::Tools};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const OUTPUT_NAME: &str = "%(title).120B.%(ext)s";
const MERGED_FORMAT: &str = "bv*[ext=mp4]+ba[ext=m4a]/bv*+ba/b";
const SINGLE_FILE_FORMAT: &str = "b[ext=mp4]/b";
const FIXED_ARGUMENTS: [&str; 5] = [
    "--newline",
    "--progress",
    "--no-playlist",
    "--no-mtime",
    "--no-simulate",
];

pub struct Request<'a> {
    pub tools: &'a Tools,
    pub page_url: &'a str,
    pub target_dir: &'a Path,
}

fn format(tools: &Tools) -> &'static str {
    match tools.ffmpeg {
        Some(_) => MERGED_FORMAT,
        None => SINGLE_FILE_FORMAT,
    }
}

fn command(request: &Request) -> Command {
    let mut command = Command::new(&request.tools.yt_dlp);
    command
        .args(FIXED_ARGUMENTS)
        .arg("--progress-template")
        .arg(output::progress_template())
        .arg("--print")
        .arg(output::saved_template())
        .arg("-f")
        .arg(format(request.tools))
        .arg("-o")
        .arg(request.target_dir.join(OUTPUT_NAME));
    if let Some(ffmpeg) = &request.tools.ffmpeg {
        command.arg("--ffmpeg-location").arg(ffmpeg);
    }
    command
        .arg("--")
        .arg(request.page_url)
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW);
    command
}

pub fn spawn(request: &Request) -> io::Result<Child> {
    command(request).spawn()
}

fn forward(pipe: impl Read + Send + 'static, lines: Sender<String>) {
    thread::spawn(move || {
        BufReader::new(pipe)
            .split(b'\n')
            .map_while(Result::ok)
            .map(|line| String::from_utf8_lossy(&line).into_owned())
            .try_for_each(|line| lines.send(line))
            .ok();
    });
}

pub fn lines(child: &mut Child) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    if let Some(stdout) = child.stdout.take() {
        forward(stdout, sender.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        forward(stderr, sender.clone());
    }
    receiver
}

pub fn kill_tree(process_id: u32) {
    let killed = Command::new("taskkill")
        .args(["/PID", &process_id.to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    if let Err(error) = killed {
        eprintln!("magpie: could not stop the video downloader: {error}");
    }
}
