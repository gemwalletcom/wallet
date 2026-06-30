use std::error::Error;
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    sync::Arc,
};

use reqwest::StatusCode;
use tokio::sync::RwLock;

use super::switch_reason::NodeSwitchReason;
use crate::cache::RequestCache;
use crate::config::{CacheConfig, ChainConfig, ChainTypesConfig, ErrorMatcherConfig, HeadersConfig, RetryConfig, Url};
use crate::jsonrpc_types::{JsonRpcErrorResponse, RequestType};
use crate::metrics::Metrics;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_builder::ProxyBuilder;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::response_builder::ResponseBuilder;
use crate::proxy::{NodeDomain, ProxyResponse};
use crate::webhook::DynodeBroadcastWebhookClient;
use gem_tracing::{DurationMs, info_with_fields};
use primitives::{Chain, ResponseError, response::ErrorDetail};
use serde_json::Value;
use settings_chain::BroadcastProviders;

const NODE_NOT_FOUND: &str = "Node not found";
const REQUEST_NOT_ALLOWED: &str = "Request not allowed";
const UPSTREAMS_FAILED: &str = "All upstream URLs failed";
const RETRY_REASON_REQUEST_ERROR: &str = "request_error";

#[derive(Clone)]
pub struct NodeService {
    pub chains: HashMap<Chain, ChainConfig>,
    pub nodes: Arc<RwLock<HashMap<Chain, NodeDomain>>>,
    pub metrics: Arc<Metrics>,
    chain_types: ChainTypesConfig,
    pub retry_config: RetryConfig,
    proxy_builder: ProxyBuilder,
}

impl NodeService {
    pub fn new(
        chains: HashMap<Chain, ChainConfig>,
        metrics: Metrics,
        client: reqwest::Client,
        chain_types: ChainTypesConfig,
        cache_config: CacheConfig,
        retry_config: RetryConfig,
        headers_config: HeadersConfig,
        broadcast_webhook: DynodeBroadcastWebhookClient,
    ) -> Self {
        let nodes = chains.values().map(|c| (c.chain, NodeDomain::new(c.urls.first().unwrap().clone(), c.clone()))).collect();

        let cache = RequestCache::new(cache_config, &chain_types, chains.values());
        let broadcast_providers = Arc::new(BroadcastProviders::from_chains(chains.keys().copied()));
        let proxy_builder = ProxyBuilder::new(metrics.clone(), cache, client, headers_config, broadcast_webhook, broadcast_providers);

        Self {
            chains,
            nodes: Arc::new(RwLock::new(nodes)),
            metrics: Arc::new(metrics),
            chain_types,
            retry_config,
            proxy_builder,
        }
    }

    pub async fn get_node_domain(nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>, chain: Chain) -> Option<NodeDomain> {
        nodes.read().await.get(&chain).cloned()
    }

    pub fn sync_current_node_metric(metrics: &Arc<Metrics>, chain: Chain, url: &Url) {
        metrics.set_node_host_current(chain.as_ref(), &url.host());
    }

    pub(crate) async fn switch_node_if_current(
        nodes: &Arc<RwLock<HashMap<Chain, NodeDomain>>>,
        metrics: &Arc<Metrics>,
        chain_config: &ChainConfig,
        expected_current: &Url,
        selected: &Url,
        reason: &NodeSwitchReason,
    ) -> Option<(String, String)> {
        let (old_host, new_host) = {
            let mut nodes_write = nodes.write().await;
            let active_node = nodes_write.get(&chain_config.chain)?;
            if active_node.url.url != expected_current.url || active_node.url.url == selected.url {
                return None;
            }

            let old_host = active_node.url.host();
            let new_host = selected.host();
            nodes_write.insert(chain_config.chain, NodeDomain::new(selected.clone(), chain_config.clone()));
            (old_host, new_host)
        };

        metrics.move_node_host_current(chain_config.chain.as_ref(), &old_host, &new_host);
        metrics.add_node_switch(chain_config.chain.as_ref(), &old_host, &new_host, &reason.metric_reason());
        Some((old_host, new_host))
    }

    pub async fn handle_request(&self, request: ProxyRequest) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        let request_for_metrics = request.clone();
        let result = self.handle_request_inner(request).await;
        if let Ok(response) = &result {
            self.metrics.add_proxy_response(
                request_for_metrics.chain.as_ref(),
                request_for_metrics.method.as_str(),
                request_for_metrics.path.as_str(),
                response.status,
                request_for_metrics.elapsed().as_millis(),
            );
        }
        result
    }

    async fn handle_request_inner(&self, request: ProxyRequest) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        Self::log_incoming_request(&request);

        let chain_config = self.get_chain_config(&request)?;
        if !self.chain_types.allows(chain_config, request.request_type()) {
            return Self::request_not_allowed_response(&request);
        }
        let Some(urls) = self.resolve_request_urls(chain_config, &request).await else {
            return self.node_not_found_response(&request);
        };
        if urls.len() == 1 {
            let primary = NodeDomain::new(urls[0].clone(), chain_config.clone());
            return self.proxy_builder.handle_request(request, &primary).await;
        }

        let retry_enabled = self.retry_config.enabled && urls.len() > 1;
        let mut last_error: Option<String> = None;
        let mut last_error_data: Option<Value> = None;
        let max_attempts = if retry_enabled { self.retry_config.effective_max_attempts(urls.len()) } else { 1 };

        for (index, url) in urls.iter().take(max_attempts).enumerate() {
            let node_domain = NodeDomain::new(url.clone(), chain_config.clone());
            let remote_host = url.host();
            if index > 0 {
                info_with_fields!(
                    "Retry attempt",
                    id = request.id.as_str(),
                    chain = request.chain.as_ref(),
                    attempt = index + 1,
                    remote_host = remote_host.as_str(),
                    reason = last_error.as_deref().unwrap_or(""),
                );
            }
            match self.proxy_builder.handle_request(request.clone(), &node_domain).await {
                Ok(response) => {
                    let retry_error = self.matches_response_error_signal(&request, &response, &self.retry_config.errors);
                    if !retry_error {
                        return Ok(response);
                    }

                    let upstream_data = serde_json::from_slice::<Value>(&response.body).ok();
                    if !retry_enabled {
                        return self.log_and_create_error_response(&request, Some(remote_host.as_str()), &format!("Upstream status code: {}", response.status), upstream_data);
                    }
                    let retry_reason = format!("status={}", response.status);
                    if index + 1 < max_attempts {
                        self.metrics.add_proxy_retry(request.chain.as_ref(), remote_host.as_str(), &retry_reason);
                    }
                    last_error = Some(retry_reason);
                    last_error_data = upstream_data;
                }
                Err(e) => {
                    if !retry_enabled {
                        return Err(e);
                    }

                    let request_id = request.id.as_str();
                    let chain = request.chain.as_ref();
                    let latency = DurationMs(request.elapsed());
                    let retry_reason = e.downcast_ref::<reqwest::Error>().map_or(RETRY_REASON_REQUEST_ERROR, |err| {
                        if err.is_timeout() {
                            "timeout"
                        } else if err.is_connect() {
                            "connect_error"
                        } else {
                            RETRY_REASON_REQUEST_ERROR
                        }
                    });
                    info_with_fields!(
                        "Upstream error",
                        id = request_id,
                        chain = chain,
                        remote_host = remote_host.as_str(),
                        error = retry_reason,
                        latency = latency,
                    );
                    if index + 1 < max_attempts {
                        self.metrics.add_proxy_retry(request.chain.as_ref(), remote_host.as_str(), retry_reason);
                    }
                    last_error = Some(retry_reason.to_string());
                }
            }
        }

        if let Some(error) = last_error.as_deref() {
            info_with_fields!(
                UPSTREAMS_FAILED,
                id = request.id.as_str(),
                chain = request.chain.as_ref(),
                method = request.method.as_str(),
                uri = request.path.as_str(),
                error = error,
                latency = DurationMs(request.elapsed()),
            );
        }
        self.log_and_create_error_response(&request, None, UPSTREAMS_FAILED, last_error_data)
    }

    fn log_incoming_request(request: &ProxyRequest) {
        let request_type = request.request_type();

        match request_type {
            RequestType::JsonRpc(_) => {
                info_with_fields!(
                    "Incoming request",
                    id = request.id.as_str(),
                    chain = request.chain.as_ref(),
                    method = request.method.as_str(),
                    uri = request.path.as_str(),
                    rpc_method = &request_type.get_methods_list(),
                    user_agent = request.user_agent.as_str(),
                );
            }
            RequestType::Regular { .. } => {
                info_with_fields!(
                    "Incoming request",
                    id = request.id.as_str(),
                    chain = request.chain.as_ref(),
                    method = request.method.as_str(),
                    uri = request.path.as_str(),
                    user_agent = request.user_agent.as_str(),
                );
            }
        }
    }

    fn get_chain_config(&self, request: &ProxyRequest) -> Result<&ChainConfig, Box<dyn Error + Send + Sync>> {
        self.chains.get(&request.chain).ok_or_else(|| format!("Chain {} not configured", request.chain).into())
    }

    async fn resolve_request_urls(&self, chain_config: &ChainConfig, request: &ProxyRequest) -> Option<Vec<Url>> {
        if chain_config.urls.is_empty() {
            return None;
        }
        if chain_config.urls.len() == 1 {
            return Some(vec![chain_config.urls[0].clone()]);
        }

        let current_node = NodeService::get_node_domain(&self.nodes, chain_config.chain).await?;
        Some(Self::get_ordered_urls(&chain_config.urls, &current_node.url, request.id.as_str()))
    }

    fn node_not_found_response(&self, request: &ProxyRequest) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        self.log_and_create_error_response(request, None, NODE_NOT_FOUND, None)
    }

    fn request_not_allowed_response(request: &ProxyRequest) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        info_with_fields!(
            REQUEST_NOT_ALLOWED,
            id = request.id.as_str(),
            chain = request.chain.as_ref(),
            method = request.method.as_str(),
            uri = request.path.as_str(),
            user_agent = request.user_agent.as_str(),
            request = &request.request_type().get_methods_list(),
        );

        let body = serde_json::to_vec(&serde_json::json!({
            "error": "Forbidden",
            "message": REQUEST_NOT_ALLOWED,
            "code": StatusCode::FORBIDDEN.as_u16()
        }))?;

        let headers = ResponseBuilder::create_proxy_headers(request.id.as_str(), request.elapsed());
        ResponseBuilder::build_with_headers(body, StatusCode::FORBIDDEN.as_u16(), JSON_CONTENT_TYPE, headers)
    }

    fn get_ordered_urls(urls: &[Url], current: &Url, request_id: &str) -> Vec<Url> {
        let mut ordered_urls = urls.to_vec();
        if let Some(current_index) = ordered_urls.iter().position(|url| *url == *current) {
            ordered_urls.swap(0, current_index);
        }

        Self::rotate_fallback_urls(&mut ordered_urls, request_id);
        ordered_urls
    }

    fn rotate_fallback_urls(urls: &mut [Url], request_id: &str) {
        if urls.len() <= 2 {
            return;
        }

        let mut hasher = DefaultHasher::new();
        request_id.hash(&mut hasher);
        let tail_len = urls.len() - 1;
        let offset = (hasher.finish() as usize) % tail_len;
        if offset > 0 {
            urls[1..].rotate_left(offset);
        }
    }

    fn matches_response_error_signal(&self, request: &ProxyRequest, response: &ProxyResponse, matcher: &ErrorMatcherConfig) -> bool {
        if matcher.matches_status(response.status) {
            return true;
        }

        match request.request_type() {
            RequestType::JsonRpc(_) if response.status == StatusCode::OK.as_u16() => {
                if let Ok(error_response) = serde_json::from_slice::<JsonRpcErrorResponse>(&response.body) {
                    return matcher.matches_message(&error_response.error.message);
                }
                false
            }
            _ => false,
        }
    }

    fn log_and_create_error_response(
        &self,
        request: &ProxyRequest,
        host: Option<&str>,
        error_message: &str,
        upstream_data: Option<Value>,
    ) -> Result<ProxyResponse, Box<dyn Error + Send + Sync>> {
        let request_id = request.id.as_str();
        let chain = request.chain.as_ref();
        let uri = request.path.as_str();
        let method = request.method.as_str();
        let remote_host = host.unwrap_or("none");
        let latency = DurationMs(request.elapsed());
        let status: u16 = 500;
        info_with_fields!(
            "Proxy response",
            id = request_id,
            chain = chain,
            remote_host = remote_host,
            method = method,
            uri = uri,
            status = status,
            error = error_message,
            latency = latency,
        );

        let proxy_headers = ResponseBuilder::create_proxy_headers(request.id.as_str(), request.elapsed());

        let response = match request.request_type() {
            RequestType::JsonRpc(_) => serde_json::to_value(JsonRpcErrorResponse::new(error_message))?,
            RequestType::Regular { .. } => serde_json::to_value(ResponseError {
                error: ErrorDetail {
                    message: error_message.to_string(),
                    data: upstream_data,
                },
            })?,
        };

        let body = serde_json::to_vec(&response)?;

        ResponseBuilder::build_with_headers(body, StatusCode::INTERNAL_SERVER_ERROR.as_u16(), JSON_CONTENT_TYPE, proxy_headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CacheConfig, ChainTypesConfig, MetricsConfig, Url};
    use crate::testkit::config as testkit;
    use primitives::Chain;
    use reqwest::{Method, header, header::HeaderMap};

    fn create_service(chains: HashMap<Chain, ChainConfig>) -> NodeService {
        create_service_with_retry(chains, testkit::retry_config(false, vec![], vec![]))
    }

    fn create_service_with_retry(chains: HashMap<Chain, ChainConfig>, retry_config: RetryConfig) -> NodeService {
        create_service_with_config(chains, retry_config, ChainTypesConfig::default())
    }

    fn create_service_with_config(chains: HashMap<Chain, ChainConfig>, retry_config: RetryConfig, chain_types: ChainTypesConfig) -> NodeService {
        let metrics = Metrics::new(MetricsConfig::default());
        let broadcast_webhook = DynodeBroadcastWebhookClient::disabled();

        NodeService::new(
            chains,
            metrics,
            reqwest::Client::new(),
            chain_types,
            CacheConfig::default(),
            retry_config,
            HeadersConfig {
                forward: vec![header::CONTENT_TYPE.to_string()],
                domains: HashMap::new(),
            },
            broadcast_webhook,
        )
    }

    fn create_chain_config(chain: Chain, url: &str) -> ChainConfig {
        ChainConfig {
            chain,
            poll_interval_seconds: None,
            overrides: None,
            allowlist: None,
            cache: None,
            urls: vec![Url {
                url: url.to_string(),
                headers: None,
            }],
        }
    }

    fn create_request(host: &str, chain: Chain) -> ProxyRequest {
        ProxyRequest::new(
            Method::POST,
            HeaderMap::new(),
            vec![],
            "/".to_string(),
            "/".to_string(),
            host.to_string(),
            "test".to_string(),
            chain,
        )
    }

    fn create_jsonrpc_request(chain: Chain, method: &str) -> ProxyRequest {
        ProxyRequest::new(
            Method::POST,
            HeaderMap::new(),
            format!(r#"{{"jsonrpc":"2.0","method":"{method}","params":[],"id":1}}"#).into_bytes(),
            "/".to_string(),
            "/".to_string(),
            "ethereum.example.com".to_string(),
            "test".to_string(),
            chain,
        )
    }

    fn ethereum_chain_types() -> ChainTypesConfig {
        serde_json::from_value(serde_json::json!({
            "ethereum": {
                "allowlist": [
                    { "rpc_method": "eth_chainId" }
                ]
            }
        }))
        .unwrap()
    }

    #[test]
    fn test_get_chain_config_found() {
        let chains = HashMap::from([(Chain::Bitcoin, create_chain_config(Chain::Bitcoin, "https://bitcoin.example.com"))]);
        let service = create_service(chains);
        let request = create_request("any.host.com", Chain::Bitcoin);

        let result = service.get_chain_config(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().chain, Chain::Bitcoin);
    }

    #[test]
    fn test_get_chain_config_not_found() {
        let chains = HashMap::from([(Chain::Bitcoin, create_chain_config(Chain::Bitcoin, "https://bitcoin.example.com"))]);
        let service = create_service(chains);
        let request = create_request("unknown", Chain::Ethereum);

        let result = service.get_chain_config(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Chain ethereum not configured"));
    }

    #[test]
    fn test_matches_retry_status_codes() {
        let chains = HashMap::from([(Chain::Ethereum, create_chain_config(Chain::Ethereum, "https://ethereum.example.com"))]);
        let service = create_service_with_retry(chains, testkit::retry_config(true, vec![429], vec![]));

        let request = create_request("ethereum.example.com", Chain::Ethereum);
        let response = ProxyResponse::new(429, HeaderMap::new(), vec![]);

        assert!(service.matches_response_error_signal(&request, &response, &service.retry_config.errors));
    }

    #[test]
    fn test_matches_retry_jsonrpc_messages() {
        let chains = HashMap::from([(Chain::Ethereum, create_chain_config(Chain::Ethereum, "https://ethereum.example.com"))]);
        let service = create_service_with_retry(chains, testkit::retry_config(true, vec![], vec!["Exceeded the quota usage"]));

        let request = ProxyRequest::new(
            Method::POST,
            HeaderMap::new(),
            br#"{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}"#.to_vec(),
            "/".to_string(),
            "/".to_string(),
            "ethereum.example.com".to_string(),
            "test".to_string(),
            Chain::Ethereum,
        );
        let response = ProxyResponse::new(
            200,
            HeaderMap::new(),
            br#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Exceeded the quota usage"},"id":1}"#.to_vec(),
        );

        assert!(service.matches_response_error_signal(&request, &response, &service.retry_config.errors));
    }

    #[tokio::test]
    async fn test_handle_request_denies_disallowed_jsonrpc_method() {
        let chains = HashMap::from([(Chain::Ethereum, create_chain_config(Chain::Ethereum, "https://ethereum.example.com"))]);
        let service = create_service_with_config(chains, testkit::retry_config(false, vec![], vec![]), ethereum_chain_types());
        let request = create_jsonrpc_request(Chain::Ethereum, "trace_replayTransaction");

        let response = service.handle_request(request).await.unwrap();

        assert_eq!(response.status, StatusCode::FORBIDDEN.as_u16());
        assert_eq!(
            serde_json::from_slice::<Value>(&response.body).unwrap(),
            serde_json::json!({
                "error": "Forbidden",
                "message": REQUEST_NOT_ALLOWED,
                "code": StatusCode::FORBIDDEN.as_u16()
            })
        );
    }

    #[tokio::test]
    async fn test_handle_request_allowed_jsonrpc_reaches_proxy_path() {
        let chains = HashMap::from([(Chain::Ethereum, create_chain_config(Chain::Ethereum, "http://127.0.0.1:9"))]);
        let service = create_service_with_config(chains, testkit::retry_config(false, vec![], vec![]), ethereum_chain_types());
        let request = create_jsonrpc_request(Chain::Ethereum, "eth_chainId");

        let result = service.handle_request(request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_request_hides_upstream_url_on_retry_failure() {
        let mut chain_config = create_chain_config(Chain::Solana, "http://127.0.0.1:9/secret-key");
        chain_config.urls.push(Url {
            url: "http://127.0.0.1:10/other-secret-key".to_string(),
            headers: None,
        });
        let chains = HashMap::from([(Chain::Solana, chain_config)]);
        let service = create_service_with_retry(chains, testkit::retry_config(true, vec![500], vec![]));
        let request = ProxyRequest::new(
            Method::POST,
            HeaderMap::new(),
            br#"{"jsonrpc":"2.0","method":"getSlot","params":[],"id":1}"#.to_vec(),
            "/".to_string(),
            "/".to_string(),
            "solana.example.com".to_string(),
            "test".to_string(),
            Chain::Solana,
        );

        let response = service.handle_request(request).await.unwrap();
        let body = serde_json::from_slice::<JsonRpcErrorResponse>(&response.body).unwrap();

        assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR.as_u16());
        assert_eq!(body.error.message, UPSTREAMS_FAILED);
    }
}
