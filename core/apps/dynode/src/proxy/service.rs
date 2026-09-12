use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use gem_tracing::{DurationMs, info_with_fields};
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use settings_chain::BroadcastProviders;

use crate::BoxError;
use crate::cache::RequestCache;
use crate::config::{ChainConfig, HeadersConfig, Url};
use crate::jsonrpc_types::{JsonRpcRequest, RequestType};
use crate::metrics::Metrics;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::jsonrpc::JsonRpcHandler;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::request_url::RequestUrl;
use crate::proxy::transport::{self, TransportError};
use crate::proxy::{CacheStatus, ProxyResponse};
use crate::webhook::DynodeBroadcastWebhookClient;

const GRPC_ACCEPT_ENCODING: HeaderName = HeaderName::from_static("grpc-accept-encoding");
const GRPC_CONTENT_TYPE: &str = "application/grpc";

pub struct ProxyRequestService {
    metrics: Metrics,
    cache: RequestCache,
    client: Client,
    forward_headers: HashSet<HeaderName>,
    broadcast_webhook: DynodeBroadcastWebhookClient,
    broadcast_providers: Arc<BroadcastProviders>,
}

impl ProxyRequestService {
    pub fn new(
        metrics: Metrics,
        cache: RequestCache,
        client: Client,
        headers_config: HeadersConfig,
        broadcast_webhook: DynodeBroadcastWebhookClient,
        broadcast_providers: Arc<BroadcastProviders>,
    ) -> Self {
        let forward_headers = headers_config.forward.iter().filter_map(|name| HeaderName::from_str(name).ok()).collect();

        Self {
            metrics,
            cache,
            client,
            forward_headers,
            broadcast_webhook,
            broadcast_providers,
        }
    }

    fn build_headers(&self, original: &HeaderMap) -> HeaderMap {
        let mut headers = transport::filter_headers(original, &self.forward_headers);

        if original
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with(GRPC_CONTENT_TYPE))
        {
            headers.insert(GRPC_ACCEPT_ENCODING, HeaderValue::from_static("identity"));
        }

        headers
    }

    pub async fn handle_request(&self, request: &ProxyRequest, active_url: &Url, chain_config: &ChainConfig) -> Result<ProxyResponse, BoxError> {
        let chain = request.chain;
        let request_type = request.request_type();

        let rpc_method = match request_type {
            RequestType::JsonRpc(JsonRpcRequest::Single(call)) => Some(call.method.as_str()),
            _ => None,
        };

        let resolved_url = chain_config.resolve_url(active_url, rpc_method, Some(&request.path));
        let url = RequestUrl::from_parts(resolved_url, &request.path_with_query);
        let headers = self.build_headers(&request.headers);

        let methods_for_metrics = request_type.get_methods_for_metrics();
        self.metrics.add_proxy_request(request.chain.as_ref(), &methods_for_metrics);

        if let RequestType::JsonRpc(rpc_request) = request_type {
            return JsonRpcHandler::handle_request(
                rpc_request,
                request,
                &self.cache,
                &self.metrics,
                &url,
                &self.client,
                &headers,
                &self.broadcast_webhook,
                &self.broadcast_providers,
            )
            .await;
        }

        let cache_ttl = self.cache.should_cache_request(&chain, request_type);
        let cache_key = cache_ttl.and_then(|_| request_type.cache_key(&request.host));
        if let Some(key) = &cache_key
            && let Some(response) = self.try_cache_hit(key, request, &methods_for_metrics).await
        {
            return Ok(response);
        }

        let upstream_request = url.build_request(&request.method, request.body.clone(), headers);
        let attempt_start = Instant::now();
        let result = transport::send(&self.client, upstream_request).await;
        self.metrics.record_node_upstream(
            request.chain,
            url.url.host_str().unwrap_or_default(),
            &request.path,
            result.as_ref().map_or(StatusCode::BAD_GATEWAY.as_u16(), |response| response.status),
            attempt_start.elapsed(),
        );
        let response = result.map_err(TransportError::into_inner)?;
        let status = response.status;
        let response_headers = response.headers;
        let body = response.body;

        let response_latency = request.elapsed();
        let headers = transport::filter_headers(&response_headers, &self.forward_headers);

        let remote_host = url.url.host_str().unwrap_or_default();
        for method in &methods_for_metrics {
            self.metrics
                .add_proxy_upstream_response(request.chain.as_ref(), method, remote_host, status, request.elapsed().as_millis());
        }

        self.broadcast_webhook.notify_broadcast(request, status, &body, &self.broadcast_providers);

        info_with_fields!(
            "Proxy response",
            id = request.id.as_str(),
            chain = request.chain.as_ref(),
            remote_host = remote_host,
            method = request.method.as_str(),
            uri = request.path.as_str(),
            status = status,
            latency = DurationMs(request.elapsed()),
        );

        if status == StatusCode::OK.as_u16()
            && !body.is_empty()
            && let (Some(ttl), Some(key)) = (cache_ttl, cache_key)
        {
            let cache = self.cache.clone();
            let content_type = response_headers.get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).unwrap_or(JSON_CONTENT_TYPE);
            let cached = ProxyResponse::with_content_type(status, body.clone(), content_type);
            let size = cached.body.len();
            let id = request.id.clone();
            let host = request.host.clone();
            let method = request.method.to_string();
            let path = request.path.clone();
            let elapsed = request.elapsed();
            tokio::spawn(async move {
                cache.set(&chain, key, cached, ttl).await;
                info_with_fields!(
                    "Cache SET",
                    id = id.as_str(),
                    chain = chain.as_ref(),
                    host = &host,
                    method = method.as_str(),
                    path = &path,
                    ttl_ms = ttl.as_millis(),
                    size_bytes = size,
                    latency = DurationMs(elapsed),
                );
            });
        }

        Ok(ProxyResponse::new(status, headers, body).with_proxy_headers(request.id.as_str(), response_latency, CacheStatus::Miss))
    }

    async fn try_cache_hit(&self, cache_key: &str, request: &ProxyRequest, methods_for_metrics: &[String]) -> Option<ProxyResponse> {
        if let Some(cached) = self.cache.get(&request.chain, cache_key).await {
            for method_name in methods_for_metrics {
                self.metrics.add_cache_hit(request.chain.as_ref(), method_name);
            }

            info_with_fields!(
                "Cache HIT",
                id = request.id.as_str(),
                chain = request.chain.as_ref(),
                host = &request.host,
                method = &methods_for_metrics.join(",")
            );

            Some(cached.with_proxy_headers(request.id.as_str(), request.elapsed(), CacheStatus::Hit))
        } else {
            for method_name in methods_for_metrics {
                self.metrics.add_cache_miss(request.chain.as_ref(), method_name);
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use primitives::Chain;
    use reqwest::header;
    use settings_chain::BroadcastProviders;

    use super::*;
    use crate::cache::RequestCache;
    use crate::config::HeadersConfig;
    use crate::metrics::Metrics;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use crate::testkit::config::metrics_config;

    fn create_service(headers_config: HeadersConfig) -> ProxyRequestService {
        let metrics = Metrics::new(metrics_config());
        ProxyRequestService::new(
            metrics.clone(),
            RequestCache::default(),
            gem_client::reqwest_client(),
            headers_config,
            DynodeBroadcastWebhookClient::disabled(),
            Arc::new(BroadcastProviders::from_chains([Chain::Ethereum])),
        )
    }

    #[test]
    fn test_build_headers_forwards_configured_headers() {
        let service = create_service(HeadersConfig {
            forward: vec![header::CONTENT_TYPE.to_string(), header::USER_AGENT.to_string()],
        });

        let mut original = HeaderMap::new();
        original.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(JSON_CONTENT_TYPE));
        original.insert(header::USER_AGENT, header::HeaderValue::from_static("TestAgent/1.0"));
        original.insert("x-drop", header::HeaderValue::from_static("dropped"));

        let headers = service.build_headers(&original);

        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), JSON_CONTENT_TYPE);
        assert_eq!(headers.get(header::USER_AGENT).unwrap(), "TestAgent/1.0");
        assert!(headers.get("x-drop").is_none());
    }

    #[test]
    fn test_build_headers_drops_unconfigured_headers() {
        let service = create_service(HeadersConfig {
            forward: vec![header::CONTENT_TYPE.to_string()],
        });

        let mut original = HeaderMap::new();
        original.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(JSON_CONTENT_TYPE));
        original.insert(header::USER_AGENT, header::HeaderValue::from_static("TestAgent/1.0"));

        let headers = service.build_headers(&original);

        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), JSON_CONTENT_TYPE);
        assert!(headers.get(header::USER_AGENT).is_none());
    }

    #[test]
    fn test_build_headers_forces_grpc_identity_encoding() {
        let service = create_service(HeadersConfig {
            forward: vec![header::CONTENT_TYPE.to_string(), GRPC_ACCEPT_ENCODING.to_string()],
        });

        let mut original = HeaderMap::new();
        original.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/grpc+proto"));
        original.insert(GRPC_ACCEPT_ENCODING, header::HeaderValue::from_static("gzip"));

        let headers = service.build_headers(&original);

        assert_eq!(headers.get(GRPC_ACCEPT_ENCODING).unwrap(), "identity");
    }
}
