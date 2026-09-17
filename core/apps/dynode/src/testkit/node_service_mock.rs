use std::collections::HashMap;

use reqwest::header;

use crate::cache::RequestCache;
use crate::config::{ChainConfig, ChainTypesConfig, HeadersConfig, MonitoringConfig, RetryConfig};
use crate::metrics::Metrics;
use crate::node_service::NodeService;
use crate::webhook::DynodeBroadcastWebhookClient;

impl NodeService {
    pub fn mock(chain_config: ChainConfig) -> Self {
        Self::new(
            HashMap::from([(chain_config.chain, chain_config)]),
            Metrics::mock(),
            gem_client::reqwest_client(),
            ChainTypesConfig::default(),
            RequestCache::default(),
            RetryConfig::default(),
            HeadersConfig {
                forward: vec![header::CONTENT_TYPE.to_string()],
            },
            DynodeBroadcastWebhookClient::disabled(),
            MonitoringConfig {
                enabled: false,
                ..MonitoringConfig::mock()
            },
        )
    }
}
