use std::{
    env,
    ffi::OsString,
    fs,
    io::{self, Read, Write},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, PoisonError},
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    YtDlp,
    Ffmpeg,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tools {
    pub yt_dlp: PathBuf,
    pub ffmpeg: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct Toolbox {
    folder: PathBuf,
    search_path: OsString,
}

const YT_DLP_RELEASE: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
const FFMPEG_RELEASE: &str = "https://github.com/yt-dlp/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl-shared.zip";
const FFMPEG_FOLDER: &str = "ffmpeg";
const FFMPEG_ARCHIVE_BINARIES: &str = "bin";
const PARTIAL_SUFFIX: &str = ".tmp";
const ARCHIVE_SUFFIX: &str = ".zip.tmp";
const WINDOWS_TAR: &str = r"System32\tar.exe";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const FETCH_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const CHUNK_BYTES: usize = 64 * 1024;

static FETCHING: Mutex<()> = Mutex::new(());

fn local_folder() -> PathBuf {
    let base = env::var_os("LOCALAPPDATA").map_or_else(env::temp_dir, PathBuf::from);
    base.join("Magpie").join("tools")
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn save(
    address: &str,
    destination: &Path,
    on_fetch: &mut impl FnMut(u64, Option<u64>),
) -> io::Result<()> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(FETCH_TIMEOUT))
        .build()
        .into();
    let mut response = agent.get(address).call().map_err(io::Error::other)?;
    let total_bytes = response.body().content_length();
    let mut reader = response.body_mut().as_reader();
    let mut file = fs::File::create(destination)?;
    let mut buffer = vec![0; CHUNK_BYTES];
    let mut received_bytes = 0;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return file.sync_all();
        }
        file.write_all(&buffer[..read])?;
        received_bytes += read as u64;
        on_fetch(received_bytes, total_bytes);
    }
}

fn windows_tar() -> PathBuf {
    let root = env::var_os("SystemRoot").unwrap_or_else(|| OsString::from(r"C:\Windows"));
    PathBuf::from(root).join(WINDOWS_TAR)
}

fn extract(archive: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    let status = Command::new(windows_tar())
        .arg("-xf")
        .arg(archive)
        .arg("-C")
        .arg(destination)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "tar could not unpack ffmpeg ({status})"
        )));
    }
    Ok(())
}

fn binaries(unpacked: &Path) -> io::Result<PathBuf> {
    fs::read_dir(unpacked)?
        .flatten()
        .map(|entry| entry.path().join(FFMPEG_ARCHIVE_BINARIES))
        .find(|candidate| candidate.is_dir())
        .ok_or_else(|| io::Error::other("the ffmpeg download had no programs in it"))
}

fn discard(path: &Path) {
    let removed = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    removed.ok();
}

impl Tool {
    fn file_name(self) -> &'static str {
        match self {
            Tool::YtDlp => "yt-dlp.exe",
            Tool::Ffmpeg => "ffmpeg.exe",
        }
    }

    fn installed(self, folder: &Path) -> PathBuf {
        match self {
            Tool::YtDlp => folder.join(self.file_name()),
            Tool::Ffmpeg => folder.join(FFMPEG_FOLDER).join(self.file_name()),
        }
    }
}

impl Toolbox {
    pub fn new(folder: PathBuf, search_path: OsString) -> Self {
        Self {
            folder,
            search_path,
        }
    }

    pub fn system() -> Self {
        Self::new(local_folder(), env::var_os("PATH").unwrap_or_default())
    }

    pub fn find(&self, tool: Tool) -> Option<PathBuf> {
        let installed = tool.installed(&self.folder);
        if installed.is_file() {
            return Some(installed);
        }
        env::split_paths(&self.search_path)
            .map(|folder| folder.join(tool.file_name()))
            .find(|candidate| candidate.is_file())
    }

    pub fn prepare(&self, mut on_fetch: impl FnMut(u64, Option<u64>)) -> io::Result<Tools> {
        let _only_one_fetch = FETCHING.lock().unwrap_or_else(PoisonError::into_inner);
        let yt_dlp = match self.find(Tool::YtDlp) {
            Some(path) => path,
            None => self.install_yt_dlp(&mut on_fetch)?,
        };
        let ffmpeg = self.find(Tool::Ffmpeg).or_else(|| {
            self.install_ffmpeg(&mut on_fetch)
                .inspect_err(|error| eprintln!("magpie: could not set up ffmpeg: {error}"))
                .ok()
        });
        Ok(Tools { yt_dlp, ffmpeg })
    }

    fn install_yt_dlp(&self, on_fetch: &mut impl FnMut(u64, Option<u64>)) -> io::Result<PathBuf> {
        let target = Tool::YtDlp.installed(&self.folder);
        let partial = with_suffix(&target, PARTIAL_SUFFIX);
        fs::create_dir_all(&self.folder)?;
        let installed =
            save(YT_DLP_RELEASE, &partial, on_fetch).and_then(|_| fs::rename(&partial, &target));
        discard(&partial);
        installed.map(|_| target)
    }

    fn install_ffmpeg(&self, on_fetch: &mut impl FnMut(u64, Option<u64>)) -> io::Result<PathBuf> {
        let folder = self.folder.join(FFMPEG_FOLDER);
        let archive = with_suffix(&folder, ARCHIVE_SUFFIX);
        let unpacked = with_suffix(&folder, PARTIAL_SUFFIX);
        fs::create_dir_all(&self.folder)?;
        discard(&unpacked);
        let installed = save(FFMPEG_RELEASE, &archive, on_fetch)
            .and_then(|_| extract(&archive, &unpacked))
            .and_then(|_| binaries(&unpacked))
            .and_then(|binaries| fs::rename(binaries, &folder));
        discard(&archive);
        discard(&unpacked);
        installed.map(|_| Tool::Ffmpeg.installed(&self.folder))
    }
}
