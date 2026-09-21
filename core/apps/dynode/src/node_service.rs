use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    sync::Arc,
};

use gem_tracing::{DurationMs, info_with_fields};
use primitives::{Chain, ResponseError, ResponseResult, response::ErrorDetail};
use reqwest::StatusCode;
use serde_json::Value;
use settings_chain::BroadcastProviders;
use tokio::sync::RwLock;

use self::error::NodeServiceError;
use crate::BoxError;
use crate::cache::RequestCache;
use crate::config::{ChainConfig, ChainTypesConfig, ErrorMatcherConfig, HeadersConfig, MonitoringConfig, RetryConfig, Url};
use crate::failure_reason::FailureReason;
use crate::jsonrpc_types::{JsonRpcErrorResponse, RequestType};
use crate::metrics::Metrics;
use crate::monitoring::NodeMonitor;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::{CacheStatus, ProxyRequestService, ProxyResponse};
use crate::webhook::DynodeBroadcastWebhookClient;

mod error;

pub struct NodeService {
    pub(crate) chains: HashMap<Chain, ChainConfig>,
    nodes: Arc<RwLock<HashMap<Chain, Url>>>,
    metrics: Arc<Metrics>,
    chain_types: ChainTypesConfig,
    retry_config: RetryConfig,
    proxy: ProxyRequestService,
    broadcast_providers: Arc<BroadcastProviders>,
    node_monitor: NodeMonitor,
}

impl NodeService {
    pub fn new(
        chains: HashMap<Chain, ChainConfig>,
        metrics: Metrics,
        client: reqwest::Client,
        chain_types: ChainTypesConfig,
        cache: RequestCache,
        retry_config: RetryConfig,
        headers_config: HeadersConfig,
        broadcast_webhook: DynodeBroadcastWebhookClient,
        monitoring_config: MonitoringConfig,
    ) -> Self {
        let nodes = chains.values().filter_map(|config| config.urls.first().cloned().map(|url| (config.chain, url))).collect();

        metrics.initialize_transaction_broadcasts(chains.keys().copied());
        let broadcast_providers = Arc::new(BroadcastProviders::from_chains(chains.keys().copied()));
        let proxy = ProxyRequestService::new(metrics.clone(), cache, client, headers_config, broadcast_webhook, Arc::clone(&broadcast_providers));
        let nodes = Arc::new(RwLock::new(nodes));
        let metrics = Arc::new(metrics);
        let node_monitor = NodeMonitor::new(chains.values().cloned(), Arc::clone(&nodes), Arc::clone(&metrics), monitoring_config);

        Self {
            chains,
            nodes,
            metrics,
            chain_types,
            retry_config,
            proxy,
            broadcast_providers,
            node_monitor,
        }
    }

    pub fn start_monitoring(&mut self) {
        self.node_monitor.start();
    }

    pub async fn handle_request(&self, request: &ProxyRequest) -> Result<ProxyResponse, BoxError> {
        let chain = request.chain;
        let _inflight = self.metrics.track_node_inflight(chain);
        let mut remote_host = None;
        let result = self.handle_request_inner(request, &mut remote_host).await;
        if request.is_broadcast(&self.broadcast_providers) {
            self.metrics.record_transaction_broadcast(request, &result, &self.broadcast_providers, remote_host.as_deref().unwrap_or("unknown"));
        }
        let status = result.as_ref().map_or(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), |response| response.status);
        self.metrics.record_node_response(chain, &request.path, status);
        if let Ok(response) = &result {
            self.metrics.add_proxy_response(chain.as_ref(), request.method.as_str(), request.path.as_str(), response.status, request.elapsed().as_millis());
        }
        result
    }

    async fn handle_request_inner(&self, request: &ProxyRequest, broadcast_host: &mut Option<String>) -> Result<ProxyResponse, BoxError> {
        Self::log_incoming_request(request);

        let chain_config = self.get_chain_config(request)?;
        if !self.chain_types.allows(chain_config, request.request_type()) {
            return Self::request_not_allowed_response(request);
        }
        let Some(urls) = self.resolve_request_urls(chain_config, request).await else {
            return self.node_not_found_response(request);
        };
        if urls.len() == 1 {
            return self.proxy.handle_request(request, &urls[0], chain_config, broadcast_host).await;
        }

        let retry_enabled = self.retry_config.enabled;
        let mut last_error: Option<String> = None;
        let mut last_error_data: Option<Value> = None;
        let max_attempts = if retry_enabled { self.retry_config.effective_max_attempts(urls.len()) } else { 1 };

        for (index, url) in urls.iter().take(max_attempts).enumerate() {
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
            match self.proxy.handle_request(request, url, chain_config, broadcast_host).await {
                Ok(response) => {
                    let retry_error = self.matches_response_error_signal(request, &response, &self.retry_config.errors);
                    if !response.is_from_cache() {
                        self.report_active_node_outcome(index, request.chain, url, retry_error);
                    }
                    if !retry_error {
                        return Ok(response);
                    }

                    let upstream_data = serde_json::from_slice::<Value>(&response.body).ok();
                    if !retry_enabled {
                        return self.log_and_create_error_response(request, Some(remote_host.as_str()), NodeServiceError::UpstreamStatus(response.status), upstream_data);
                    }
                    let retry_reason = FailureReason::Status(response.status).to_string();
                    if index + 1 < max_attempts {
                        self.metrics.add_proxy_retry(request.chain.as_ref(), remote_host.as_str(), &retry_reason);
                        self.metrics.record_node_failover(request.chain, remote_host.as_str(), &request.path, &retry_reason);
                    }
                    last_error = Some(retry_reason);
                    last_error_data = upstream_data;
                }
                Err(e) => {
                    self.report_active_node_outcome(index, request.chain, url, true);
                    if !retry_enabled {
                        return Err(e);
                    }

                    let request_id = request.id.as_str();
                    let chain = request.chain.as_ref();
                    let latency = DurationMs(request.elapsed());
                    let retry_reason = FailureReason::from_error(e.as_ref()).to_string();
                    info_with_fields!("Upstream error", id = request_id, chain = chain, remote_host = remote_host.as_str(), error = retry_reason.as_str(), latency = latency,);
                    if index + 1 < max_attempts {
                        self.metrics.add_proxy_retry(request.chain.as_ref(), remote_host.as_str(), &retry_reason);
                        self.metrics.record_node_failover(request.chain, remote_host.as_str(), &request.path, &retry_reason);
                    }
                    last_error = Some(retry_reason);
                }
            }
        }

        if let Some(error) = last_error.as_deref() {
            info_with_fields!(
                &NodeServiceError::UpstreamsFailed.to_string(),
                id = request.id.as_str(),
                chain = request.chain.as_ref(),
                method = request.method.as_str(),
                uri = request.path.as_str(),
                error = error,
                latency = DurationMs(request.elapsed()),
            );
        }
        self.log_and_create_error_response(request, None, NodeServiceError::UpstreamsFailed, last_error_data)
    }

    fn report_active_node_outcome(&self, attempt_index: usize, chain: Chain, url: &Url, failed: bool) {
        if attempt_index == 0 {
            self.node_monitor.report(chain, url, failed);
        }
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

    fn get_chain_config(&self, request: &ProxyRequest) -> Result<&ChainConfig, NodeServiceError> {
        self.chains.get(&request.chain).ok_or(NodeServiceError::ChainNotConfigured(request.chain))
    }

    async fn resolve_request_urls(&self, chain_config: &ChainConfig, request: &ProxyRequest) -> Option<Vec<Url>> {
        if chain_config.urls.is_empty() {
            return None;
        }
        if chain_config.urls.len() == 1 {
            return Some(vec![chain_config.urls[0].clone()]);
        }

        let current_node = self.nodes.read().await.get(&chain_config.chain).cloned()?;
        Some(Self::get_ordered_urls(&chain_config.urls, &current_node, request.id.as_str()))
    }

    fn node_not_found_response(&self, request: &ProxyRequest) -> Result<ProxyResponse, BoxError> {
        self.log_and_create_error_response(request, None, NodeServiceError::NodeNotFound, None)
    }

    fn request_not_allowed_response(request: &ProxyRequest) -> Result<ProxyResponse, BoxError> {
        let error = NodeServiceError::RequestNotAllowed;
        let error_message = error.to_string();
        let status = error.status();
        info_with_fields!(
            error_message.as_str(),
            id = request.id.as_str(),
            chain = request.chain.as_ref(),
            method = request.method.as_str(),
            uri = request.path.as_str(),
            user_agent = request.user_agent.as_str(),
            request = &request.request_type().get_methods_list(),
        );

        let body = serde_json::to_vec(&ResponseResult::<()>::error(error_message))?;

        Ok(ProxyResponse::with_content_type(status.as_u16(), body, JSON_CONTENT_TYPE).with_proxy_headers(request.id.as_str(), request.elapsed(), CacheStatus::Miss))
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

    fn log_and_create_error_response(&self, request: &ProxyRequest, host: Option<&str>, error: NodeServiceError, upstream_data: Option<Value>) -> Result<ProxyResponse, BoxError> {
        let error_message = error.to_string();
        let request_id = request.id.as_str();
        let chain = request.chain.as_ref();
        let uri = request.path.as_str();
        let method = request.method.as_str();
        let remote_host = host.unwrap_or("none");
        let latency = DurationMs(request.elapsed());
        let status = error.status().as_u16();
        info_with_fields!(
            "Proxy response",
            id = request_id,
            chain = chain,
            remote_host = remote_host,
            method = method,
            uri = uri,
            status = status,
            error = error_message.as_str(),
            latency = latency,
        );

        let response_latency = request.elapsed();

        let response = match request.request_type() {
            RequestType::JsonRpc(_) => serde_json::to_value(JsonRpcErrorResponse::new(&error_message))?,
            RequestType::Regular { .. } => serde_json::to_value(ResponseError {
                error: ErrorDetail { message: error_message, data: upstream_data },
            })?,
        };

        let body = serde_json::to_vec(&response)?;

        Ok(ProxyResponse::with_content_type(status, body, JSON_CONTENT_TYPE).with_proxy_headers(request.id.as_str(), response_latency, CacheStatus::Miss))
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;
    use reqwest::{Method, header::HeaderMap};

    use super::*;
    use crate::config::Override;

    #[tokio::test]
    async fn test_broadcast_metrics_count_final_response_without_webhook() {
        let service = NodeService::mock(ChainConfig {
            urls: vec![],
            ..ChainConfig::mock(Chain::Ethereum)
        });
        let prefix = "dynode_transaction_broadcasts_total{";
        let before = service.metrics.get_metrics();
        let initial = before.lines().filter(|line| line.starts_with(prefix)).collect::<Vec<_>>();
        assert_eq!(initial.len(), 2);
        assert!(initial.iter().all(|line| line.ends_with(" 0")));

        service.handle_request(&ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_chainId")).await.unwrap();
        service.handle_request(&ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_sendRawTransaction")).await.unwrap();
        let encoded = service.metrics.get_metrics();
        assert_eq!(
            encoded.lines().filter(|line| line.starts_with(prefix) && !line.ends_with(" 0")).collect::<Vec<_>>(),
            vec!["dynode_transaction_broadcasts_total{source=\"public\",group=\"evm\",service=\"ethereum\",chain=\"ethereum\",outcome=\"failure\"} 1"]
        );
    }

    #[tokio::test]
    async fn test_broadcast_host_tracks_final_attempt_and_override() {
        for (urls, override_url, expected) in [
            (vec![], None, None),
            (vec!["http://127.0.0.1:9/secret-key"], None, Some("127.0.0.1")),
            (vec!["http://127.0.0.1:9"], Some("http://localhost:9/secret-key"), Some("localhost")),
            (vec!["http://127.0.0.1:9/secret-key", "http://localhost:9/other-key"], None, Some("localhost")),
        ] {
            let config = ChainConfig {
                urls: urls.into_iter().map(Url::mock).collect(),
                overrides: override_url.map(|url| {
                    vec![Override {
                        rpc_method: Some("eth_sendRawTransaction".to_string()),
                        path: None,
                        url: url.to_string(),
                    }]
                }),
                ..ChainConfig::mock(Chain::Ethereum)
            };
            let service = NodeService {
                retry_config: RetryConfig::mock(),
                ..NodeService::mock(config)
            };
            let request = ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_sendRawTransaction");
            let mut remote_host = None;
            let _ = service.handle_request_inner(&request, &mut remote_host).await;
            assert_eq!(remote_host.as_deref(), expected);
        }
    }

    #[test]
    fn test_get_chain_config_found() {
        let service = NodeService::mock(ChainConfig::mock(Chain::Bitcoin));
        let request = ProxyRequest::mock(Chain::Bitcoin, Method::POST, "/", &[]);

        let result = service.get_chain_config(&request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().chain, Chain::Bitcoin);
    }

    #[test]
    fn test_get_chain_config_not_found() {
        let service = NodeService::mock(ChainConfig::mock(Chain::Bitcoin));
        let request = ProxyRequest::mock(Chain::Ethereum, Method::POST, "/", &[]);

        let result = service.get_chain_config(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Chain ethereum not configured"));
    }

    #[test]
    fn test_matches_retry_status_codes() {
        let service = NodeService {
            retry_config: RetryConfig::mock_with_errors(vec![429], vec![]),
            ..NodeService::mock(ChainConfig::mock(Chain::Ethereum))
        };

        let request = ProxyRequest::mock(Chain::Ethereum, Method::POST, "/", &[]);
        let response = ProxyResponse::new(429, HeaderMap::new(), vec![]);

        assert!(service.matches_response_error_signal(&request, &response, &service.retry_config.errors));
    }

    #[test]
    fn test_matches_retry_jsonrpc_messages() {
        let service = NodeService {
            retry_config: RetryConfig::mock_with_errors(vec![], vec!["Exceeded the quota usage"]),
            ..NodeService::mock(ChainConfig::mock(Chain::Ethereum))
        };

        let request = ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_blockNumber");
        let response = ProxyResponse::new(200, HeaderMap::new(), br#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Exceeded the quota usage"},"id":1}"#.to_vec());

        assert!(service.matches_response_error_signal(&request, &response, &service.retry_config.errors));
    }

    #[tokio::test]
    async fn test_handle_request_denies_disallowed_jsonrpc_method() {
        let service = NodeService {
            chain_types: ChainTypesConfig::mock(),
            ..NodeService::mock(ChainConfig::mock(Chain::Ethereum))
        };
        let request = ProxyRequest::mock_jsonrpc(Chain::Ethereum, "unsupported_method");

        let response = service.handle_request(&request).await.unwrap();

        assert_eq!(response.status, StatusCode::FORBIDDEN.as_u16());
        assert_eq!(
            serde_json::from_slice::<Value>(&response.body).unwrap(),
            serde_json::json!({
                "error": { "message": NodeServiceError::RequestNotAllowed.to_string() }
            })
        );
    }

    #[tokio::test]
    async fn test_handle_request_allowed_jsonrpc_reaches_proxy_path() {
        let service = NodeService {
            chain_types: ChainTypesConfig::mock(),
            ..NodeService::mock(ChainConfig {
                urls: vec![Url::mock("http://127.0.0.1:9")],
                ..ChainConfig::mock(Chain::Ethereum)
            })
        };
        let request = ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_chainId");

        let result = service.handle_request(&request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_request_hides_upstream_url_on_retry_failure() {
        let service = NodeService {
            retry_config: RetryConfig::mock_with_errors(vec![500], vec![]),
            ..NodeService::mock(ChainConfig {
                urls: vec![Url::mock("http://127.0.0.1:9/secret-key"), Url::mock("http://127.0.0.1:10/other-secret-key")],
                ..ChainConfig::mock(Chain::Solana)
            })
        };
        let request = ProxyRequest::mock_jsonrpc(Chain::Solana, "getSlot");

        let response = service.handle_request(&request).await.unwrap();
        let body = serde_json::from_slice::<JsonRpcErrorResponse>(&response.body).unwrap();

        assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR.as_u16());
        assert_eq!(body.error.message, NodeServiceError::UpstreamsFailed.to_string());
    }
}
