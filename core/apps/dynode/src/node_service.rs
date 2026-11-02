use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    sync::Arc,
};

use bytes::Bytes;
use chain_providers::BroadcastProviders;
use futures::future::join_all;
use gem_tracing::{DurationMs, info_with_fields};
use http_server::method_label;
use primitives::{Chain, ResponseError, ResponseResult, response::ErrorDetail};
use reqwest::StatusCode;
use serde_json::Value;
use tokio::sync::RwLock;

use self::batch::BatchPart;
use self::error::NodeServiceError;
use self::upstream::Upstream;
use crate::BoxError;
use crate::cache::RequestCache;
use crate::config::{ChainConfig, ChainTypesConfig, HeadersConfig, MonitoringConfig, REJECTED_ROUTE, RetryConfig, Url};
use crate::failure_reason::FailureReason;
use crate::jsonrpc_types::{JsonRpcErrorResponse, RequestType};
use crate::metrics::Metrics;
use crate::monitoring::NodeMonitor;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::{CacheStatus, ProxyRequestService, ProxyResponse};
use crate::webhook::DynodeBroadcastWebhookClient;

mod batch;
mod error;
mod upstream;

const ERROR_EXECUTION_REVERTED: i64 = 3;
const EXECUTION_REVERTED: &str = "execution reverted";

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

    pub async fn proxy_request(&self, request: &ProxyRequest) -> Result<ProxyResponse, BoxError> {
        let chain = request.chain;
        let _inflight = self.metrics.track_node_inflight(chain);
        let route = self.route(request);
        let mut remote_host = None;
        let parts = route.as_ref().and_then(|_| self.batch_parts(request));
        let result = match (route.as_deref(), parts) {
            (Some(route), Some(parts)) => self.proxy_batch_parts(request, &parts, route).await,
            (route, _) => self.proxy_request_to_node(request, route, &mut remote_host).await,
        };
        if request.is_broadcast(&self.broadcast_providers) {
            self.metrics.record_transaction_broadcast(request, &result, &self.broadcast_providers, remote_host.as_deref().unwrap_or("unknown"));
        }
        let status = result.as_ref().map_or(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), |response| response.status);
        let route = route.as_deref().unwrap_or(REJECTED_ROUTE);
        self.metrics.record_node_response(chain, route, status);
        if let Ok(response) = &result {
            self.metrics.add_proxy_response(chain.as_ref(), method_label(&request.method), route, response.status, request.elapsed().as_millis());
        }
        result
    }

    fn route(&self, request: &ProxyRequest) -> Option<String> {
        let chain_config = self.get_chain_config(request).ok()?;
        self.chain_types.route(chain_config, request.request_type())
    }

    fn batch_parts(&self, request: &ProxyRequest) -> Option<Vec<BatchPart>> {
        BatchPart::split(request, self.get_chain_config(request).ok()?)
    }

    async fn proxy_batch_parts(&self, request: &ProxyRequest, parts: &[BatchPart], route: &str) -> Result<ProxyResponse, BoxError> {
        let responses = join_all(parts.iter().map(|part| async move { self.proxy_request_to_node(&part.request, Some(route), &mut None).await })).await;
        BatchPart::merge(request, parts, &responses)
    }

    async fn proxy_request_to_node(&self, request: &ProxyRequest, route: Option<&str>, broadcast_host: &mut Option<String>) -> Result<ProxyResponse, BoxError> {
        Self::log_incoming_request(request);

        let chain_config = self.get_chain_config(request)?;
        let Some(route) = route else {
            return Self::request_not_allowed_response(request);
        };
        let Some(urls) = self.request_urls(chain_config, request).await else {
            return self.node_not_found_response(request);
        };
        let upstreams = Upstream::plan(request, chain_config, &urls, &self.retry_config);
        if let ([upstream], [active_url]) = (upstreams.as_slice(), urls.as_slice()) {
            let url = upstream.request_url(request, active_url)?;
            return self.proxy.forward_request(request, &url, route, broadcast_host).await;
        }

        let retry_enabled = self.retry_config.enabled;
        let mut last_error: Option<String> = None;
        let mut last_error_data: Option<Value> = None;
        let mut original_host = String::new();
        let mut failed_host = String::new();
        let max_attempts = upstreams.len();

        for (index, upstream) in upstreams.into_iter().enumerate() {
            let upstream_url = upstream.request_url(request, &urls[0]);
            let remote_host = upstream_url.as_ref().ok().and_then(|url| url.url.host_str()).unwrap_or("unknown").to_string();
            if index == 0 {
                original_host.clone_from(&remote_host);
            }
            if index > 0 {
                info_with_fields!(
                    "Retry attempt",
                    id = request.id.as_str(),
                    chain = request.chain.as_ref(),
                    attempt = index + 1,
                    remote_host = remote_host.as_str(),
                    reason = last_error.as_deref().unwrap_or(""),
                    original_host = original_host.as_str(),
                    failed_host = failed_host.as_str(),
                );
            }
            let result = match upstream_url {
                Ok(url) => self.proxy.forward_request(request, &url, route, broadcast_host).await,
                Err(error) => Err(error.into()),
            };
            match result {
                Ok(response) => {
                    let retry_error = self.matches_response_error_signal(&response);
                    if !response.is_from_cache() {
                        self.report_active_node_outcome(upstream, request.chain, retry_error);
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
                        self.metrics.record_node_failover(request.chain, remote_host.as_str(), route, &retry_reason);
                    }
                    last_error = Some(retry_reason);
                    last_error_data = upstream_data;
                }
                Err(error) => {
                    self.report_active_node_outcome(upstream, request.chain, true);
                    if !retry_enabled {
                        return Err(error);
                    }

                    let request_id = request.id.as_str();
                    let chain = request.chain.as_ref();
                    let latency = DurationMs(request.elapsed());
                    let retry_reason = FailureReason::from_error(error.as_ref()).to_string();
                    let rpc_methods = request.request_type().get_methods_list();
                    let error_detail = FailureReason::error_detail(error.as_ref());
                    info_with_fields!(
                        "Upstream error",
                        id = request_id,
                        chain = chain,
                        remote_host = remote_host.as_str(),
                        method = request.method.as_str(),
                        rpc_method = rpc_methods.as_str(),
                        error = retry_reason.as_str(),
                        error_detail = error_detail.as_str(),
                        latency = latency,
                    );
                    if index + 1 < max_attempts {
                        self.metrics.add_proxy_retry(request.chain.as_ref(), remote_host.as_str(), &retry_reason);
                        self.metrics.record_node_failover(request.chain, remote_host.as_str(), route, &retry_reason);
                    }
                    last_error = Some(retry_reason);
                }
            }
            failed_host = remote_host;
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

    fn report_active_node_outcome(&self, upstream: Upstream, chain: Chain, failed: bool) {
        if let Some(url) = upstream.active_node() {
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

    async fn request_urls(&self, chain_config: &ChainConfig, request: &ProxyRequest) -> Option<Vec<Url>> {
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

        let body = Bytes::from(serde_json::to_vec(&ResponseResult::<()>::error(error_message))?);

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

    fn matches_response_error_signal(&self, response: &ProxyResponse) -> bool {
        if self.retry_config.errors.iter().any(|rule| rule.matches(response.status, None)) {
            return true;
        }

        let Ok(body) = serde_json::from_slice::<Value>(&response.body) else {
            return false;
        };
        let Some(error) = body.get("error") else {
            return false;
        };
        if is_contract_revert(error) {
            return false;
        }
        self.retry_config.errors.iter().any(|rule| rule.matches(response.status, error_message(error)))
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

        let body = Bytes::from(serde_json::to_vec(&response)?);

        Ok(ProxyResponse::with_content_type(status, body, JSON_CONTENT_TYPE).with_proxy_headers(request.id.as_str(), response_latency, CacheStatus::Miss))
    }
}

fn error_message(error: &Value) -> Option<&str> {
    error.as_str().or_else(|| error.get("message").and_then(Value::as_str))
}

fn is_contract_revert(error: &Value) -> bool {
    error.get("code").and_then(Value::as_i64) == Some(ERROR_EXECUTION_REVERTED) || error_message(error).is_some_and(|message| message.to_ascii_lowercase().starts_with(EXECUTION_REVERTED))
}

#[cfg(test)]
mod tests {
    use primitives::Chain;
    use reqwest::{Method, header::HeaderMap};

    use super::*;
    use crate::config::{AllowlistConfig, ErrorMatcherConfig, Override, RPC_ROUTE};
    use crate::testkit::server_mock::spawn_node;

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

        service.proxy_request(&ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_chainId")).await.unwrap();
        service.proxy_request(&ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_sendRawTransaction")).await.unwrap();
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
            (vec!["http://127.0.0.1:9"], Some("http://localhost:9/secret-key"), Some("127.0.0.1")),
            (vec!["http://127.0.0.1:9/secret-key", "http://localhost:9/other-key"], None, Some("localhost")),
            (vec!["http://127.0.0.1:9/secret-key", "http://localhost:9/other-key", "http://localhost:9/third-key"], None, Some("localhost")),
            (vec!["http://127.0.0.1:9/secret-key", "http://127.0.0.1:9/other-key"], Some("http://localhost:9/override-key"), Some("127.0.0.1")),
        ] {
            let config = ChainConfig {
                urls: urls.into_iter().map(Url::mock).collect(),
                overrides: override_url.map(|url| {
                    vec![Override {
                        rpc_method: Some("eth_sendRawTransaction".to_string()),
                        path: None,
                        url: url.to_string(),
                        fallback: true,
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
            let _ = service.proxy_request_to_node(&request, Some(RPC_ROUTE), &mut remote_host).await;
            assert_eq!(remote_host.as_deref(), expected);
        }
    }

    #[tokio::test]
    async fn test_failed_override_falls_back_to_nodes() {
        let node = spawn_node("node").await;
        let config = |fallback| ChainConfig {
            urls: vec![Url::mock(&node)],
            overrides: Some(vec![Override {
                rpc_method: Some("getTransaction".to_string()),
                path: None,
                url: "http://127.0.0.1:9".to_string(),
                fallback,
            }]),
            ..ChainConfig::mock(Chain::Solana)
        };
        let request = ProxyRequest::mock_jsonrpc(Chain::Solana, "getTransaction");

        for (fallback, retry_config, falls_back) in [(true, RetryConfig::mock(), true), (false, RetryConfig::mock(), false), (true, RetryConfig::default(), false)] {
            let service = NodeService {
                retry_config,
                ..NodeService::mock(config(fallback))
            };
            let result = service.proxy_request_to_node(&request, Some(RPC_ROUTE), &mut None).await;
            assert_eq!(result.is_ok_and(|response| response.status == StatusCode::OK.as_u16()), falls_back);
        }
    }

    #[tokio::test]
    async fn test_batch_calls_follow_their_overrides() {
        let node = spawn_node("node").await;
        let helius = spawn_node("override").await;
        let closed = "http://127.0.0.1:9";
        let config = |node: &str, url: &str| ChainConfig {
            urls: vec![Url::mock(node)],
            overrides: Some(vec![Override {
                rpc_method: Some("getProgramAccounts".to_string()),
                path: None,
                url: url.to_string(),
                fallback: false,
            }]),
            ..ChainConfig::mock(Chain::Solana)
        };
        let failed = NodeServiceError::UpstreamsFailed.to_string();

        for (methods, node, override_url, status, expected) in [
            (
                vec!["getProgramAccounts", "getProgramAccounts"],
                node.as_str(),
                helius.as_str(),
                StatusCode::OK,
                ["1 override getProgramAccounts batch", "2 override getProgramAccounts batch"].map(String::from).to_vec(),
            ),
            (
                vec!["getProgramAccounts", "getSlot", "getProgramAccounts"],
                node.as_str(),
                helius.as_str(),
                StatusCode::OK,
                ["1 override getProgramAccounts batch", "2 node getSlot single", "3 override getProgramAccounts batch"].map(String::from).to_vec(),
            ),
            (vec!["getProgramAccounts", "getSlot"], node.as_str(), closed, StatusCode::OK, vec![format!("1 {failed}"), "2 node getSlot single".to_string()]),
            (vec!["getProgramAccounts", "getSlot"], closed, closed, StatusCode::INTERNAL_SERVER_ERROR, vec![format!("1 {failed}"), format!("2 {failed}")]),
        ] {
            let response = NodeService::mock(config(node, override_url)).proxy_request(&ProxyRequest::mock_batch(Chain::Solana, &methods)).await.unwrap();
            let results = serde_json::from_slice::<Vec<Value>>(&response.body).unwrap();
            let results = results
                .iter()
                .map(|result| format!("{} {}", result["id"], result["result"].as_str().or(result["error"]["message"].as_str()).unwrap()))
                .collect::<Vec<_>>();
            assert_eq!(response.status, status.as_u16());
            assert_eq!(results, expected);
        }
    }

    #[tokio::test]
    async fn test_rejected_requests_share_one_metric_label() {
        let service = NodeService::mock(ChainConfig {
            allowlist: Some(AllowlistConfig::mock()),
            ..ChainConfig::mock(Chain::Bitcoin)
        });
        for path in ["/random/a1", "/random/b2"] {
            let response = service.proxy_request(&ProxyRequest::mock(Chain::Bitcoin, Method::GET, path, &[])).await.unwrap();
            assert_eq!(response.status, NodeServiceError::RequestNotAllowed.status());
        }

        let encoded = service.metrics.get_metrics();
        assert!(encoded.contains(r#"path="rejected""#));
        assert!(!encoded.contains("/random"));
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
            retry_config: RetryConfig::mock_with_errors(vec![ErrorMatcherConfig {
                status: Some(429),
                ..ErrorMatcherConfig::default()
            }]),
            ..NodeService::mock(ChainConfig::mock(Chain::Ethereum))
        };

        let response = ProxyResponse::new(429, HeaderMap::new(), Bytes::new());

        assert!(service.matches_response_error_signal(&response));
    }

    #[test]
    fn test_matches_retry_jsonrpc_messages() {
        let service = NodeService {
            retry_config: RetryConfig::mock_with_errors(vec![ErrorMatcherConfig {
                message: Some("Exceeded the quota usage".to_string()),
                ..ErrorMatcherConfig::default()
            }]),
            ..NodeService::mock(ChainConfig::mock(Chain::Ethereum))
        };

        let response = ProxyResponse::new(200, HeaderMap::new(), Bytes::from_static(br#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Exceeded the quota usage"},"id":1}"#));

        assert!(service.matches_response_error_signal(&response));
    }

    #[test]
    fn test_matches_retry_error_message_across_statuses() {
        let service = NodeService {
            retry_config: RetryConfig::mock_with_errors(vec![ErrorMatcherConfig {
                message: Some("502 Bad Gateway".to_string()),
                ..ErrorMatcherConfig::default()
            }]),
            ..NodeService::mock(ChainConfig::mock(Chain::Zcash))
        };
        let gateway_error = br#"{"error":"502 Bad Gateway invalid character '<' looking for beginning of value"}"#;

        for (status, body, retry) in [
            (200, gateway_error.as_slice(), true),
            (400, gateway_error.as_slice(), true),
            (500, gateway_error.as_slice(), true),
            (400, br#"{"error":"-26: txn-mempool-conflict"}"#.as_slice(), false),
            (400, br#"{"error":"transaction already in block chain"}"#.as_slice(), false),
            (400, br#"{"error":"-26: min relay fee not met"}"#.as_slice(), false),
            (400, br#"{"error":"invalid transaction"}"#.as_slice(), false),
        ] {
            let response = ProxyResponse::new(status, HeaderMap::new(), Bytes::copy_from_slice(body));
            assert_eq!(service.matches_response_error_signal(&response), retry);
        }
    }

    #[test]
    fn test_a_contract_revert_skips_the_message_rules() {
        let service = NodeService {
            retry_config: RetryConfig::mock_with_errors(vec![
                ErrorMatcherConfig {
                    status: Some(429),
                    ..ErrorMatcherConfig::default()
                },
                ErrorMatcherConfig {
                    message: Some("rate limit".to_string()),
                    ..ErrorMatcherConfig::default()
                },
                ErrorMatcherConfig {
                    message: Some("Unauthorized".to_string()),
                    ..ErrorMatcherConfig::default()
                },
            ]),
            ..NodeService::mock(ChainConfig::mock(Chain::Ethereum))
        };

        for (status, body, retry) in [
            (200, br#"{"error":{"code":3,"message":"execution reverted: Unauthorized","data":"0x"}}"#.as_slice(), false),
            (200, br#"{"error":{"code":3,"message":"rate limit"}}"#.as_slice(), false),
            (200, br#"{"error":{"code":-32000,"message":"Execution reverted: rate limit"}}"#.as_slice(), false),
            (429, br#"{"error":{"code":3,"message":"execution reverted"}}"#.as_slice(), true),
            (200, br#"{"error":{"code":-32000,"message":"rate limit exceeded"}}"#.as_slice(), true),
            (200, br#"{"error":{"code":-32001,"message":"Unauthorized"}}"#.as_slice(), true),
            (200, br#"{"error":"Unauthorized"}"#.as_slice(), true),
        ] {
            let response = ProxyResponse::new(status, HeaderMap::new(), Bytes::copy_from_slice(body));
            assert_eq!(service.matches_response_error_signal(&response), retry, "{}", String::from_utf8_lossy(body));
        }
    }

    #[test]
    fn test_matches_retry_combined_rules() {
        let service = NodeService {
            retry_config: RetryConfig::mock_with_errors(vec![
                ErrorMatcherConfig {
                    status: Some(429),
                    ..ErrorMatcherConfig::default()
                },
                ErrorMatcherConfig {
                    status: Some(400),
                    message: Some("502 Bad Gateway".to_string()),
                },
            ]),
            ..NodeService::mock(ChainConfig::mock(Chain::Ethereum))
        };

        for (status, body, retry) in [
            (429, b"rate limited".as_slice(), true),
            (400, br#"{"error":"502 Bad Gateway"}"#.as_slice(), true),
            (400, br#"{"error":{"message":"502 Bad Gateway"}}"#.as_slice(), true),
            (500, br#"{"error":"502 Bad Gateway"}"#.as_slice(), false),
            (400, br#"{"error":"invalid transaction"}"#.as_slice(), false),
            (400, br#"{"result":"502 Bad Gateway"}"#.as_slice(), false),
            (400, b"invalid json".as_slice(), false),
        ] {
            let response = ProxyResponse::new(status, HeaderMap::new(), Bytes::copy_from_slice(body));
            assert_eq!(service.matches_response_error_signal(&response), retry);
        }
    }

    #[tokio::test]
    async fn test_proxy_request_denies_disallowed_jsonrpc_method() {
        let service = NodeService {
            chain_types: ChainTypesConfig::mock(),
            ..NodeService::mock(ChainConfig::mock(Chain::Ethereum))
        };
        let request = ProxyRequest::mock_jsonrpc(Chain::Ethereum, "unsupported_method");

        let response = service.proxy_request(&request).await.unwrap();

        assert_eq!(response.status, StatusCode::FORBIDDEN.as_u16());
        assert_eq!(
            serde_json::from_slice::<Value>(&response.body).unwrap(),
            serde_json::json!({
                "error": { "message": NodeServiceError::RequestNotAllowed.to_string() }
            })
        );
    }

    #[tokio::test]
    async fn test_proxy_request_allowed_jsonrpc_reaches_proxy_path() {
        let service = NodeService {
            chain_types: ChainTypesConfig::mock(),
            ..NodeService::mock(ChainConfig {
                urls: vec![Url::mock("http://127.0.0.1:9")],
                ..ChainConfig::mock(Chain::Ethereum)
            })
        };
        let request = ProxyRequest::mock_jsonrpc(Chain::Ethereum, "eth_chainId");

        let result = service.proxy_request(&request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_proxy_request_hides_upstream_url_on_retry_failure() {
        let service = NodeService {
            retry_config: RetryConfig::mock_with_errors(vec![ErrorMatcherConfig {
                status: Some(500),
                ..ErrorMatcherConfig::default()
            }]),
            ..NodeService::mock(ChainConfig {
                urls: vec![Url::mock("http://127.0.0.1:9/secret-key"), Url::mock("http://127.0.0.1:10/other-secret-key")],
                ..ChainConfig::mock(Chain::Solana)
            })
        };
        let request = ProxyRequest::mock_jsonrpc(Chain::Solana, "getSlot");

        let response = service.proxy_request(&request).await.unwrap();
        let body = serde_json::from_slice::<JsonRpcErrorResponse>(&response.body).unwrap();

        assert_eq!(response.status, StatusCode::INTERNAL_SERVER_ERROR.as_u16());
        assert_eq!(body.error.message, NodeServiceError::UpstreamsFailed.to_string());
    }
}
