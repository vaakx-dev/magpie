use url::Url;

use super::site::Site;

const WATCH_PATH: &str = "/watch";
const VIDEO_PARAM: &str = "v";

fn short_id(url: &Url) -> Option<String> {
    let segments: Vec<&str> = url
        .path_segments()?
        .filter(|segment| !segment.is_empty())
        .collect();
    match segments.as_slice() {
        ["shorts", id] => Some((*id).to_owned()),
        _ => None,
    }
}

fn on_youtube(url: &Url) -> bool {
    url.host_str().and_then(Site::of) == Some(Site::YouTube)
}

pub(super) fn watch(url: &Url) -> Option<Url> {
    let id = short_id(url).filter(|_| on_youtube(url))?;
    let mut watch = url.clone();
    watch.set_path(WATCH_PATH);
    watch.set_query(None);
    watch.set_fragment(None);
    watch.query_pairs_mut().append_pair(VIDEO_PARAM, &id);
    Some(watch)
}
