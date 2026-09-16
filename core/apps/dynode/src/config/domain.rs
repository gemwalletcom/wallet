use std::time::Duration;

use primitives::Chain;
use serde::Deserialize;

use super::AllowlistConfig;
use super::MonitoringConfig;
use super::url::{Override, Url};

#[derive(Debug, Clone, Deserialize)]
pub struct ChainConfig {
    pub chain: Chain,
    pub poll_interval_seconds: Option<u64>,
    #[serde(default, deserialize_with = "serde_serializers::duration::deserialize_option")]
    pub latency: Option<Duration>,
    pub overrides: Option<Vec<Override>>,
    pub allowlist: Option<AllowlistConfig>,
    pub urls: Vec<Url>,
}

impl ChainConfig {
    pub fn monitoring_interval(&self, monitoring_config: &MonitoringConfig) -> Duration {
        self.poll_interval_seconds.map(Duration::from_secs).unwrap_or(monitoring_config.interval)
    }

    pub fn monitoring_latency(&self, monitoring_config: &MonitoringConfig) -> Option<Duration> {
        self.latency.or(monitoring_config.trigger.latency)
    }

    pub fn resolve_url(&self, base_url: &Url, rpc_method: Option<&str>, request_path: Option<&str>) -> Url {
        let Some(overrides) = &self.overrides else {
            return base_url.clone();
        };

        for override_config in overrides {
            let rpc_matches = override_config
                .rpc_method
                .as_ref()
                .is_none_or(|override_method| Some(override_method.as_str()) == rpc_method);

            let path_matches = override_config.path.as_ref().is_none_or(|override_path| Some(override_path.as_str()) == request_path);

            if rpc_matches && path_matches {
                return Url {
                    url: override_config.url.clone(),
                    headers: base_url.headers.clone(),
                };
            }
        }

        base_url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitoring_interval_uses_chain_override() {
        let chain_config = ChainConfig {
            poll_interval_seconds: Some(20),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        let config = MonitoringConfig {
            interval: Duration::from_secs(45),
            ..MonitoringConfig::mock()
        };
        assert_eq!(chain_config.monitoring_interval(&config), Duration::from_secs(20));
    }

    #[test]
    fn monitoring_interval_uses_global_fallback() {
        let chain_config = ChainConfig::mock(Chain::Ethereum);
        let config = MonitoringConfig {
            interval: Duration::from_secs(45),
            ..MonitoringConfig::mock()
        };
        assert_eq!(chain_config.monitoring_interval(&config), Duration::from_secs(45));
    }

    #[test]
    fn monitoring_latency_uses_chain_override() {
        let chain_config = ChainConfig {
            latency: Some(Duration::from_secs(3)),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        let mut config = MonitoringConfig::mock();
        config.trigger.latency = Some(Duration::from_secs(1));

        assert_eq!(chain_config.monitoring_latency(&config), Some(Duration::from_secs(3)));
    }

    #[test]
    fn monitoring_latency_uses_global_fallback() {
        let chain_config = ChainConfig::mock(Chain::Ethereum);
        let mut config = MonitoringConfig::mock();
        config.trigger.latency = Some(Duration::from_secs(1));

        assert_eq!(chain_config.monitoring_latency(&config), Some(Duration::from_secs(1)));
    }

    #[test]
    fn resolve_url_without_override() {
        let chain_config = ChainConfig::mock(Chain::Ethereum);
        let base_url = Url::mock("https://example.com/rpc");
        assert_eq!(chain_config.resolve_url(&base_url, Some("eth_sendTransaction"), None).url, "https://example.com/rpc");
    }

    #[test]
    fn resolve_url_with_rpc_method_override() {
        let chain_config = ChainConfig {
            overrides: Some(vec![Override {
                rpc_method: Some("eth_sendTransaction".to_string()),
                path: None,
                url: "https://tx-relay.example.com".to_string(),
            }]),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        let base_url = Url::mock("https://example.com/rpc");
        assert_eq!(chain_config.resolve_url(&base_url, Some("eth_sendTransaction"), None).url, "https://tx-relay.example.com");
    }

    #[test]
    fn resolve_url_with_rpc_method_and_path_override() {
        let chain_config = ChainConfig {
            overrides: Some(vec![Override {
                rpc_method: Some("eth_sendTransaction".to_string()),
                path: None,
                url: "https://tx-relay.example.com/tx/submit".to_string(),
            }]),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        let base_url = Url::mock("https://example.com/rpc");
        assert_eq!(
            chain_config.resolve_url(&base_url, Some("eth_sendTransaction"), None).url,
            "https://tx-relay.example.com/tx/submit"
        );
    }

    #[test]
    fn resolve_url_without_matching_override() {
        let chain_config = ChainConfig {
            overrides: Some(vec![Override {
                rpc_method: Some("eth_sendTransaction".to_string()),
                path: None,
                url: "https://tx-relay.example.com".to_string(),
            }]),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        let base_url = Url::mock("https://example.com/rpc");
        assert_eq!(chain_config.resolve_url(&base_url, Some("eth_blockNumber"), None).url, "https://example.com/rpc");
    }

    #[test]
    fn resolve_url_with_wildcard_override() {
        let chain_config = ChainConfig {
            overrides: Some(vec![Override {
                rpc_method: None,
                path: None,
                url: "https://fallback.example.com/v2/rpc".to_string(),
            }]),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        let base_url = Url::mock("https://example.com/rpc");
        assert_eq!(
            chain_config.resolve_url(&base_url, Some("eth_blockNumber"), None).url,
            "https://fallback.example.com/v2/rpc"
        );
    }

    #[test]
    fn resolve_url_with_path_override() {
        let chain_config = ChainConfig {
            overrides: Some(vec![Override {
                rpc_method: None,
                path: Some("/api/v1/block".to_string()),
                url: "https://api.example.com/v2/block".to_string(),
            }]),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        let base_url = Url::mock("https://example.com");
        assert_eq!(chain_config.resolve_url(&base_url, None, Some("/api/v1/block")).url, "https://api.example.com/v2/block");
    }

    #[test]
    fn resolve_url_preserves_headers() {
        let chain_config = ChainConfig {
            overrides: Some(vec![Override {
                rpc_method: Some("eth_sendTransaction".to_string()),
                path: None,
                url: "https://tx-relay.example.com".to_string(),
            }]),
            ..ChainConfig::mock(Chain::Ethereum)
        };
        let base_url = Url {
            headers: Some(std::collections::HashMap::from([("x-api-key".to_string(), "test123".to_string())])),
            ..Url::mock("https://example.com/rpc")
        };
        let resolved = chain_config.resolve_url(&base_url, Some("eth_sendTransaction"), None);
        assert_eq!(resolved.headers.as_ref().unwrap().get("x-api-key").unwrap(), "test123");
    }
}
