use std::sync::Arc;

use primitives::Chain;
use reqwest::Method;
use reqwest::header::HeaderMap;
use settings_chain::BroadcastProviders;

use crate::cache::RequestCache;
use crate::config::HeadersConfig;
use crate::metrics::Metrics;
use crate::proxy::{ProxyRequest, ProxyRequestService};
use crate::webhook::DynodeBroadcastWebhookClient;

impl ProxyRequest {
    pub fn mock(chain: Chain, method: Method, path: &str, body: &[u8]) -> Self {
        Self::new(method, HeaderMap::new(), body.to_vec(), path.to_string(), path.to_string(), "example.com".to_string(), "test-agent".to_string(), chain)
    }

    pub fn mock_jsonrpc(chain: Chain, method: &str) -> Self {
        Self::mock(chain, Method::POST, "/", format!(r#"{{"jsonrpc":"2.0","method":"{method}","params":[],"id":1}}"#).as_bytes())
    }
}

impl ProxyRequestService {
    pub fn mock(headers_config: HeadersConfig) -> Self {
        Self::new(
            Metrics::mock(),
            RequestCache::default(),
            gem_client::reqwest_client(),
            headers_config,
            DynodeBroadcastWebhookClient::disabled(),
            Arc::new(BroadcastProviders::from_chains([Chain::Ethereum])),
        )
    }
}
