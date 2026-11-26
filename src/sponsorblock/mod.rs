pub mod api;
pub mod bridge;
pub mod cache;
pub mod category;
pub mod client;
pub mod config;
pub mod segment;
pub mod video;

pub use bridge::{Reply, Request, answer};
pub use cache::Cache;
pub use category::{Categories, Category};
pub use client::{Client, Error};
pub use config::{Channel, Config};
pub use segment::{Action, Segment, Skip, skips};
pub use video::VideoId;

pub const SCRIPT: &str = include_str!("youtube.js");
