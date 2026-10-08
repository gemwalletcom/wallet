use std::collections::{HashMap, HashSet};
use std::time::Duration;

use config_keys::{ConfigKey, ConfigParamKey};
use primitives::{ScanProvider, ScanType};
use storage::DatabaseError;

use crate::ConfigCacher;

pub(crate) struct ScanConfig {
    pub(crate) enforced: HashSet<ScanType>,
    pub(crate) enabled_providers: HashSet<ScanProvider>,
    pub(crate) safe_cache_durations: HashMap<ScanType, Duration>,
    pub(crate) detection_max_age: Duration,
    pub(crate) required_successes: usize,
}

impl ScanConfig {
    pub(crate) async fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        let mut enforced = HashSet::new();
        for scan_type in ScanType::all() {
            if config.get_param_bool(&ConfigParamKey::ScanTypeEnable(scan_type)).await? {
                enforced.insert(scan_type);
            }
        }
        let mut enabled_providers = HashSet::new();
        for provider in ScanProvider::all() {
            if config.get_param_bool(&ConfigParamKey::ScanProviderEnable(provider)).await? {
                enabled_providers.insert(provider);
            }
        }
        let mut safe_cache_durations = HashMap::new();
        for scan_type in ScanType::all().into_iter().filter(ScanType::is_safe_cacheable) {
            safe_cache_durations.insert(scan_type, config.get_param_duration(&ConfigParamKey::ScanSafeCacheDuration(scan_type)).await?);
        }
        Ok(Self {
            enforced,
            enabled_providers,
            safe_cache_durations,
            detection_max_age: config.get_duration(ConfigKey::ScanDetectionMaxAge).await?,
            required_successes: config.get_usize(ConfigKey::ScanRequiredSuccesses).await?,
        })
    }

    pub(crate) fn safe_cache_ttl(&self, scan_type: ScanType) -> u64 {
        self.safe_cache_durations.get(&scan_type).map(Duration::as_secs).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::testkit::MemoryConfigRepository;

    fn config(repository: MemoryConfigRepository) -> ConfigCacher {
        ConfigCacher::new(Arc::new(repository))
    }

    #[tokio::test]
    async fn test_from_config_defaults() {
        let scan = ScanConfig::from_config(&config(MemoryConfigRepository::new())).await.unwrap();

        for scan_type in ScanType::all() {
            assert_eq!(scan.enforced.contains(&scan_type), ConfigParamKey::ScanTypeEnable(scan_type).default_value() == "true");
            assert_eq!(scan.safe_cache_durations.contains_key(&scan_type), scan_type.is_safe_cacheable());
        }
        assert_eq!(scan.enabled_providers, ScanProvider::all().into_iter().collect());
    }

    #[tokio::test]
    async fn test_from_config_stored_values() {
        let scan_type = ScanType::all().into_iter().find(ScanType::is_safe_cacheable).unwrap();
        let repository = MemoryConfigRepository::new()
            .with_value(&ConfigParamKey::ScanTypeEnable(scan_type).key(), "false")
            .with_value(&ConfigParamKey::ScanProviderEnable(ScanProvider::Internal).key(), "false")
            .with_value(&ConfigParamKey::ScanSafeCacheDuration(scan_type).key(), "0s");

        let scan = ScanConfig::from_config(&config(repository)).await.unwrap();

        assert!(!scan.enforced.contains(&scan_type));
        assert!(!scan.enabled_providers.contains(&ScanProvider::Internal));
        assert_eq!(scan.safe_cache_ttl(scan_type), 0);
    }

    #[tokio::test]
    async fn test_from_config_malformed_value() {
        let repository = MemoryConfigRepository::new().with_value(&ConfigParamKey::ScanProviderEnable(ScanProvider::remote()[0]).key(), "maybe");

        assert!(ScanConfig::from_config(&config(repository)).await.is_err());
    }
}
