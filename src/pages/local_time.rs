use std::ffi::c_void;

use windows::{
    Win32::System::Registry::{HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RegGetValueW},
    core::{PCWSTR, w},
};

use crate::storage::clock::Epoch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalTime {
    pub now: Epoch,
    pub utc_offset_secs: i64,
}

const TIME_ZONE: PCWSTR = w!(r"SYSTEM\CurrentControlSet\Control\TimeZoneInformation");
const ACTIVE_BIAS: PCWSTR = w!("ActiveTimeBias");
const DWORD_BYTES: u32 = 4;
const SECS_PER_MINUTE: i64 = 60;
const SECS_PER_DAY: i64 = 24 * 60 * 60;

fn bias_minutes() -> i64 {
    let mut bias = 0u32;
    let mut size = DWORD_BYTES;
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            TIME_ZONE,
            ACTIVE_BIAS,
            RRF_RT_REG_DWORD,
            None,
            Some((&raw mut bias).cast::<c_void>()),
            Some(&mut size),
        )
    };
    match status.is_ok() {
        true => i64::from(bias.cast_signed()),
        false => 0,
    }
}

impl LocalTime {
    pub fn current() -> Self {
        Self {
            now: Epoch::now(),
            utc_offset_secs: -bias_minutes() * SECS_PER_MINUTE,
        }
    }

    fn day(self, at: Epoch) -> i64 {
        let local_secs = i64::try_from(at.0)
            .unwrap_or(i64::MAX)
            .saturating_add(self.utc_offset_secs);
        local_secs.div_euclid(SECS_PER_DAY)
    }

    pub fn days_ago(self, at: Epoch) -> i64 {
        self.day(self.now) - self.day(at)
    }
}
