pub mod rules;

use std::time::Duration;

use primitives::ConnectionStatus;

use crate::models::GemConnectionComponent;

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemRefreshKind {
    Market,
    Wallet,
}

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

    pub fn ping_interval_milliseconds(&self) -> u64 {
        rules::ping_interval_milliseconds()
    }

    pub fn refresh_interval(&self, kind: GemRefreshKind, status: ConnectionStatus) -> Duration {
        rules::refresh_interval(kind, status)
    }

    pub fn resets_component_health(&self, component: GemConnectionComponent, is_healthy: bool, was_healthy: Option<bool>) -> bool {
        rules::resets_component_health(component, is_healthy, was_healthy)
    }
}
