use super::model::{GemAuthPromptOutcome, GemLockPeriod};

const MILLISECONDS_PER_MINUTE: u32 = 60 * 1_000;

#[uniffi::export]
impl GemLockPeriod {
    pub fn minutes(self) -> u32 {
        match self {
            Self::Immediate => 0,
            Self::OneMinute => 1,
            Self::FiveMinutes => 5,
            Self::FifteenMinutes => 15,
            Self::OneHour => 60,
            Self::SixHours => 6 * 60,
        }
    }
}

impl GemLockPeriod {
    fn milliseconds(self) -> u32 {
        self.minutes() * MILLISECONDS_PER_MINUTE
    }
}

#[uniffi::export]
impl GemAuthPromptOutcome {
    pub fn is_cancelled(self) -> bool {
        match self {
            Self::CancelledByUser | Self::CancelledBySystem => true,
            Self::Unavailable | Self::LockedOut | Self::Transient | Self::Failed => false,
        }
    }

    pub fn retry_delay_milliseconds(self) -> Option<u32> {
        match self {
            Self::CancelledByUser | Self::CancelledBySystem => Some(500),
            Self::Transient => Some(1_000),
            Self::LockedOut => Some(30_000),
            Self::Unavailable | Self::Failed => None,
        }
    }
}

fn lock_periods() -> Vec<GemLockPeriod> {
    vec![
        GemLockPeriod::Immediate,
        GemLockPeriod::OneMinute,
        GemLockPeriod::FiveMinutes,
        GemLockPeriod::FifteenMinutes,
        GemLockPeriod::OneHour,
        GemLockPeriod::SixHours,
    ]
}

fn default_lock_period() -> GemLockPeriod {
    GemLockPeriod::OneMinute
}

fn lock_period_from_minutes(minutes: u32) -> GemLockPeriod {
    lock_periods().into_iter().find(|period| period.minutes() == minutes).unwrap_or_else(default_lock_period)
}

pub(super) fn should_relock(elapsed_milliseconds: i64, lock_interval_minutes: u32, auth_required: bool, has_pending_request: bool) -> bool {
    let period = lock_period_from_minutes(lock_interval_minutes);
    auth_required && !has_pending_request && elapsed_milliseconds > i64::from(period.milliseconds())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_a_recoverable_prompt_outcome_is_retried() {
        assert_eq!(GemAuthPromptOutcome::CancelledByUser.retry_delay_milliseconds(), Some(500));
        assert_eq!(GemAuthPromptOutcome::Transient.retry_delay_milliseconds(), Some(1_000));
        assert_eq!(GemAuthPromptOutcome::LockedOut.retry_delay_milliseconds(), Some(30_000));
        assert_eq!(
            GemAuthPromptOutcome::Unavailable.retry_delay_milliseconds(),
            None,
            "no enrolled biometry cannot be retried into working"
        );
        assert_eq!(GemAuthPromptOutcome::Failed.retry_delay_milliseconds(), None);
    }

    #[test]
    fn test_cancellation_covers_both_the_user_and_the_system() {
        assert!(GemAuthPromptOutcome::CancelledByUser.is_cancelled());
        assert!(GemAuthPromptOutcome::CancelledBySystem.is_cancelled());
        assert!(!GemAuthPromptOutcome::LockedOut.is_cancelled());
        assert!(!GemAuthPromptOutcome::Failed.is_cancelled());
    }

    #[test]
    fn test_lock_periods_carry_the_same_minutes_on_both_platforms() {
        let minutes: Vec<u32> = lock_periods().into_iter().map(GemLockPeriod::minutes).collect();
        assert_eq!(minutes, vec![0, 1, 5, 15, 60, 360]);
        assert_eq!(GemLockPeriod::SixHours.milliseconds(), 21_600_000);
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
