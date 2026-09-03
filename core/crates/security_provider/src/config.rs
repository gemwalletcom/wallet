use std::time::Duration;

use gem_client::RemoteProviderConfig;

pub struct ScanProviderRemoteConfig {
    pub url: String,
    pub public_key: String,
    pub secret_key: String,
}

pub struct AddressScanProviderConfig {
    pub timeout: Duration,
    pub goplus: ScanProviderRemoteConfig,
    pub hashdit: ScanProviderRemoteConfig,
}

pub struct TokenScanProviderConfig {
    pub timeout: Duration,
    pub goplus: ScanProviderRemoteConfig,
    pub hashdit: ScanProviderRemoteConfig,
    pub jupiter: RemoteProviderConfig,
}
