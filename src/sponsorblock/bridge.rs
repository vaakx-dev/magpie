use serde::Deserialize;
use serde_json::{Value, json};
use url::Url;

use super::{
    client::Client,
    config::{Channel, Config},
    segment::{Skip, skips},
    video::VideoId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub number: u64,
    pub video: VideoId,
    pub channel: Channel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reply {
    pub request: u64,
    pub notice: bool,
    pub skips: Vec<Skip>,
}

#[derive(Deserialize)]
struct Incoming {
    kind: String,
    request: u64,
    address: String,
    channel_id: Option<String>,
    channel_name: Option<String>,
}

pub const REQUEST_KIND: &str = "magpie_sponsorblock_request";
pub const REPLY_KIND: &str = "magpie_sponsorblock_segments";

impl Request {
    pub fn parse(json: &str) -> Option<Self> {
        let incoming: Incoming = serde_json::from_str(json).ok()?;
        if incoming.kind != REQUEST_KIND {
            return None;
        }
        let video = VideoId::from_url(&Url::parse(&incoming.address).ok()?)?;
        Some(Self {
            number: incoming.request,
            video,
            channel: Channel {
                id: incoming.channel_id,
                name: incoming.channel_name,
            },
        })
    }
}

fn skip_json(skip: &Skip) -> Value {
    json!({
        "start": skip.start_secs,
        "end": skip.end_secs,
        "label": skip.category.label(),
        "color": skip.category.color(),
    })
}

impl Reply {
    pub fn json(&self) -> String {
        let skips: Vec<Value> = self.skips.iter().map(skip_json).collect();
        json!({
            "kind": REPLY_KIND,
            "request": self.request,
            "notice": self.notice,
            "skips": skips,
        })
        .to_string()
    }
}

fn wanted(config: &Config, request: &Request) -> bool {
    config.enabled && !config.exempts(&request.channel)
}

fn fetched(client: &Client, config: &Config, request: &Request) -> Vec<Skip> {
    match client.segments(&request.video, config.categories) {
        Ok(segments) => skips(&segments),
        Err(error) => {
            eprintln!("magpie: {error}");
            Vec::new()
        }
    }
}

pub fn answer(client: &Client, config: &Config, request: &Request) -> Reply {
    let skips = if wanted(config, request) {
        fetched(client, config, request)
    } else {
        Vec::new()
    };
    Reply {
        request: request.number,
        notice: config.show_skip_notice,
        skips,
    }
}
