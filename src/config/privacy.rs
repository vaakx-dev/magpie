use std::borrow::Cow;

use super::{Choosable, Switch};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrackingPrevention {
    Basic,
    #[default]
    Balanced,
    Strict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Privacy {
    pub shield: Switch,
    pub tracking: TrackingPrevention,
    pub do_not_track: Switch,
    pub https_only: Switch,
    pub strip_tracking: Switch,
    pub trim_referrer: Switch,
    pub block_preloading: Switch,
    pub block_pings: Switch,
    pub clear_on_exit: Switch,
}

impl Choosable for TrackingPrevention {
    const ALL: &'static [Self] = &[Self::Basic, Self::Balanced, Self::Strict];

    fn label(self) -> Cow<'static, str> {
        Cow::Borrowed(match self {
            Self::Basic => "Basic",
            Self::Balanced => "Balanced",
            Self::Strict => "Strict",
        })
    }
}

impl Default for Privacy {
    fn default() -> Self {
        Self {
            shield: Switch::On,
            tracking: TrackingPrevention::default(),
            do_not_track: Switch::On,
            https_only: Switch::Off,
            strip_tracking: Switch::On,
            trim_referrer: Switch::Off,
            block_preloading: Switch::Off,
            block_pings: Switch::Off,
            clear_on_exit: Switch::Off,
        }
    }
}
