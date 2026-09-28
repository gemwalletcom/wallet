use std::time::Duration;

use gem_client::RemoteProviderConfig;
use settings::Security;

pub struct ScanProviderRemoteConfig {
    pub url: String,
    pub public_key: String,
    pub secret_key: String,
}

pub struct ScanProviderConfig {
    pub timeout: Duration,
    pub goplus: ScanProviderRemoteConfig,
    pub hashdit: RemoteProviderConfig,
    pub jupiter: RemoteProviderConfig,
    pub tronscan: RemoteProviderConfig,
}

impl ScanProviderConfig {
    pub fn new(security: &Security, timeout: Duration) -> Self {
        Self {
            timeout,
            goplus: ScanProviderRemoteConfig {
                url: security.goplus.url.clone(),
                public_key: security.goplus.key.public.clone(),
                secret_key: security.goplus.key.secret.clone(),
            },
            hashdit: security.hashdit.remote_provider_config(),
            jupiter: security.jupiter.remote_provider_config(),
            tronscan: security.tronscan.remote_provider_config(),
        }
    }
}
