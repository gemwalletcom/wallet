pub mod model;
pub mod rules;

pub use model::{GemAuthPromptOutcome, GemLockPeriod};

#[derive(Default, uniffi::Object)]
pub struct GemSecurityService {}

#[uniffi::export]
impl GemSecurityService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn lock_periods(&self) -> Vec<GemLockPeriod> {
        rules::lock_periods()
    }

    pub fn default_lock_period(&self) -> GemLockPeriod {
        rules::default_lock_period()
    }

    pub fn lock_period_minutes(&self, period: GemLockPeriod) -> u32 {
        rules::lock_period_minutes(period)
    }

    pub fn lock_period_milliseconds(&self, period: GemLockPeriod) -> u32 {
        rules::lock_period_milliseconds(period)
    }

    pub fn lock_period_from_minutes(&self, minutes: u32) -> GemLockPeriod {
        rules::lock_period_from_minutes(minutes)
    }

    pub fn is_auth_cancelled(&self, outcome: GemAuthPromptOutcome) -> bool {
        rules::is_auth_cancelled(outcome)
    }

    pub fn auth_retry_delay_milliseconds(&self, outcome: GemAuthPromptOutcome) -> Option<u32> {
        rules::auth_retry_delay_milliseconds(outcome)
    }

    pub fn should_relock(&self, elapsed_milliseconds: i64, lock_interval_minutes: u32, auth_required: bool, has_pending_request: bool) -> bool {
        rules::should_relock(elapsed_milliseconds, lock_interval_minutes, auth_required, has_pending_request)
    }
}
