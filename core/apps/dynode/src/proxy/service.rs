use crate::cache::{CacheProvider, RequestCache};
use crate::config::{ChainConfig, HeadersConfig, Url};
use crate::jsonrpc_types::{JsonRpcRequest, RequestType};
use crate::metrics::Metrics;
use crate::proxy::CachedResponse;
use crate::proxy::jsonrpc::JsonRpcHandler;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::request_builder::RequestBuilder;
use crate::proxy::request_url::RequestUrl;
use crate::proxy::response_builder::{ProxyResponse, ResponseBuilder};
use crate::webhook::DynodeBroadcastWebhookClient;
use gem_tracing::{DurationMs, info_with_fields};
use primitives::Chain;
use reqwest::Method;
use reqwest::StatusCode;
use reqwest::header::{HeaderMap, HeaderName};
use settings_chain::BroadcastProviders;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

struct CacheStoreInfo {
    id: String,
    chain: Chain,
    host: String,
    method: String,
    path: String,
    elapsed: Duration,
    content_type: String,
}

#[derive(Clone)]
pub struct ProxyRequestService {
    pub metrics: Metrics,
    pub cache: RequestCache,
    pub client: reqwest::Client,
    pub forward_headers: Arc<HashSet<HeaderName>>,
    pub headers_config: HeadersConfig,
    pub broadcast_webhook: DynodeBroadcastWebhookClient,
    pub broadcast_providers: Arc<BroadcastProviders>,
}

#[derive(Debug, Clone)]
pub struct NodeDomain {
    pub url: Url,
    pub config: ChainConfig,
}

impl NodeDomain {
    pub fn new(url: Url, config: ChainConfig) -> Self {
        Self { url, config }
    }
}

impl ProxyRequestService {
    pub fn new(
        metrics: Metrics,
        cache: RequestCache,
        client: reqwest::Client,
        headers_config: HeadersConfig,
        broadcast_webhook: DynodeBroadcastWebhookClient,
        broadcast_providers: Arc<BroadcastProviders>,
    ) -> Self {
        let forward_headers: Arc<HashSet<HeaderName>> = Arc::new(headers_config.forward.iter().filter_map(|s| HeaderName::from_str(s).ok()).collect());

        Self {
            metrics,
            cache,
            client,
            forward_headers,
            headers_config,
            broadcast_webhook,
            broadcast_providers,
        }
    }

    fn build_headers(&self, host: &str, original: &HeaderMap) -> HeaderMap {
        let mut headers = RequestBuilder::filter_headers(original, &self.forward_headers);

        if let Some(names) = self.headers_config.get_domain_headers(host) {
            for name in names {
                if let Ok(key) = HeaderName::from_str(name)
                    && let Some(value) = original.get(&key)
                {
                    headers.insert(key, value.clone());
                }
            }
        }

        headers
    }

    fn add_proxy_response_metrics(metrics: &Metrics, request: &ProxyRequest, methods_for_metrics: &[String], host: &str, status: u16) {
        for method_name in methods_for_metrics {
            metrics.add_proxy_upstream_response(request.chain.as_ref(), method_name, host, status, request.elapsed().as_millis());
        }
    }

    pub async fn handle_request(&self, request: ProxyRequest, node_domain: &NodeDomain) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let chain = request.chain;
        let request_type = request.request_type();

        let rpc_method = match request_type {
            RequestType::JsonRpc(JsonRpcRequest::Single(call)) => Some(call.method.as_str()),
            _ => None,
        };

        let resolved_url = node_domain.config.resolve_url(&node_domain.url, rpc_method, Some(&request.path));
        let url = RequestUrl::from_parts(resolved_url, &request.path_with_query);
        let headers = self.build_headers(url.url.host_str().unwrap_or_default(), &request.headers);

        let methods_for_metrics = request_type.get_methods_for_metrics();
        self.metrics.add_proxy_request(request.chain.as_ref(), &methods_for_metrics);

        if let RequestType::JsonRpc(rpc_request) = request_type {
            return JsonRpcHandler::handle_request(
                rpc_request,
                &request,
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
            && let Some(result) = Self::try_cache_hit(&self.cache, key, &request, &self.metrics, &methods_for_metrics).await
        {
            return result;
        }

        let response = match Self::proxy_pass_get_data(request.method.clone(), request.body.clone(), url.clone(), &self.client, headers).await {
            Ok(response) => response,
            Err(error) => return Err(error),
        };
        let status = response.status().as_u16();
        let proxy_headers = ResponseBuilder::create_proxy_headers(request.id.as_str(), request.elapsed());
        let (processed_response, body_bytes) = match Self::proxy_pass_response(response, &self.forward_headers, proxy_headers).await {
            Ok(result) => result,
            Err(error) => return Err(error),
        };

        let remote_host = url.url.host_str().unwrap_or_default();
        Self::add_proxy_response_metrics(&self.metrics, &request, &methods_for_metrics, remote_host, status);

        self.broadcast_webhook.notify_broadcast(&request, status, &body_bytes, &self.broadcast_providers);

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
            && let (Some(ttl), Some(key)) = (cache_ttl, cache_key)
        {
            let store_info = CacheStoreInfo {
                id: request.id.clone(),
                chain: request.chain,
                host: request.host.clone(),
                method: request.method.to_string(),
                path: request.path.clone(),
                elapsed: request.elapsed(),
                content_type: request_type.content_type().to_string(),
            };
            let cache_clone = self.cache.clone();
            tokio::spawn(async move {
                Self::store_cache(status, ttl, key, body_bytes, store_info, cache_clone).await;
            });
        }

        Ok(processed_response)
    }

    async fn try_cache_hit(
        cache: &RequestCache,
        cache_key: &str,
        request: &ProxyRequest,
        metrics: &Metrics,
        methods_for_metrics: &[String],
    ) -> Option<Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>>> {
        if let Some(cached) = cache.get(&request.chain, cache_key).await {
            for method_name in methods_for_metrics {
                metrics.add_cache_hit(request.chain.as_ref(), method_name);
            }

            info_with_fields!(
                "Cache HIT",
                id = request.id.as_str(),
                chain = request.chain.as_ref(),
                host = &request.host,
                method = &methods_for_metrics.join(",")
            );

            let proxy_headers = ResponseBuilder::create_proxy_headers(request.id.as_str(), request.elapsed());
            Some(Ok(ResponseBuilder::build_cached_with_headers(cached, proxy_headers)))
        } else {
            for method_name in methods_for_metrics {
                metrics.add_cache_miss(request.chain.as_ref(), method_name);
            }
            None
        }
    }

    async fn store_cache(status: u16, cache_ttl: Duration, cache_key: String, body_bytes: Vec<u8>, info: CacheStoreInfo, cache: RequestCache) {
        let CacheStoreInfo {
            id,
            chain,
            host,
            method,
            path,
            elapsed,
            content_type,
        } = info;
        let body_size = body_bytes.len();
        let cached = CachedResponse::new(body_bytes, status, content_type);

        cache.set(&chain, cache_key, cached, cache_ttl).await;

        info_with_fields!(
            "Cache SET",
            id = id.as_str(),
            chain = chain.as_ref(),
            host = &host,
            method = method.as_str(),
            path = &path,
            ttl_ms = cache_ttl.as_millis(),
            size_bytes = body_size,
            latency = DurationMs(elapsed),
        );
    }

    async fn proxy_pass_response(
        response: reqwest::Response,
        forward_headers: &HashSet<HeaderName>,
        additional_headers: HeaderMap,
    ) -> Result<(ProxyResponse, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let resp_headers = response.headers().clone();
        let status = response.status().as_u16();
        let body = response.bytes().await?.to_vec();

        let mut headers = RequestBuilder::filter_headers(&resp_headers, forward_headers);
        headers.extend(additional_headers);

        Ok((ProxyResponse::new(status, headers, body.clone()), body))
    }

    async fn proxy_pass_get_data(
        method: Method,
        body: Vec<u8>,
        url: RequestUrl,
        client: &reqwest::Client,
        headers: HeaderMap,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let request = RequestBuilder::build(&method, &url, body, headers)?;
        Ok(client.execute(request).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::RequestCache;
    use crate::config::{CacheConfig, HeadersConfig, MetricsConfig};
    use crate::metrics::Metrics;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use primitives::Chain;
    use reqwest::header;
    use settings_chain::BroadcastProviders;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_service(headers_config: HeadersConfig) -> ProxyRequestService {
        let metrics = Metrics::new(MetricsConfig::default());
        ProxyRequestService::new(
            metrics.clone(),
            RequestCache::new(CacheConfig::default(), std::iter::empty()),
            gem_client::reqwest_client(),
            headers_config,
            DynodeBroadcastWebhookClient::disabled(),
            Arc::new(BroadcastProviders::from_chains([Chain::Ethereum])),
        )
    }

    #[test]
    fn test_build_headers_with_domain_config() {
        let mut domains = HashMap::new();
        domains.insert("example.com".to_string(), vec![header::USER_AGENT.to_string()]);

        let service = create_service(HeadersConfig {
            forward: vec![header::CONTENT_TYPE.to_string()],
            domains,
        });

        let mut original = HeaderMap::new();
        original.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(JSON_CONTENT_TYPE));
        original.insert(header::USER_AGENT, header::HeaderValue::from_static("TestAgent/1.0"));
        original.insert("x-drop", header::HeaderValue::from_static("dropped"));

        let headers = service.build_headers("example.com", &original);

        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), JSON_CONTENT_TYPE);
        assert_eq!(headers.get(header::USER_AGENT).unwrap(), "TestAgent/1.0");
        assert!(headers.get("x-drop").is_none());
    }

    #[test]
    fn test_build_headers_without_domain_config() {
        let service = create_service(HeadersConfig {
            forward: vec![header::CONTENT_TYPE.to_string()],
            domains: HashMap::new(),
        });

        let mut original = HeaderMap::new();
        original.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(JSON_CONTENT_TYPE));
        original.insert(header::USER_AGENT, header::HeaderValue::from_static("TestAgent/1.0"));

        let headers = service.build_headers("example.com", &original);

        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), JSON_CONTENT_TYPE);
        assert!(headers.get(header::USER_AGENT).is_none());
    }
}
