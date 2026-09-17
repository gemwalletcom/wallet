use config::{Config as FileConfig, File, FileFormat};
use primitives::{Chain, MINUTE, NodeCheckProfile};
use serde_json::json;

use crate::config::path::PathAllowlist;
use crate::config::routes::{EndpointConfig, RouteConfig, Selection};
use crate::config::{
    AllowlistConfig, CacheConfig, ChainConfig, ChainTypesConfig, Config, ErrorMatcherConfig, FailureTriggerConfig, MemoryConfig, MetricsConfig, MonitoringConfig, RetryConfig,
    RoutesConfig, Url,
};

impl Config {
    pub fn mock() -> Self {
        let mut config: Config = FileConfig::builder()
            .add_source(File::from_str(include_str!("../../config.yml"), FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        config.chains = Some(
            FileConfig::builder()
                .add_source(File::from_str(include_str!("../../chains.yml"), FileFormat::Yaml))
                .build()
                .unwrap()
                .try_deserialize()
                .unwrap(),
        );
        config.routes = Some(
            FileConfig::builder()
                .add_source(File::from_str(include_str!("../../routes.yml"), FileFormat::Yaml))
                .build()
                .unwrap()
                .try_deserialize()
                .unwrap(),
        );
        config
    }
}

impl RoutesConfig {
    pub fn mock() -> Self {
        FileConfig::builder()
            .add_source(File::from_str(include_str!("../../testdata/route_headers.yml"), FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}

impl RouteConfig {
    pub fn mock() -> Self {
        Self {
            group: "indexer".to_string(),
            selection: Selection::Ordered,
            headers: None,
            allowlist: PathAllowlist::default(),
            cache: Vec::new(),
            rate: None,
            retry: None,
            endpoints: Vec::new(),
        }
    }
}

impl EndpointConfig {
    pub fn mock() -> Self {
        Self {
            name: "key_1".to_string(),
            url: "https://api.blockscout.com".to_string(),
            headers: None,
            query: None,
            proxy: None,
        }
    }
}

impl Url {
    pub fn mock(url: &str) -> Self {
        Self {
            url: url.to_string(),
            headers: None,
        }
    }
}

impl ChainConfig {
    pub fn mock(chain: Chain) -> Self {
        Self {
            chain,
            poll_interval_seconds: None,
            latency: None,
            overrides: None,
            allowlist: None,
            urls: vec![Url::mock("https://example.com")],
        }
    }
}

impl ChainTypesConfig {
    pub fn mock() -> Self {
        serde_json::from_value(json!({
            "ethereum": {
                "allowlist": [
                    { "rpc_method": "eth_chainId" }
                ],
                "cache": [
                    { "path": "/api/v1/data", "method": "GET", "ttl": "5m" },
                    { "rpc_method": "eth_blockNumber", "ttl": "1m" }
                ]
            }
        }))
        .unwrap()
    }
}

impl AllowlistConfig {
    pub fn mock() -> Self {
        serde_json::from_value(json!([
            { "rpc_method": "eth_call" },
            { "rpc_method": "eth_chainId" },
            { "path": "/api/v2/address/**", "method": "GET" },
            { "path": "/api/v2/sendtx/", "method": "POST" }
        ]))
        .unwrap()
    }
}

impl PathAllowlist {
    pub fn mock() -> Self {
        serde_json::from_value(json!([
            { "path": "/quote/v2", "method": "POST" },
            { "path": "/chains", "method": "GET" },
            { "path": "/api/v2/address/**", "method": "GET" }
        ]))
        .unwrap()
    }
}

impl CacheConfig {
    pub fn mock() -> Self {
        Self {
            memory: MemoryConfig { max: 64_000_000 },
        }
    }
}

impl MonitoringConfig {
    pub fn mock() -> Self {
        Self {
            enabled: true,
            profile: NodeCheckProfile::Basic,
            interval: MINUTE * 10,
            trigger: FailureTriggerConfig::mock(),
        }
    }
}

impl FailureTriggerConfig {
    pub fn mock() -> Self {
        Self {
            failures: 15,
            rate: 50,
            window: MINUTE,
            latency: None,
        }
    }
}

impl RetryConfig {
    pub fn mock() -> Self {
        Self { enabled: true, ..Self::default() }
    }

    pub fn mock_with_errors(status_codes: Vec<u16>, error_messages: Vec<&str>) -> Self {
        Self {
            errors: ErrorMatcherConfig {
                status_codes,
                error_messages: error_messages.into_iter().map(|value| value.to_string()).collect(),
            },
            ..Self::mock()
        }
    }
}

impl MetricsConfig {
    pub fn mock() -> Self {
        Self {
            prefix: "dynode".to_string(),
            source: "public".to_string(),
        }
    }
}
