use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use magpie_engine::update::{
    CHECK_INTERVAL, Download, Failure, Offer, Package, Platform, Repo, Request, Schedule, Status,
    Version, check,
};
use serde_json::json;

const LATEST: &str = "https://api.github.com/repos/vaakx-dev/magpie/releases/latest";
const PAGE: &str = "https://github.com/vaakx-dev/magpie/releases/tag/v2.0.0";
const HASH: &str = "9F86D081884C7D659A2FEAA0C55AD015A3BF4F1B2B0B822CD15D6C15B0F00A08";
const LOWER_HASH: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

fn version(text: &str) -> Version {
    Version::parse(text).unwrap_or_else(|| panic!("{text} should parse"))
}

fn asset_url(name: &str) -> String {
    format!("https://github.com/vaakx-dev/magpie/releases/download/v2.0.0/{name}")
}

fn release(tag: &str, assets: &[&str]) -> String {
    let assets: Vec<_> = assets
        .iter()
        .map(|name| json!({ "name": name, "browser_download_url": asset_url(name), "size": 1024 }))
        .collect();
    json!({
        "tag_name": tag,
        "html_url": PAGE,
        "draft": false,
        "body": "Faster tabs",
        "assets": assets,
    })
    .to_string()
}

fn github(files: &[(&str, String)]) -> impl Fn(&str) -> Result<String, Failure> + use<> {
    let files: HashMap<String, String> = files
        .iter()
        .map(|(address, body)| (address.to_string(), body.clone()))
        .collect();
    move |address| files.get(address).cloned().ok_or(Failure::NotFound)
}

fn status_on(platform: Platform, fetch: impl Fn(&str) -> Result<String, Failure>) -> Status {
    let request = Request {
        repo: &Repo::default(),
        current: &version("1.4.0"),
        platform,
    };
    check(&request, fetch)
}

fn offered(platform: Platform, assets: &[&str]) -> Option<Download> {
    let fetch = github(&[(LATEST, release("v2.0.0", assets))]);
    match status_on(platform, fetch) {
        Status::Available(offer) => offer.download,
        other => panic!("expected an update, got {other:?}"),
    }
}

fn offered_name(platform: Platform, assets: &[&str]) -> Option<String> {
    offered(platform, assets).map(|download| download.name)
}

#[test]
fn versions_follow_semantic_version_precedence() {
    let ascending = [
        "0.1.0",
        "0.9.9",
        "1.0.0-alpha",
        "1.0.0-alpha.1",
        "1.0.0-alpha.beta",
        "1.0.0-beta",
        "1.0.0-beta.2",
        "1.0.0-beta.11",
        "1.0.0-rc.1",
        "1.0.0",
        "1.0.1",
        "1.2.0",
        "1.10.0",
        "2.0.0",
    ];

    for pair in ascending.windows(2) {
        assert!(version(pair[0]) < version(pair[1]), "{pair:?}");
    }
    assert_eq!(version("v1.2.3"), version("1.2.3"));
    assert_eq!(version("1.2.3+build.7"), version("1.2.3"));
    assert_eq!(version("V1.2.3-rc.1").to_string(), "1.2.3-rc.1");
    for invalid in [
        "",
        "1.2",
        "1.2.3.4",
        "one.two.three",
        "1.2.3-",
        "1.2.3-rc..1",
        "1.2.3-r c",
    ] {
        assert_eq!(Version::parse(invalid), None, "{invalid}");
    }
}

#[test]
fn a_newer_release_offers_its_windows_installer_with_its_checksum() {
    let fetch = github(&[
        (
            LATEST,
            release(
                "v2.0.0",
                &[
                    "Magpie-2.0.0-windows-x64.exe",
                    "Magpie-2.0.0-setup.exe",
                    "checksums.txt",
                ],
            ),
        ),
        (
            &asset_url("checksums.txt"),
            format!(
                "{}  Magpie-2.0.0-windows-x64.exe\n{HASH} *Magpie-2.0.0-setup.exe\n",
                "0".repeat(64)
            ),
        ),
    ]);

    assert_eq!(
        status_on(Platform::WindowsInstalled, fetch),
        Status::Available(Offer {
            version: "2.0.0".to_owned(),
            page: PAGE.to_owned(),
            notes: "Faster tabs".to_owned(),
            download: Some(Download {
                name: "Magpie-2.0.0-setup.exe".to_owned(),
                url: asset_url("Magpie-2.0.0-setup.exe"),
                size: 1024,
                sha256: Some(LOWER_HASH.to_owned()),
                package: Package::Installer,
            }),
        })
    );
}

#[test]
fn windows_prefers_the_package_that_matches_how_magpie_was_installed() {
    let both = [
        "Magpie-2.0.0-setup.exe",
        "magpie.exe",
        "Magpie-2.0.0-windows-x64.zip",
    ];
    let installed = offered(Platform::WindowsInstalled, &both).unwrap();
    let portable = offered(Platform::WindowsPortable, &both).unwrap();

    assert_eq!(
        (installed.name.as_str(), installed.package),
        ("Magpie-2.0.0-setup.exe", Package::Installer)
    );
    assert_eq!(
        (portable.name.as_str(), portable.package),
        ("magpie.exe", Package::Portable)
    );
    assert_eq!(
        offered_name(Platform::WindowsPortable, &["Magpie.msi"]).as_deref(),
        Some("Magpie.msi")
    );
    assert_eq!(
        offered_name(
            Platform::WindowsInstalled,
            &["magpie-arm64-v8a.apk", "source.zip"]
        ),
        None
    );
}

#[test]
fn android_picks_the_apk_for_the_device_abi_then_a_universal_one() {
    let apks = [
        "magpie-2.0.0-x86_64.apk",
        "magpie-2.0.0-arm64-v8a.apk",
        "magpie-2.0.0-universal.apk",
        "Magpie-2.0.0-setup.exe",
    ];
    let abis = |list: &[&str]| list.iter().map(|abi| abi.to_string()).collect::<Vec<_>>();
    let phone = abis(&["arm64-v8a", "armeabi-v7a"]);
    let emulator = abis(&["x86", "armeabi-v7a"]);
    let tablet = abis(&["x86_64"]);

    let android = |abis: &[String], assets: &[&str]| offered(Platform::Android { abis }, assets);

    let picked = android(&phone, &apks).unwrap();
    assert_eq!(
        (picked.name.as_str(), picked.package),
        ("magpie-2.0.0-arm64-v8a.apk", Package::Apk)
    );
    assert_eq!(
        android(&tablet, &apks).unwrap().name,
        "magpie-2.0.0-x86_64.apk"
    );
    assert_eq!(
        android(&emulator, &apks).unwrap().name,
        "magpie-2.0.0-universal.apk"
    );
    assert_eq!(
        android(&emulator, &["magpie.apk", "magpie-x86_64.apk"])
            .unwrap()
            .name,
        "magpie.apk"
    );
    assert_eq!(android(&emulator, &["magpie-x86_64.apk"]), None);
}

#[test]
fn a_single_checksum_file_verifies_its_asset() {
    let fetch = github(&[
        (
            LATEST,
            release(
                "v2.0.0",
                &["magpie.exe", "magpie.exe.sha256", "checksums.txt"],
            ),
        ),
        (&asset_url("magpie.exe.sha256"), format!("{HASH}\n")),
    ]);

    let Status::Available(offer) = status_on(Platform::WindowsPortable, fetch) else {
        panic!("expected an update");
    };
    assert_eq!(offer.download.unwrap().sha256.as_deref(), Some(LOWER_HASH));
}

#[test]
fn only_a_newer_release_is_offered() {
    let status = |tag: &str| {
        status_on(
            Platform::WindowsInstalled,
            github(&[(LATEST, release(tag, &[]))]),
        )
    };

    assert!(
        matches!(status("v1.5.0"), Status::Available(Offer { version, download: None, .. }) if version == "1.5.0")
    );
    assert_eq!(status("v1.4.0"), Status::UpToDate);
    assert_eq!(status("v1.3.9"), Status::UpToDate);
    assert_eq!(status("v1.4.0-rc.1"), Status::UpToDate);
}

#[test]
fn missing_releases_and_failures_end_in_a_clean_status() {
    let failed =
        |failure: Failure| status_on(Platform::WindowsInstalled, move |_: &str| Err(failure));
    let message = |status: Status| match status {
        Status::Failed { message } => message,
        other => panic!("expected a failure, got {other:?}"),
    };

    assert_eq!(failed(Failure::NotFound), Status::UpToDate);
    assert_eq!(
        message(failed(Failure::Offline)),
        "Can't reach GitHub. Check your connection"
    );
    assert_eq!(
        message(failed(Failure::RateLimited)),
        "GitHub is busy. Try again later"
    );
    for unreadable in [
        "<html>",
        "{\"message\": \"Not Found\"}",
        &release("nightly", &[]),
    ] {
        let status = status_on(
            Platform::WindowsInstalled,
            github(&[(LATEST, unreadable.to_owned())]),
        );
        assert_eq!(
            message(status),
            "The release on GitHub is unreadable",
            "{unreadable}"
        );
    }
    let unfetched_checksum =
        github(&[(LATEST, release("v2.0.0", &["magpie.exe", "checksums.txt"]))]);
    assert_eq!(
        message(status_on(Platform::WindowsPortable, unfetched_checksum)),
        "GitHub couldn't find the update"
    );
}

#[test]
fn checks_are_due_twelve_hours_after_the_last_one() {
    let folder = tempfile::tempdir().unwrap();
    let schedule = Schedule::at(folder.path().join("updates").join("last-check"));
    let checked = SystemTime::now();

    assert!(schedule.due(checked));
    schedule.record(checked).unwrap();
    assert!(!schedule.due(checked + CHECK_INTERVAL - Duration::from_secs(60)));
    assert!(schedule.due(checked + CHECK_INTERVAL));
}
