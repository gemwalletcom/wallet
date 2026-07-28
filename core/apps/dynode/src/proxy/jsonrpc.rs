use crate::cache::{CacheProvider, RequestCache};
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcRequest, JsonRpcResponse, JsonRpcResult};
use crate::metrics::Metrics;
use crate::proxy::CachedResponse;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::request_builder::RequestBuilder;
use crate::proxy::request_url::RequestUrl;
use crate::proxy::response_builder::ResponseBuilder;
use crate::webhook::DynodeBroadcastWebhookClient;
use gem_tracing::{DurationMs, info_with_fields};
use reqwest::header::HeaderMap;
use reqwest::{Method, StatusCode};
use settings_chain::BroadcastProviders;

use crate::proxy::ProxyResponse;

pub struct JsonRpcHandler;

impl JsonRpcHandler {
    pub async fn handle_request(
        rpc_request: &JsonRpcRequest,
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &reqwest::Client,
        forward_headers: &HeaderMap,
        broadcast_webhook: &DynodeBroadcastWebhookClient,
        broadcast_providers: &BroadcastProviders,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        match rpc_request {
            JsonRpcRequest::Single(call) => Self::handle_single_request(call, request, cache, metrics, url, client, forward_headers, broadcast_webhook, broadcast_providers).await,
            JsonRpcRequest::Batch(calls) => Self::handle_batch_request(rpc_request, calls, request, cache, metrics, url, client, forward_headers).await,
        }
    }

    async fn handle_single_request(
        call: &JsonRpcCall,
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &reqwest::Client,
        forward_headers: &HeaderMap,
        broadcast_webhook: &DynodeBroadcastWebhookClient,
        broadcast_providers: &BroadcastProviders,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = call.cache_key(&request.host, &request.path_with_query);
        let cacheable = cache.should_cache_call(&request.chain, call).is_some();
        if cacheable
            && let Some(cached) = cache.get(&request.chain, &cache_key).await
            && let Some(response) = Self::cached_result(call, &cached)
        {
            metrics.add_cache_hit(request.chain.as_ref(), &call.method);
            let request_id = request.id.as_str();
            info_with_fields!(
                "Cache HIT",
                id = request_id,
                chain = request.chain.as_ref(),
                host = request.host.as_str(),
                method = call.method.as_str()
            );

            let proxy_headers = ResponseBuilder::create_proxy_headers(request.id.as_str(), request.elapsed());
            return Self::build_json_response(&response, proxy_headers, StatusCode::OK.as_u16()).map(ProxyResponse::into_cached);
        }
        if cacheable {
            metrics.add_cache_miss(request.chain.as_ref(), &call.method);
        }

        let (response, response_status, response_body) = Self::fetch_single_response(call, request, cache, url, client, forward_headers).await?;

        metrics.add_proxy_upstream_response(
            request.chain.as_ref(),
            &call.method,
            url.url.host_str().unwrap_or_default(),
            response_status,
            request.elapsed().as_millis(),
        );

        let request_id = request.id.as_str();
        match &response {
            JsonRpcResult::Success(_) => {
                info_with_fields!(
                    "Proxy response",
                    id = request_id,
                    chain = request.chain.as_ref(),
                    remote_host = url.url.host_str().unwrap_or_default(),
                    method = request.method.as_str(),
                    uri = request.path.as_str(),
                    rpc_method = call.method.as_str(),
                    status = response_status,
                    latency = DurationMs(request.elapsed()),
                );
            }
            JsonRpcResult::Error(error_response) => {
                info_with_fields!(
                    "Proxy response",
                    id = request_id,
                    chain = request.chain.as_ref(),
                    remote_host = url.url.host_str().unwrap_or_default(),
                    method = request.method.as_str(),
                    uri = request.path.as_str(),
                    rpc_method = call.method.as_str(),
                    status = response_status,
                    latency = DurationMs(request.elapsed()),
                    error_code = error_response.error.code,
                    error = error_response.error.message.as_str(),
                );
            }
        }

        broadcast_webhook.notify_broadcast(request, response_status, &response_body, broadcast_providers);

        let proxy_headers = ResponseBuilder::create_proxy_headers(request.id.as_str(), request.elapsed());
        Self::build_json_response(&response, proxy_headers, response_status)
    }

    async fn handle_batch_request(
        rpc_request: &JsonRpcRequest,
        calls: &[JsonRpcCall],
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &reqwest::Client,
        forward_headers: &HeaderMap,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut cached_results = vec![None; calls.len()];
        let mut misses = Vec::new();

        for (index, call) in calls.iter().enumerate() {
            let cacheable = cache.should_cache_call(&request.chain, call).is_some();
            let cached = if cacheable {
                cache
                    .get(&request.chain, &call.cache_key(&request.host, &request.path_with_query))
                    .await
                    .and_then(|response| Self::cached_result(call, &response))
            } else {
                None
            };

            if let Some(result) = cached {
                metrics.add_cache_hit(request.chain.as_ref(), &call.method);
                cached_results[index] = Some(result);
            } else {
                if cacheable {
                    metrics.add_cache_miss(request.chain.as_ref(), &call.method);
                }
                misses.push(call.clone());
            }
        }

        let all_cached = misses.is_empty();
        let (responses, response_status) = if all_cached {
            (serde_json::to_value(cached_results.into_iter().flatten().collect::<Vec<_>>())?, StatusCode::OK.as_u16())
        } else {
            let (upstream, status) = Self::fetch_batch_responses(&misses, url, client, &request.method, forward_headers).await?;
            match serde_json::from_value::<Vec<JsonRpcResult>>(upstream.clone()) {
                Ok(live_results) => {
                    if status == StatusCode::OK.as_u16() {
                        Self::store_batch_results(&misses, &live_results, request, cache).await?;
                    }
                    (serde_json::to_value(Self::merge_batch_results(calls, cached_results, live_results))?, status)
                }
                Err(_) => (upstream, status),
            }
        };

        for call in &misses {
            metrics.add_proxy_upstream_response(
                request.chain.as_ref(),
                &call.method,
                url.url.host_str().unwrap_or_default(),
                response_status,
                request.elapsed().as_millis(),
            );
        }

        let rpc_methods = rpc_request.get_methods_list();
        let request_id = request.id.as_str();
        info_with_fields!(
            "Proxy response",
            id = request_id,
            chain = request.chain.as_ref(),
            remote_host = url.url.host_str().unwrap_or_default(),
            method = request.method.as_str(),
            uri = request.path.as_str(),
            rpc_method = &rpc_methods,
            status = response_status,
            latency = DurationMs(request.elapsed()),
        );

        let proxy_headers = ResponseBuilder::create_proxy_headers(request.id.as_str(), request.elapsed());
        let response = Self::build_json_response(&responses, proxy_headers, response_status)?;
        Ok(if all_cached { response.into_cached() } else { response })
    }

    async fn send_jsonrpc_request(
        client: &reqwest::Client,
        method: &Method,
        url: &RequestUrl,
        body: Vec<u8>,
        headers: &HeaderMap,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let request = RequestBuilder::build(method, url, body, headers.clone())?;
        Ok(client.execute(request).await?)
    }

    async fn fetch_single_response(
        call: &JsonRpcCall,
        request: &ProxyRequest,
        cache: &RequestCache,
        url: &RequestUrl,
        client: &reqwest::Client,
        forward_headers: &HeaderMap,
    ) -> Result<(JsonRpcResult, u16, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_vec(call)?;
        let response = Self::send_jsonrpc_request(client, &request.method, url, body, forward_headers).await?;
        let status = response.status().as_u16();
        let body_bytes = response.bytes().await?.to_vec();

        let result: JsonRpcResult = serde_json::from_slice(&body_bytes).map_err(|e| Self::format_parse_error(status, &body_bytes, e))?;

        if status == StatusCode::OK.as_u16()
            && let (JsonRpcResult::Success(success), Some(ttl)) = (&result, cache.should_cache_call(&request.chain, call))
        {
            let result_bytes = serde_json::to_vec(&success.result)?;
            let size_bytes = result_bytes.len();
            let cached = CachedResponse::new(result_bytes, StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string());
            let cache_key = call.cache_key(&request.host, &request.path_with_query);
            cache.set(&request.chain, cache_key, cached, ttl).await;

            info_with_fields!(
                "Cache SET",
                id = request.id.as_str(),
                chain = request.chain.as_ref(),
                host = request.host.as_str(),
                method = call.method.as_str(),
                ttl_ms = ttl.as_millis(),
                size_bytes = size_bytes,
                latency = DurationMs(request.elapsed()),
            );
        }

        Ok((result, status, body_bytes))
    }

    async fn fetch_batch_responses(
        calls: &[JsonRpcCall],
        url: &RequestUrl,
        client: &reqwest::Client,
        method: &Method,
        forward_headers: &HeaderMap,
    ) -> Result<(serde_json::Value, u16), Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_vec(&calls)?;
        let response = Self::send_jsonrpc_request(client, method, url, body, forward_headers).await?;
        let status = response.status().as_u16();
        let body_bytes = response.bytes().await?.to_vec();
        let responses: serde_json::Value = serde_json::from_slice(&body_bytes).map_err(|e| Self::format_parse_error(status, &body_bytes, e))?;
        Ok((responses, status))
    }

    async fn store_batch_results(
        calls: &[JsonRpcCall],
        results: &[JsonRpcResult],
        request: &ProxyRequest,
        cache: &RequestCache,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for call in calls {
            if calls.iter().filter(|candidate| candidate.id == call.id).count() != 1 {
                continue;
            }
            let Some(ttl) = cache.should_cache_call(&request.chain, call) else {
                continue;
            };
            let Some(JsonRpcResult::Success(success)) = results.iter().find(|result| result.id() == Some(call.id)) else {
                continue;
            };

            let result_bytes = serde_json::to_vec(&success.result)?;
            let size_bytes = result_bytes.len();
            let cached = CachedResponse::new(result_bytes, StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string());
            cache.set(&request.chain, call.cache_key(&request.host, &request.path_with_query), cached, ttl).await;

            info_with_fields!(
                "Cache SET",
                id = request.id.as_str(),
                chain = request.chain.as_ref(),
                host = request.host.as_str(),
                method = call.method.as_str(),
                ttl_ms = ttl.as_millis(),
                size_bytes = size_bytes,
                latency = DurationMs(request.elapsed()),
            );
        }
        Ok(())
    }

    fn merge_batch_results(calls: &[JsonRpcCall], cached_results: Vec<Option<JsonRpcResult>>, live_results: Vec<JsonRpcResult>) -> Vec<JsonRpcResult> {
        let mut used = vec![false; live_results.len()];
        let mut merged = Vec::with_capacity(calls.len().max(live_results.len()));

        for (call, cached) in calls.iter().zip(cached_results) {
            if let Some(result) = cached {
                merged.push(result);
                continue;
            }
            if let Some((index, result)) = live_results.iter().enumerate().find(|(index, result)| !used[*index] && result.id() == Some(call.id)) {
                used[index] = true;
                merged.push(result.clone());
            }
        }

        merged.extend(live_results.into_iter().enumerate().filter_map(|(index, result)| (!used[index]).then_some(result)));
        merged
    }

    fn cached_result(call: &JsonRpcCall, cached: &CachedResponse) -> Option<JsonRpcResult> {
        Some(JsonRpcResult::Success(JsonRpcResponse {
            jsonrpc: call.jsonrpc.clone(),
            result: serde_json::from_slice(&cached.body).ok()?,
            id: Some(call.id),
        }))
    }

    fn build_json_response<T: serde::Serialize>(data: &T, headers: HeaderMap, status: u16) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let response_body = serde_json::to_vec(data)?;
        ResponseBuilder::build_with_headers(response_body, status, JSON_CONTENT_TYPE, headers)
    }

    fn format_parse_error(status: u16, body: &[u8], error: serde_json::Error) -> String {
        const MAX_BODY_LEN: usize = 256;
        if body.len() <= MAX_BODY_LEN
            && let Ok(text) = std::str::from_utf8(body)
        {
            let body = text.split_whitespace().collect::<Vec<_>>().join(" ");
            return format!("status={}, body: {}", status, body);
        }
        format!("status={}, parse error: {}", status, error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonrpc_types::{JsonRpcError, JsonRpcErrorResponse};
    use serde_json::json;
    fn make_call(id: u64, method: &str) -> JsonRpcCall {
        JsonRpcCall {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params: json!([]),
            id,
        }
    }

    fn success(id: u64, value: serde_json::Value) -> JsonRpcResult {
        JsonRpcResult::Success(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: value,
            id: Some(id),
        })
    }

    fn error(id: Option<u64>, code: i32) -> JsonRpcResult {
        JsonRpcResult::Error(JsonRpcErrorResponse {
            jsonrpc: "2.0".to_string(),
            error: JsonRpcError {
                code,
                message: "upstream error".to_string(),
            },
            id,
        })
    }

    #[test]
    fn test_format_parse_error() {
        let err = || serde_json::from_slice::<serde_json::Value>(b"x").unwrap_err();

        assert_eq!(
            JsonRpcHandler::format_parse_error(415, b"Expected Content-Type: application/json", err()),
            "status=415, body: Expected Content-Type: application/json"
        );
        assert_eq!(
            JsonRpcHandler::format_parse_error(400, b"<html>\n<body>Bad Request</body>\n</html>", err()),
            "status=400, body: <html> <body>Bad Request</body> </html>"
        );
        assert_eq!(
            JsonRpcHandler::format_parse_error(502, b"<html>Bad Gateway...</html>".repeat(20).as_slice(), err()),
            "status=502, parse error: expected value at line 1 column 1"
        );
        assert_eq!(
            JsonRpcHandler::format_parse_error(500, &[0xff, 0xfe], err()),
            "status=500, parse error: expected value at line 1 column 1"
        );
    }

    #[test]
    fn test_merge_batch_results_preserves_request_order_ids_and_errors() {
        let calls = vec![make_call(7, "cached"), make_call(3, "failed"), make_call(9, "live")];
        let cached_results = vec![Some(success(7, json!("cached"))), None, None];
        let live_results = vec![success(9, json!("live")), error(Some(3), -32000), error(None, -32600)];

        let merged = JsonRpcHandler::merge_batch_results(&calls, cached_results, live_results);

        assert_eq!(
            serde_json::to_value(merged).unwrap(),
            serde_json::to_value(vec![success(7, json!("cached")), error(Some(3), -32000), success(9, json!("live")), error(None, -32600),]).unwrap()
        );
    }

    #[test]
    fn test_cached_result_rewrites_client_id_and_rejects_invalid_json() {
        let call = make_call(42, "eth_chainId");
        let cached = CachedResponse::new(br#""0x1""#.to_vec(), 200, JSON_CONTENT_TYPE.to_string());
        let invalid = CachedResponse::new(b"invalid".to_vec(), 200, JSON_CONTENT_TYPE.to_string());

        assert_eq!(
            serde_json::to_value(JsonRpcHandler::cached_result(&call, &cached).unwrap()).unwrap(),
            serde_json::to_value(success(42, json!("0x1"))).unwrap()
        );
        assert!(JsonRpcHandler::cached_result(&call, &invalid).is_none());
    }
}
