use url::Url;

use crate::host::within;

const WEB_SCHEMES: [&str; 2] = ["http", "https"];
const CAMPAIGN_PREFIX: &str = "utm_";
const CLICK_IDS: [&str; 20] = [
    "fbclid",
    "gclid",
    "gclsrc",
    "dclid",
    "gbraid",
    "wbraid",
    "msclkid",
    "mc_cid",
    "mc_eid",
    "igshid",
    "yclid",
    "twclid",
    "ttclid",
    "li_fat_id",
    "_hsenc",
    "_hsmi",
    "mkt_tok",
    "oly_anon_id",
    "oly_enc_id",
    "vero_id",
];
const SITE_PARAMS: [(&str, &str); 3] = [
    ("youtube.com", "si"),
    ("youtu.be", "si"),
    ("spotify.com", "si"),
];

fn key(pair: &str) -> &str {
    pair.split_once('=').map_or(pair, |(key, _)| key)
}

fn tracks(key: &str, host: &str) -> bool {
    key.starts_with(CAMPAIGN_PREFIX)
        || CLICK_IDS.contains(&key)
        || SITE_PARAMS
            .iter()
            .any(|(site, param)| *param == key && within(host, site))
}

pub fn stripped(url: &Url) -> Option<Url> {
    if !WEB_SCHEMES.contains(&url.scheme()) {
        return None;
    }
    let (Some(host), Some(query)) = (url.host_str(), url.query()) else {
        return None;
    };
    let pairs = query.split('&');
    let kept: Vec<&str> = pairs
        .clone()
        .filter(|pair| !tracks(key(pair), host))
        .collect();
    if kept.len() == pairs.count() {
        return None;
    }
    let mut clean = url.clone();
    clean.set_query((!kept.is_empty()).then(|| kept.join("&")).as_deref());
    Some(clean)
}

pub fn trimmed_referrer(referrer: &Url, destination: &Url) -> Option<String> {
    let origin = referrer.origin();
    if !origin.is_tuple() || origin == destination.origin() {
        return None;
    }
    let trimmed = format!("{}/", origin.ascii_serialization());
    (trimmed != referrer.as_str()).then_some(trimmed)
}
