use super::model::{GemAuthPromptOutcome, GemLockPeriod};
use crate::constants::LOCK_PERIODS;

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

#[uniffi::export]
pub fn lock_period_from_minutes(minutes: Option<u32>) -> GemLockPeriod {
    minutes.and_then(|minutes| LOCK_PERIODS.iter().copied().find(|period| period.minutes() == minutes)).unwrap_or(GemLockPeriod::OneMinute)
}

pub(super) fn should_relock(elapsed_milliseconds: i64, lock_interval_minutes: u32, auth_required: bool) -> bool {
    let period = lock_period_from_minutes(Some(lock_interval_minutes));
    auth_required && elapsed_milliseconds > i64::from(period.milliseconds())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_a_recoverable_prompt_outcome_is_retried() {
        assert_eq!(GemAuthPromptOutcome::CancelledByUser.retry_delay_milliseconds(), Some(500));
        assert_eq!(GemAuthPromptOutcome::Transient.retry_delay_milliseconds(), Some(1_000));
        assert_eq!(GemAuthPromptOutcome::LockedOut.retry_delay_milliseconds(), Some(30_000));
        assert_eq!(GemAuthPromptOutcome::Unavailable.retry_delay_milliseconds(), None, "no enrolled biometry cannot be retried into working");
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
        let minutes: Vec<u32> = LOCK_PERIODS.iter().copied().map(GemLockPeriod::minutes).collect();
        assert_eq!(minutes, vec![0, 1, 5, 15, 60, 360]);
        assert_eq!(GemLockPeriod::SixHours.milliseconds(), 21_600_000);
        assert_eq!(lock_period_from_minutes(Some(15)), GemLockPeriod::FifteenMinutes);
        assert_eq!(lock_period_from_minutes(Some(7)), GemLockPeriod::OneMinute, "an unknown stored value falls back to the default");
        assert_eq!(lock_period_from_minutes(None), GemLockPeriod::OneMinute, "a missing stored value is the default");
    }

    #[test]
    fn test_relock_needs_auth_and_an_elapsed_period() {
        assert!(should_relock(60_001, 1, true));
        assert!(!should_relock(60_000, 1, true), "the period has to be exceeded, not merely reached");
        assert!(!should_relock(60_001, 1, false), "no lock when authentication is off");
        assert!(should_relock(1, 0, true), "immediate locks as soon as any time has passed");
        assert!(!should_relock(0, 0, true));
        assert!(should_relock(60_001, 7, true), "an unknown stored interval falls back to the default period");
        assert!(should_relock(i64::MAX, 60, true), "nothing in flight holds the lock off once the period has passed");
    }
}
