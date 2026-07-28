use crate::cache::{CacheProvider, RequestCache};
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcRequest, JsonRpcResult};
use crate::metrics::Metrics;
use crate::proxy::ProxyResponse;
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
use std::collections::{HashMap, HashSet};
use std::time::Duration;

mod cache;

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
            JsonRpcRequest::Batch(calls) => Self::handle_batch_request(calls, request, cache, metrics, url, client, forward_headers).await,
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
        let cache_ttl = cache.should_cache_call(&request.chain, call);
        if cache_ttl.is_some()
            && let Some(response) = cache::get(call, request, cache).await
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
            let body = serde_json::to_vec(&response)?;
            return ResponseBuilder::build_with_headers(body, StatusCode::OK.as_u16(), JSON_CONTENT_TYPE, proxy_headers).map(ProxyResponse::into_cached);
        }
        if cache_ttl.is_some() {
            metrics.add_cache_miss(request.chain.as_ref(), &call.method);
        }

        let (response, response_status, response_body) = Self::fetch_single_response(call, cache_ttl, request, cache, url, client, forward_headers).await?;

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
        ResponseBuilder::build_with_headers(response_body, response_status, JSON_CONTENT_TYPE, proxy_headers)
    }

    async fn handle_batch_request(
        calls: &[JsonRpcCall],
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &reqwest::Client,
        forward_headers: &HeaderMap,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let cache_ttls = calls.iter().map(|call| cache.should_cache_call(&request.chain, call)).collect::<Option<Vec<_>>>();
        let cached_results = if cache_ttls.is_some() { cache::get_all(calls, request, cache).await } else { None };
        if cache_ttls.is_some() {
            for call in calls {
                if cached_results.is_some() {
                    metrics.add_cache_hit(request.chain.as_ref(), &call.method);
                } else {
                    metrics.add_cache_miss(request.chain.as_ref(), &call.method);
                }
            }
        }

        let (response_body, response_status, from_cache) = if let Some(results) = cached_results {
            (serde_json::to_vec(&results)?, StatusCode::OK.as_u16(), true)
        } else {
            let (body, status) = Self::fetch(calls, &request.method, url, client, forward_headers).await?;
            let response = serde_json::from_slice::<serde_json::Value>(&body).map_err(|error| Self::format_parse_error(status, &body, error))?;
            let body = if response.is_array() {
                serde_json::to_vec(&Self::order_batch(calls, response)?)?
            } else {
                body
            };
            let results = serde_json::from_slice::<Vec<JsonRpcResult>>(&body);
            if status == StatusCode::OK.as_u16()
                && let (Some(ttls), Ok(results)) = (cache_ttls.as_deref(), results.as_ref())
            {
                cache::set_all(calls, ttls, results, request, cache).await?;
            }

            for call in calls {
                metrics.add_proxy_upstream_response(
                    request.chain.as_ref(),
                    &call.method,
                    url.url.host_str().unwrap_or_default(),
                    status,
                    request.elapsed().as_millis(),
                );
            }
            (body, status, false)
        };

        let rpc_methods = request.request_type().get_methods_list();
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
        let response = ResponseBuilder::build_with_headers(response_body, response_status, JSON_CONTENT_TYPE, proxy_headers)?;
        Ok(if from_cache { response.into_cached() } else { response })
    }

    async fn fetch<T: serde::Serialize + ?Sized>(
        data: &T,
        method: &Method,
        url: &RequestUrl,
        client: &reqwest::Client,
        headers: &HeaderMap,
    ) -> Result<(Vec<u8>, u16), Box<dyn std::error::Error + Send + Sync>> {
        let body = serde_json::to_vec(data)?;
        let request = RequestBuilder::build(method, url, body, headers.clone())?;
        let response = client.execute(request).await?;
        let status = response.status().as_u16();
        Ok((response.bytes().await?.to_vec(), status))
    }

    async fn fetch_single_response(
        call: &JsonRpcCall,
        cache_ttl: Option<Duration>,
        request: &ProxyRequest,
        cache: &RequestCache,
        url: &RequestUrl,
        client: &reqwest::Client,
        forward_headers: &HeaderMap,
    ) -> Result<(JsonRpcResult, u16, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
        let (body, status) = Self::fetch(call, &request.method, url, client, forward_headers).await?;

        let result: JsonRpcResult = serde_json::from_slice(&body).map_err(|error| Self::format_parse_error(status, &body, error))?;

        if status == StatusCode::OK.as_u16()
            && let (JsonRpcResult::Success(success), Some(ttl)) = (&result, cache_ttl)
        {
            cache::set(call, success, ttl, request, cache).await?;
        }

        Ok((result, status, body))
    }

    fn order_batch(calls: &[JsonRpcCall], response: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let serde_json::Value::Array(results) = response else {
            return Ok(response);
        };

        let request_ids = calls.iter().map(|call| call.id).collect::<HashSet<_>>();
        if request_ids.len() != calls.len() {
            return Ok(serde_json::Value::Array(results));
        }
        if results.len() != calls.len() {
            return Err("invalid JSON-RPC batch response length".into());
        }

        let results_by_id = results
            .into_iter()
            .filter_map(|result| result.get("id").and_then(serde_json::Value::as_u64).map(|id| (id, result)))
            .collect::<HashMap<_, _>>();
        if results_by_id.len() != calls.len() || !results_by_id.keys().all(|id| request_ids.contains(id)) {
            return Err("invalid JSON-RPC batch response IDs".into());
        }

        let ordered = calls.iter().filter_map(|call| results_by_id.get(&call.id).cloned()).collect();
        Ok(serde_json::Value::Array(ordered))
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
    use serde_json::json;

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
    fn test_order_batch_matches_response_ids() {
        let calls = vec![JsonRpcCall::mock(7, "cached"), JsonRpcCall::mock(3, "failed")];
        let response = json!([
            {
                "jsonrpc": "2.0",
                "error": {
                    "code": -32000,
                    "message": "upstream error",
                    "data": { "retry": true }
                },
                "id": 3
            },
            {
                "jsonrpc": "2.0",
                "result": "cached",
                "id": 7
            }
        ]);

        let ordered = JsonRpcHandler::order_batch(&calls, response).unwrap();
        assert_eq!(
            ordered,
            json!([
                {
                    "jsonrpc": "2.0",
                    "result": "cached",
                    "id": 7
                },
                {
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32000,
                        "message": "upstream error",
                        "data": { "retry": true }
                    },
                    "id": 3
                }
            ])
        );

        let duplicate = json!([
            { "jsonrpc": "2.0", "result": "first", "id": 7 },
            { "jsonrpc": "2.0", "result": "second", "id": 7 }
        ]);
        assert!(JsonRpcHandler::order_batch(&calls, duplicate).is_err());
    }
}
