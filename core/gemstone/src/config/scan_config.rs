use std::time::Duration;

pub const TIMEOUT: Duration = Duration::from_secs(3);

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct ScanConfig {
    pub timeout_seconds: u32,
}

pub fn get_scan_config() -> ScanConfig {
    ScanConfig {
        timeout_seconds: TIMEOUT.as_secs() as u32,
    }
}
