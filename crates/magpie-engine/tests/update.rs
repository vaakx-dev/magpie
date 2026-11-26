use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use magpie_engine::update::{
    CHECK_INTERVAL, Channel, Download, Failure, Offer, Package, Platform, Repo, Request, Schedule,
    Status, Version, check,
};
use serde_json::json;

const LATEST: &str = "https://api.github.com/repos/vaakx-dev/magpie/releases/latest";
const NIGHTLY: &str = "https://api.github.com/repos/vaakx-dev/magpie/releases/tags/nightly";
const PAGE: &str = "https://github.com/vaakx-dev/magpie/releases/tag/v2.0.0";
const HASH: &str = "9F86D081884C7D659A2FEAA0C55AD015A3BF4F1B2B0B822CD15D6C15B0F00A08";
const LOWER_HASH: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

fn version(text: &str) -> Version {
    Version::parse(text).unwrap_or_else(|| panic!("{text} should parse"))
}

fn asset_url(name: &str) -> String {
    format!("https://github.com/vaakx-dev/magpie/releases/download/v2.0.0/{name}")
}

fn release_with(tag: &str, body: &str, assets: &[&str]) -> String {
    let assets: Vec<_> = assets
        .iter()
        .map(|name| json!({ "name": name, "browser_download_url": asset_url(name), "size": 1024 }))
        .collect();
    json!({
        "tag_name": tag,
        "html_url": PAGE,
        "draft": false,
        "body": body,
        "assets": assets,
    })
    .to_string()
}

fn release(tag: &str, assets: &[&str]) -> String {
    release_with(tag, "Faster tabs", assets)
}

fn nightly_release(version: &str, assets: &[&str]) -> String {
    let body = format!("Version: {version}\nCommit: 0123456789abcdef0123456789abcdef01234567\n");
    release_with("nightly", &body, assets)
}

fn github(files: &[(&str, String)]) -> impl Fn(&str) -> Result<String, Failure> + use<> {
    let files: HashMap<String, String> = files
        .iter()
        .map(|(address, body)| (address.to_string(), body.clone()))
        .collect();
    move |address| files.get(address).cloned().ok_or(Failure::NotFound)
}

fn status_of(
    current: &str,
    channel: Channel,
    platform: Platform,
    fetch: impl Fn(&str) -> Result<String, Failure>,
) -> Status {
    let request = Request {
        repo: &Repo::default(),
        current: &version(current),
        platform,
        channel,
    };
    check(&request, fetch)
}

fn status_on(platform: Platform, fetch: impl Fn(&str) -> Result<String, Failure>) -> Status {
    status_of("1.4.0", Channel::Stable, platform, fetch)
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
#[test]
fn channels_parse_from_their_labels() {
    assert_eq!(Channel::default(), Channel::Stable);
    assert_eq!(Channel::parse("Stable"), Some(Channel::Stable));
    assert_eq!(Channel::parse(" nightly "), Some(Channel::Nightly));
    assert_eq!(Channel::parse("NIGHTLY"), Some(Channel::Nightly));
    assert_eq!(Channel::parse("beta"), None);
    assert_eq!(Channel::parse(""), None);
    for channel in Channel::ALL {
        assert_eq!(Channel::parse(channel.label()), Some(channel));
    }
}

#[test]
fn nightly_builds_are_told_apart_from_releases() {
    assert!(version("0.1.1-nightly.202611261600").nightly());
    assert!(!version("0.1.1").nightly());
    assert!(!version("0.1.1-rc.1").nightly());
    assert!(version("0.1.1-nightly.202611261600") < version("0.1.1-nightly.202611261900"));
    assert!(version("0.1.1-nightly.202612312100") < version("0.1.2-nightly.202701010000"));
}

#[test]
fn the_nightly_channel_offers_each_newer_nightly_once() {
    let latest = "0.1.1-nightly.202611261600";
    let status = |current: &str| {
        status_of(
            current,
            Channel::Nightly,
            Platform::WindowsInstalled,
            github(&[(NIGHTLY, nightly_release(latest, &[]))]),
        )
    };
    let offered_version = |status: Status| match status {
        Status::Available(offer) => offer.version,
        other => panic!("expected an update, got {other:?}"),
    };

    assert_eq!(offered_version(status("0.1.1")), latest);
    assert_eq!(offered_version(status("0.2.0")), latest);
    assert_eq!(
        offered_version(status("0.1.1-nightly.202611261300")),
        latest
    );
    assert_eq!(status(latest), Status::UpToDate);
    assert_eq!(status("0.1.1-nightly.202611261900"), Status::UpToDate);
}

#[test]
fn a_nightly_without_a_stated_version_is_unreadable() {
    let status = status_of(
        "0.1.1",
        Channel::Nightly,
        Platform::WindowsInstalled,
        github(&[(NIGHTLY, release("nightly", &[]))]),
    );

    assert_eq!(
        status,
        Status::Failed {
            message: "The release on GitHub is unreadable".to_owned()
        }
    );
}

#[test]
fn the_stable_channel_moves_a_nightly_build_to_the_release_it_leads_to() {
    let status = |current: &str| {
        status_of(
            current,
            Channel::Stable,
            Platform::WindowsInstalled,
            github(&[(LATEST, release("v0.1.1", &[]))]),
        )
    };

    assert!(matches!(
        status("0.1.1-nightly.202611261600"),
        Status::Available(Offer { version, .. }) if version == "0.1.1"
    ));
    assert_eq!(status("0.1.2-nightly.202612011200"), Status::UpToDate);
}

#[test]
fn nightly_assets_are_picked_and_verified_like_release_assets() {
    let assets = [
        "Magpie-nightly-setup.exe",
        "Magpie-nightly-windows-x64.exe",
        "Magpie-nightly-windows-x64.zip",
        "Magpie-nightly-android-arm64-v8a.apk",
        "checksums.txt",
    ];
    let checksums = format!(
        "{}  Magpie-nightly-setup.exe\n{HASH}  Magpie-nightly-windows-x64.exe\n{HASH}  Magpie-nightly-android-arm64-v8a.apk\n",
        "0".repeat(64)
    );
    let fetch = || {
        github(&[
            (
                NIGHTLY,
                nightly_release("0.1.1-nightly.202611261600", &assets),
            ),
            (&asset_url("checksums.txt"), checksums.clone()),
        ])
    };
    let download =
        |platform: Platform| match status_of("0.1.1", Channel::Nightly, platform, fetch()) {
            Status::Available(offer) => offer.download.unwrap(),
            other => panic!("expected an update, got {other:?}"),
        };
    let phone = ["arm64-v8a".to_owned()];

    let portable = download(Platform::WindowsPortable);
    let installer = download(Platform::WindowsInstalled);
    let apk = download(Platform::Android { abis: &phone });

    assert_eq!(
        (
            portable.name.as_str(),
            portable.package,
            portable.sha256.as_deref()
        ),
        (
            "Magpie-nightly-windows-x64.exe",
            Package::Portable,
            Some(LOWER_HASH)
        )
    );
    assert_eq!(
        (installer.name.as_str(), installer.package),
        ("Magpie-nightly-setup.exe", Package::Installer)
    );
    assert_eq!(installer.sha256.as_deref(), Some("0".repeat(64).as_str()));
    assert_eq!(
        (apk.name.as_str(), apk.package, apk.sha256.as_deref()),
        (
            "Magpie-nightly-android-arm64-v8a.apk",
            Package::Apk,
            Some(LOWER_HASH)
        )
    );
}
