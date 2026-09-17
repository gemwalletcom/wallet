use std::collections::HashMap;

use primitives::Chain;
use serde_json::json;

use crate::cache::RequestCache;
use crate::config::routes::RouteConfig;
use crate::config::{CacheConfig, ChainConfig, ChainTypesConfig};

impl RequestCache {
    pub fn mock() -> Self {
        Self::for_chains(&CacheConfig::mock(), &ChainTypesConfig::mock(), [ChainConfig::mock(Chain::Ethereum)].iter())
    }

    pub fn mock_providers(config: &CacheConfig) -> Self {
        let enabled = RouteConfig {
            group: "evm".to_string(),
            cache: vec![serde_json::from_value(json!({ "path": "/info", "method": "POST", "params": { "type": "meta" }, "ttl": "1m" })).unwrap()],
            ..RouteConfig::mock()
        };
        let disabled = RouteConfig {
            group: "evm".to_string(),
            ..RouteConfig::mock()
        };
        Self::for_routes(config, &HashMap::from([("ethereum".to_string(), enabled), ("disabled".to_string(), disabled)]))
    }
}
