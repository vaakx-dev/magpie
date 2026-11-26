use std::{
    env, fs, io,
    os::windows::process::CommandExt,
    path::Path,
    process::{self, Command},
};

use magpie_engine::update::Package;

const SWAP_SCRIPT: &str = include_str!("swap.ps1");
const SWAP_FILE: &str = "swap.ps1";
const MSI_EXTENSION: &str = "msi";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn run_installer(file: &Path) -> io::Result<()> {
    let msi = file
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(MSI_EXTENSION));
    let mut command = match msi {
        true => {
            let mut msiexec = Command::new("msiexec");
            msiexec.arg("/i").arg(file);
            msiexec
        }
        false => Command::new(file),
    };
    command.spawn().map(drop)
}

fn swap_after_exit(file: &Path) -> io::Result<()> {
    let script = file.with_file_name(SWAP_FILE);
    fs::write(&script, SWAP_SCRIPT)?;
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
        ])
        .args(["-WindowStyle", "Hidden", "-File"])
        .arg(&script)
        .arg("-ProcessId")
        .arg(process::id().to_string())
        .arg("-Source")
        .arg(file)
        .arg("-Target")
        .arg(env::current_exe()?)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(drop)
}

pub fn install(file: &Path, package: Package) -> io::Result<()> {
    match package {
        Package::Installer => run_installer(file),
        Package::Portable => swap_after_exit(file),
        Package::Apk => Err(io::ErrorKind::Unsupported.into()),
    }
}
