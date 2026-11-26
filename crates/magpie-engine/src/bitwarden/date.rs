use std::time::{SystemTime, UNIX_EPOCH};

const SECS_PER_MINUTE: u64 = 60;
const SECS_PER_HOUR: u64 = 3_600;
const SECS_PER_DAY: u64 = 86_400;
const DAYS_FROM_ERA_START_TO_EPOCH: u64 = 719_468;
const DAYS_PER_ERA: u64 = 146_097;
const YEARS_PER_ERA: u64 = 400;

fn civil_date(days_since_epoch: u64) -> (u64, u64, u64) {
    let days = days_since_epoch + DAYS_FROM_ERA_START_TO_EPOCH;
    let era = days / DAYS_PER_ERA;
    let day_of_era = days % DAYS_PER_ERA;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = match shifted_month < 10 {
        true => shifted_month + 3,
        false => shifted_month - 9,
    };
    let year = year_of_era + era * YEARS_PER_ERA + u64::from(month <= 2);
    (year, month, day)
}

pub fn timestamp(now: SystemTime) -> String {
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let (year, month, day) = civil_date(secs / SECS_PER_DAY);
    let of_day = secs % SECS_PER_DAY;
    let hour = of_day / SECS_PER_HOUR;
    let minute = of_day % SECS_PER_HOUR / SECS_PER_MINUTE;
    let second = of_day % SECS_PER_MINUTE;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}
