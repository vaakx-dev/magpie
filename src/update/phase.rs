use magpie_engine::update::{Offer, Status};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Phase {
    Idle,
    Checking,
    UpToDate,
    Failed { message: String },
    Available { offer: Offer },
    Downloading { offer: Offer, percent: u8 },
    DownloadFailed { offer: Offer, message: String },
    Installing { offer: Offer },
}

impl From<Status> for Phase {
    fn from(status: Status) -> Self {
        match status {
            Status::UpToDate => Self::UpToDate,
            Status::Available(offer) => Self::Available { offer },
            Status::Failed { message } => Self::Failed { message },
        }
    }
}
