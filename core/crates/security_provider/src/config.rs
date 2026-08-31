use std::time::Duration;

pub struct ScanProviderRemoteConfig {
    pub url: String,
    pub public_key: String,
    pub secret_key: String,
}

pub struct ScanProviderConfig {
    pub timeout: Duration,
    pub goplus: ScanProviderRemoteConfig,
    pub hashdit: ScanProviderRemoteConfig,
}
