use super::Asset;

const SHA256_HEX_LENGTH: usize = 64;
const SINGLE_SUFFIX: &str = ".sha256";
const LISTS: [&str; 3] = ["checksums.txt", "sha256sums", "sha256sums.txt"];

fn single_for(asset: &Asset, name: &str) -> bool {
    asset
        .name
        .eq_ignore_ascii_case(&format!("{name}{SINGLE_SUFFIX}"))
}

fn list(asset: &Asset) -> bool {
    LISTS
        .iter()
        .any(|list| asset.name.eq_ignore_ascii_case(list))
}

pub fn listing<'a>(assets: &'a [Asset], name: &str) -> Option<&'a Asset> {
    assets
        .iter()
        .find(|asset| single_for(asset, name))
        .or_else(|| assets.iter().find(|asset| list(asset)))
}

fn hash(text: &str) -> Option<String> {
    let valid =
        text.len() == SHA256_HEX_LENGTH && text.chars().all(|digit| digit.is_ascii_hexdigit());
    valid.then(|| text.to_ascii_lowercase())
}

fn entry(line: &str, name: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    let digest = hash(parts.next()?)?;
    let file = parts.next().map(|file| file.trim_start_matches('*'));
    file.is_none_or(|file| file == name).then_some(digest)
}

pub fn sha256(listing: &str, name: &str) -> Option<String> {
    listing.lines().find_map(|line| entry(line, name))
}
