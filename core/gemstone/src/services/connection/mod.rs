pub mod rules;

use crate::models::GemConnectionComponent;

#[derive(Default, uniffi::Object)]
pub struct GemConnectionService {}

#[uniffi::export]
impl GemConnectionService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn reconnect_delay_milliseconds(&self, attempt: u32) -> u64 {
        rules::reconnect_delay_milliseconds(attempt)
    }

    pub fn offline_debounce_milliseconds(&self) -> u64 {
        rules::offline_debounce_milliseconds()
    }

    pub fn resets_component_health(&self, component: GemConnectionComponent, is_healthy: bool, was_healthy: Option<bool>) -> bool {
        rules::resets_component_health(component, is_healthy, was_healthy)
    }
}
