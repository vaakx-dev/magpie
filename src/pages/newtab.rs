use serde_json::{Value, json};

use super::link::shortcuts;
use crate::storage::session::Space;

pub fn newtab_payload(space: &Space) -> Value {
    json!({
        "space": space.name,
        "favorites": shortcuts(&space.favorites),
    })
}
