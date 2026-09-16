use std::collections::HashMap;

use primitives::Chain;
use serde_json::json;

use crate::config::{ChainConfig, Config, RoutesConfig, Url};
use crate::server::Server;

pub const TEST_REQUEST_LIMIT: usize = 8;

impl Server {
    pub fn mock_nodes() -> Self {
        let mut config = Config::mock();
        let settings = config.chains.as_mut().unwrap();
        settings.request.limit = TEST_REQUEST_LIMIT;
        settings.monitoring.enabled = false;
        config.routes = None;
        let chain = ChainConfig {
            urls: vec![Url::mock("https://upstream.example.invalid")],
            ..ChainConfig::mock(Chain::Ethereum)
        };
        Self::new(config, HashMap::from([(Chain::Ethereum, chain)])).unwrap()
    }

    pub fn mock_egress() -> Self {
        let mut config = Config::mock();
        config.chains = None;
        let mut settings = RoutesConfig::mock();
        settings.request.limit = TEST_REQUEST_LIMIT;
        settings.routes.get_mut("security_public").unwrap().allowlist = serde_json::from_value(json!([{ "path": "/allowed", "method": "GET" }])).unwrap();
        config.routes = Some(settings);
        Self::new(config, HashMap::new()).unwrap()
    }
}
