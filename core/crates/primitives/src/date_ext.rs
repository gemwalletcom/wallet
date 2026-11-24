use std::time::Duration as StdDuration;

pub trait DurationExt {
    fn days(days: u64) -> Self;
    fn as_days_ceil(&self) -> i64;
}

impl DurationExt for StdDuration {
    fn days(days: u64) -> Self {
        StdDuration::from_secs(days.saturating_mul(crate::duration::SECONDS_PER_DAY))
    }

    fn as_days_ceil(&self) -> i64 {
        let seconds_days = self.as_secs().div_ceil(crate::duration::SECONDS_PER_DAY);
        let extra_day = u64::from(self.subsec_nanos() > 0 && self.as_secs().is_multiple_of(crate::duration::SECONDS_PER_DAY));
        (seconds_days + extra_day) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HOUR, WEEK};

    #[test]
    fn test_days() {
        assert_eq!(StdDuration::days(7), WEEK);
        assert_eq!(StdDuration::days(u64::MAX), StdDuration::from_secs(u64::MAX));
    }

    #[test]
    fn test_as_days_ceil() {
        assert_eq!(StdDuration::from_secs(0).as_days_ceil(), 0);
        assert_eq!(StdDuration::from_secs(1).as_days_ceil(), 1);
        assert_eq!((HOUR * 12).as_days_ceil(), 1);
        assert_eq!((HOUR * 36).as_days_ceil(), 2);
        assert_eq!(WEEK.as_days_ceil(), 7);
        assert_eq!(StdDuration::from_secs_f64(7.1 * 24.0 * 60.0 * 60.0).as_days_ceil(), 8);
    }
}
