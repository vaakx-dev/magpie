use serde::Deserialize;
use url::form_urlencoded;

use super::{
    category::{Categories, Category},
    segment::{Action, Segment},
    video::VideoId,
};

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "videoID")]
    video: String,
    segments: Vec<Listing>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Listing {
    category: String,
    action_type: String,
    segment: [f64; 2],
    #[serde(rename = "UUID")]
    uuid: String,
    #[serde(default)]
    votes: i64,
}

const ENDPOINT: &str = "https://sponsor.ajay.app/api/skipSegments/";

fn category_list(categories: Categories) -> String {
    let names: Vec<String> = categories
        .iter()
        .map(|category| format!("\"{}\"", category.name()))
        .collect();
    format!("[{}]", names.join(","))
}

pub fn address(video: &VideoId, categories: Categories) -> String {
    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair("categories", &category_list(categories))
        .finish();
    format!("{ENDPOINT}{}?{query}", video.hash_prefix())
}

fn segment(listing: Listing) -> Option<Segment> {
    let [start_secs, end_secs] = listing.segment;
    let category = Category::parse(&listing.category)?;
    let action = Action::parse(&listing.action_type)?;
    let ordered = start_secs.is_finite() && end_secs.is_finite() && start_secs <= end_secs;
    ordered.then_some(Segment {
        start_secs,
        end_secs,
        category,
        action,
        uuid: listing.uuid,
        votes: listing.votes,
    })
}

pub fn parse(body: &str, video: &VideoId) -> serde_json::Result<Vec<Segment>> {
    let entries: Vec<Entry> = serde_json::from_str(body)?;
    Ok(entries
        .into_iter()
        .filter(|entry| entry.video == video.as_str())
        .flat_map(|entry| entry.segments)
        .filter_map(segment)
        .collect())
}
