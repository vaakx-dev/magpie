use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Epoch(pub u64);

impl Epoch {
    pub fn now() -> Self {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(since_epoch.as_secs())
    }

    pub fn since(self, earlier: Epoch) -> Duration {
        Duration::from_secs(self.0.saturating_sub(earlier.0))
    }

    pub fn after(self, duration: Duration) -> Self {
        Self(self.0.saturating_add(duration.as_secs()))
    }
}
