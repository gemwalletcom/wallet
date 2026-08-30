use super::model::GemLockPeriod;

const MILLISECONDS_PER_MINUTE: u32 = 60 * 1_000;

pub fn lock_periods() -> Vec<GemLockPeriod> {
    vec![
        GemLockPeriod::Immediate,
        GemLockPeriod::OneMinute,
        GemLockPeriod::FiveMinutes,
        GemLockPeriod::FifteenMinutes,
        GemLockPeriod::OneHour,
        GemLockPeriod::SixHours,
    ]
}

pub fn default_lock_period() -> GemLockPeriod {
    GemLockPeriod::OneMinute
}

pub fn lock_period_minutes(period: GemLockPeriod) -> u32 {
    match period {
        GemLockPeriod::Immediate => 0,
        GemLockPeriod::OneMinute => 1,
        GemLockPeriod::FiveMinutes => 5,
        GemLockPeriod::FifteenMinutes => 15,
        GemLockPeriod::OneHour => 60,
        GemLockPeriod::SixHours => 6 * 60,
    }
}

pub fn lock_period_milliseconds(period: GemLockPeriod) -> u32 {
    lock_period_minutes(period) * MILLISECONDS_PER_MINUTE
}

pub fn lock_period_from_minutes(minutes: u32) -> GemLockPeriod {
    lock_periods()
        .into_iter()
        .find(|period| lock_period_minutes(*period) == minutes)
        .unwrap_or_else(default_lock_period)
}

pub fn should_relock(elapsed_milliseconds: i64, lock_interval_minutes: u32, auth_required: bool, has_pending_request: bool) -> bool {
    let period = lock_period_from_minutes(lock_interval_minutes);
    auth_required && !has_pending_request && elapsed_milliseconds > i64::from(lock_period_milliseconds(period))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_periods_carry_the_same_minutes_on_both_platforms() {
        let minutes: Vec<u32> = lock_periods().into_iter().map(lock_period_minutes).collect();
        assert_eq!(minutes, vec![0, 1, 5, 15, 60, 360]);
        assert_eq!(lock_period_milliseconds(GemLockPeriod::SixHours), 21_600_000);
        assert_eq!(lock_period_from_minutes(15), GemLockPeriod::FifteenMinutes);
        assert_eq!(lock_period_from_minutes(7), default_lock_period(), "an unknown stored value falls back to the default");
    }

    #[test]
    fn test_relock_needs_auth_no_pending_request_and_an_elapsed_period() {
        assert!(should_relock(60_001, 1, true, false));
        assert!(!should_relock(60_000, 1, true, false), "the period has to be exceeded, not merely reached");
        assert!(!should_relock(60_001, 1, false, false), "no lock when authentication is off");
        assert!(!should_relock(60_001, 1, true, true), "a request in flight holds the lock off");
        assert!(should_relock(1, 0, true, false), "immediate locks as soon as any time has passed");
        assert!(!should_relock(0, 0, true, false));
        assert!(should_relock(60_001, 7, true, false), "an unknown stored interval falls back to the default period");
    }
}
