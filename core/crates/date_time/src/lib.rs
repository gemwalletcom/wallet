mod rfc3339;
#[cfg(feature = "serde")]
mod serialization;

use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Sub};
use std::time::{Duration, SystemTime};

use crate::rfc3339::Rfc3339;
pub use crate::rfc3339::SecondsFormat;
#[cfg(feature = "serde")]
pub use crate::serialization::{deserialize_datetime_from_micros, deserialize_datetime_from_millis, deserialize_datetime_from_nanos, deserialize_datetime_from_seconds, deserialize_option_datetime_from_millis};

const MICROS_PER_MILLI: i64 = 1_000;
const MICROS_PER_SECOND: i64 = 1_000_000;
const MICROS_PER_DAY: i64 = 86_400 * MICROS_PER_SECOND;
const DAYS_PER_ERA: i64 = 146_097;
const ERA_DAYS_BEFORE_UNIX_EPOCH: i64 = 719_468;
const MIN_MICROS: i64 = -62_167_219_200 * MICROS_PER_SECOND;
const MAX_MICROS: i64 = 253_402_300_800 * MICROS_PER_SECOND - 1;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(i64);

impl DateTime {
    pub const UNIX_EPOCH: Self = Self(0);

    pub fn now() -> Self {
        Self::UNIX_EPOCH + SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default()
    }

    pub const fn from_timestamp(seconds: i64) -> Self {
        Self::clamped(seconds.saturating_mul(MICROS_PER_SECOND))
    }

    pub const fn from_timestamp_millis(millis: i64) -> Self {
        Self::clamped(millis.saturating_mul(MICROS_PER_MILLI))
    }

    pub const fn from_timestamp_micros(micros: i64) -> Self {
        Self::clamped(micros)
    }

    pub fn checked_from_timestamp(seconds: i64) -> Option<Self> {
        seconds.checked_mul(MICROS_PER_SECOND).and_then(Self::checked)
    }

    pub fn checked_from_timestamp_millis(millis: i64) -> Option<Self> {
        millis.checked_mul(MICROS_PER_MILLI).and_then(Self::checked)
    }

    pub fn from_date(year: i32, month: u32, day: u32) -> Option<Self> {
        let days = days_from_civil(year, month, day);
        (civil_from_days(days) == (year, month, day)).then_some(days)?.checked_mul(MICROS_PER_DAY).and_then(Self::checked)
    }

    pub fn parse_rfc3339(text: &str) -> Option<Self> {
        let (local, offset) = rfc3339::parse_local(text)?;
        Some(local.shifted(-rfc3339::parse_offset(offset)?))
    }

    pub const fn timestamp(&self) -> i64 {
        self.0.div_euclid(MICROS_PER_SECOND)
    }

    pub const fn timestamp_millis(&self) -> i64 {
        self.0.div_euclid(MICROS_PER_MILLI)
    }

    pub const fn timestamp_micros(&self) -> i64 {
        self.0
    }

    pub fn date(&self) -> (i32, u32, u32) {
        civil_from_days(self.0.div_euclid(MICROS_PER_DAY))
    }

    pub fn elapsed(&self) -> Duration {
        Self::now() - *self
    }

    pub const fn abs_diff(&self, other: Self) -> Duration {
        Duration::from_micros(self.0.abs_diff(other.0))
    }

    pub fn truncated(&self, unit: Duration) -> Self {
        self.0.checked_rem_euclid(micros(unit)).map_or(*self, |rest| Self(self.0 - rest))
    }

    pub fn to_local(&self, utc_offset_seconds: i32) -> Self {
        self.shifted(i64::from(utc_offset_seconds) * MICROS_PER_SECOND)
    }

    pub fn to_rfc3339(&self, format: SecondsFormat) -> String {
        Rfc3339(*self, format).to_string()
    }

    const fn shifted(&self, micros: i64) -> Self {
        Self::clamped(self.0.saturating_add(micros))
    }

    const fn clamped(micros: i64) -> Self {
        Self(if micros < MIN_MICROS {
            MIN_MICROS
        } else if micros > MAX_MICROS {
            MAX_MICROS
        } else {
            micros
        })
    }

    fn checked(micros: i64) -> Option<Self> {
        (MIN_MICROS..=MAX_MICROS).contains(&micros).then_some(Self(micros))
    }
}

fn micros(duration: Duration) -> i64 {
    i64::try_from(duration.as_micros()).unwrap_or(i64::MAX)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let (era, year_of_era) = (year.div_euclid(400), year.rem_euclid(400));
    let day_of_year = (153 * ((i64::from(month) + 9) % 12) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * DAYS_PER_ERA + day_of_era - ERA_DAYS_BEFORE_UNIX_EPOCH
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + ERA_DAYS_BEFORE_UNIX_EPOCH;
    let (era, day_of_era) = (days.div_euclid(DAYS_PER_ERA), days.rem_euclid(DAYS_PER_ERA));
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / (DAYS_PER_ERA - 1)) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = era * 400 + year_of_era + i64::from(month <= 2);
    (year as i32, month as u32, (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32)
}

impl Add<Duration> for DateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        self.shifted(micros(duration))
    }
}

impl AddAssign<Duration> for DateTime {
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl Sub<Duration> for DateTime {
    type Output = Self;

    fn sub(self, duration: Duration) -> Self {
        self.shifted(-micros(duration))
    }
}

impl Sub for DateTime {
    type Output = Duration;

    fn sub(self, earlier: Self) -> Duration {
        Duration::from_micros(u64::try_from(self.0.saturating_sub(earlier.0)).unwrap_or_default())
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Rfc3339(*self, SecondsFormat::Auto).fmt(f)
    }
}

impl Debug for DateTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime {
        DateTime::parse_rfc3339(text).unwrap()
    }

    #[test]
    fn test_from_timestamp() {
        assert_eq!(DateTime::from_timestamp(1_424_301_369), at("2015-02-18T23:16:09Z"));
        assert_eq!(DateTime::from_timestamp(-1), at("1969-12-31T23:59:59Z"));
        assert_eq!(DateTime::from_timestamp_millis(1_424_301_369_150), at("2015-02-18T23:16:09.150Z"));
        assert_eq!(DateTime::from_timestamp(i64::MAX), at("9999-12-31T23:59:59.999999Z"));
        assert_eq!(DateTime::from_timestamp_millis(i64::MIN), at("0000-01-01T00:00:00Z"));
        assert_eq!(DateTime::checked_from_timestamp(253_402_300_799), Some(at("9999-12-31T23:59:59Z")));
        assert_eq!(DateTime::checked_from_timestamp(253_402_300_800), None);
        assert_eq!(DateTime::checked_from_timestamp_millis(-62_167_219_200_000), Some(at("0000-01-01T00:00:00Z")));
        assert_eq!(DateTime::checked_from_timestamp_millis(-62_167_219_200_001), None);
        assert_eq!(DateTime::checked_from_timestamp(i64::MAX), None);
        assert_eq!(DateTime::from_timestamp_micros(-1).timestamp(), -1);
        assert_eq!(DateTime::from_timestamp_micros(-1).timestamp_millis(), -1);
        assert_eq!(at("2015-02-18T23:16:09.150Z").timestamp(), 1_424_301_369);
        assert_eq!(at("2015-02-18T23:16:09.150Z").timestamp_millis(), 1_424_301_369_150);
        assert_eq!(DateTime::default(), DateTime::UNIX_EPOCH);
    }

    #[test]
    fn test_from_date() {
        assert_eq!(DateTime::from_date(1970, 1, 1), Some(DateTime::UNIX_EPOCH));
        assert_eq!(DateTime::from_date(2000, 2, 29), Some(at("2000-02-29T00:00:00Z")));
        assert_eq!(DateTime::from_date(2024, 2, 29), Some(at("2024-02-29T00:00:00Z")));
        assert_eq!(DateTime::from_date(1900, 2, 29), None);
        assert_eq!(DateTime::from_date(2023, 2, 29), None);
        assert_eq!(DateTime::from_date(2024, 4, 31), None);
        assert_eq!(DateTime::from_date(2024, 0, 1), None);
        assert_eq!(DateTime::from_date(2024, 13, 1), None);
        assert_eq!(DateTime::from_date(2024, 1, 0), None);
        assert_eq!(DateTime::from_date(0, 1, 1), Some(at("0000-01-01T00:00:00Z")));
        assert_eq!(DateTime::from_date(9999, 12, 31), Some(at("9999-12-31T00:00:00Z")));
        assert_eq!(DateTime::from_date(-1, 12, 31), None);
        assert_eq!(DateTime::from_date(10_000, 1, 1), None);
        assert_eq!(DateTime::from_date(i32::MAX, 1, 1), None);
    }

    #[test]
    fn test_date() {
        assert_eq!(DateTime::UNIX_EPOCH.date(), (1970, 1, 1));
        assert_eq!(DateTime::from_timestamp_micros(-1).date(), (1969, 12, 31));
        assert_eq!(at("2024-02-29T23:59:59.999999Z").date(), (2024, 2, 29));
        assert_eq!(at("2024-03-01T00:00:00Z").date(), (2024, 3, 1));
        assert_eq!((at("1970-01-01T02:00:00Z") - Duration::from_hours(3)).date(), (1969, 12, 31));
        let (first, last) = (days_from_civil(-9999, 1, 1), days_from_civil(9999, 12, 31));
        assert_eq!((civil_from_days(first), civil_from_days(last)), ((-9999, 1, 1), (9999, 12, 31)));
        assert_eq!(last - days_from_civil(1, 1, 1), 3_652_058);
        assert!((first..last).all(|days| {
            let (year, month, day) = civil_from_days(days);
            days_from_civil(year, month, day) == days && civil_from_days(days + 1) > (year, month, day)
        }));
    }

    #[test]
    fn test_elapsed() {
        assert!((DateTime::now() - Duration::from_hours(1)).elapsed() >= Duration::from_hours(1));
        assert_eq!((DateTime::now() + Duration::from_hours(1)).elapsed(), Duration::ZERO);
    }

    #[test]
    fn test_truncated() {
        let day = Duration::from_hours(24);
        assert_eq!(at("2026-09-23T15:08:42.5Z").truncated(Duration::from_hours(1)), at("2026-09-23T15:00:00Z"));
        assert_eq!(at("2026-09-23T15:08:42.5Z").truncated(day), at("2026-09-23T00:00:00Z"));
        assert_eq!(at("1969-12-31T23:30:00Z").truncated(day), at("1969-12-31T00:00:00Z"));
        assert_eq!(at("2026-09-23T15:08:42.5Z").truncated(Duration::ZERO), at("2026-09-23T15:08:42.5Z"));
    }

    #[test]
    fn test_to_local() {
        assert_eq!(at("2026-09-23T22:30:00Z").to_local(5 * 3_600).date(), (2026, 9, 24));
        assert_eq!(at("2026-09-23T02:30:00Z").to_local(-8 * 3_600).date(), (2026, 9, 22));
        assert_eq!(at("2026-09-23T02:30:00Z").to_local(0), at("2026-09-23T02:30:00Z"));
    }

    #[test]
    fn test_arithmetic() {
        let date = at("2026-09-23T15:00:00Z");
        assert_eq!(date + Duration::from_hours(24 * 8), at("2026-10-01T15:00:00Z"));
        assert_eq!(date - Duration::from_hours(16), at("2026-09-22T23:00:00Z"));
        assert_eq!(date - Duration::from_millis(1), at("2026-09-23T14:59:59.999Z"));
        assert_eq!(date - at("2026-09-22T15:00:00Z"), Duration::from_hours(24));
        assert_eq!(at("2026-09-22T15:00:00Z") - date, Duration::ZERO);
        assert_eq!(at("2026-09-22T15:00:00Z").abs_diff(date), Duration::from_hours(24));
        assert_eq!(date.abs_diff(at("2026-09-22T15:00:00Z")), Duration::from_hours(24));
        assert_eq!(at("9999-12-31T23:59:59Z") + Duration::from_secs(1), at("9999-12-31T23:59:59.999999Z"));
        assert_eq!(DateTime::UNIX_EPOCH + Duration::MAX, at("9999-12-31T23:59:59.999999Z"));
        assert_eq!(DateTime::UNIX_EPOCH - Duration::MAX, at("0000-01-01T00:00:00Z"));
        assert_eq!(at("0000-01-01T00:00:00Z").to_local(-3_600), at("0000-01-01T00:00:00Z"));
    }
}
