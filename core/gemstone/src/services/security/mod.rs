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

    pub fn should_relock(&self, elapsed_milliseconds: i64, lock_interval_minutes: u32, auth_required: bool, has_pending_request: bool) -> bool {
        rules::should_relock(elapsed_milliseconds, lock_interval_minutes, auth_required, has_pending_request)
    }
}
