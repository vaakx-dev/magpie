use std::{
    env, fs, io,
    os::windows::process::CommandExt,
    path::Path,
    process::{self, Command},
};

use magpie_engine::update::Package;

use super::FAILURE_FILE;

pub struct Handoff<'a> {
    pub process_id: u32,
    pub package: Package,
    pub update: &'a Path,
    pub target: &'a Path,
}

const SCRIPT: &str = include_str!("apply.ps1");
const SCRIPT_FILE: &str = "apply.ps1";
const MSI_EXTENSION: &str = "msi";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn kind(update: &Path, package: Package) -> io::Result<&'static str> {
    let msi = update
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(MSI_EXTENSION));
    match (package, msi) {
        (Package::Installer, true) => Ok("Msi"),
        (Package::Installer, false) => Ok("Setup"),
        (Package::Portable, _) => Ok("Portable"),
        (Package::Apk, _) => Err(io::ErrorKind::Unsupported.into()),
    }
}

pub fn command(handoff: &Handoff) -> io::Result<Command> {
    let kind = kind(handoff.update, handoff.package)?;
    let script = handoff.update.with_file_name(SCRIPT_FILE);
    fs::write(&script, SCRIPT)?;
    let mut command = Command::new("powershell");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
        ])
        .args(["-WindowStyle", "Hidden", "-File"])
        .arg(&script)
        .arg("-ProcessId")
        .arg(handoff.process_id.to_string())
        .arg("-Kind")
        .arg(kind)
        .arg("-Source")
        .arg(handoff.update)
        .arg("-Target")
        .arg(handoff.target)
        .arg("-FailureNote")
        .arg(handoff.update.with_file_name(FAILURE_FILE))
        .creation_flags(CREATE_NO_WINDOW);
    Ok(command)
}

pub fn install(update: &Path, package: Package) -> io::Result<()> {
    let target = env::current_exe()?;
    let handoff = Handoff {
        process_id: process::id(),
        package,
        update,
        target: &target,
    };
    command(&handoff)?.spawn().map(drop)
}
