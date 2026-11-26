use serde::Serialize;

use super::Asset;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Package {
    Installer,
    Portable,
    Apk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform<'a> {
    WindowsInstalled,
    WindowsPortable,
    Android { abis: &'a [String] },
}

const ANDROID_ABIS: [&str; 4] = ["arm64-v8a", "armeabi-v7a", "x86_64", "x86"];
const UNIVERSAL: &str = "universal";
const INSTALLER_WORDS: [&str; 2] = ["setup", "installer"];

fn package(name: &str) -> Option<Package> {
    let name = name.to_ascii_lowercase();
    let installer = INSTALLER_WORDS.iter().any(|word| name.contains(word));
    match name.rsplit_once('.')?.1 {
        "apk" => Some(Package::Apk),
        "msi" => Some(Package::Installer),
        "exe" if installer => Some(Package::Installer),
        "exe" => Some(Package::Portable),
        _ => None,
    }
}

fn continues_word(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn mentions(name: &str, word: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name.match_indices(word).any(|(start, _)| {
        let next = name[start + word.len()..].chars().next();
        !next.is_some_and(continues_word)
    })
}

fn of_kind(assets: &[Asset], kind: Package) -> impl Iterator<Item = &Asset> {
    assets
        .iter()
        .filter(move |asset| package(&asset.name) == Some(kind))
}

fn windows(assets: &[Asset], preferred: Package, fallback: Package) -> Option<(&Asset, Package)> {
    let first = |kind| of_kind(assets, kind).next().map(|asset| (asset, kind));
    first(preferred).or_else(|| first(fallback))
}

fn android<'a>(assets: &'a [Asset], abis: &[String]) -> Option<(&'a Asset, Package)> {
    let apks: Vec<&Asset> = of_kind(assets, Package::Apk).collect();
    let named = |word: &str| {
        apks.iter()
            .copied()
            .find(|asset| mentions(&asset.name, word))
    };
    let unmarked = || {
        apks.iter()
            .copied()
            .find(|asset| !ANDROID_ABIS.iter().any(|abi| mentions(&asset.name, abi)))
    };
    abis.iter()
        .find_map(|abi| named(abi))
        .or_else(|| named(UNIVERSAL))
        .or_else(unmarked)
        .map(|asset| (asset, Package::Apk))
}

pub fn pick<'a>(assets: &'a [Asset], platform: Platform) -> Option<(&'a Asset, Package)> {
    match platform {
        Platform::WindowsInstalled => windows(assets, Package::Installer, Package::Portable),
        Platform::WindowsPortable => windows(assets, Package::Portable, Package::Installer),
        Platform::Android { abis } => android(assets, abis),
    }
}
