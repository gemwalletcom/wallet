use bytes::Bytes;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use chain_providers::BroadcastProviders;
use gem_tracing::{DurationMs, info_with_fields};
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use self::error::ResponseError;
use crate::BoxError;
use crate::cache::RequestCache;
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcRequest, JsonRpcResult};
use crate::metrics::Metrics;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_request::ProxyRequest;
use crate::proxy::request_url::RequestUrl;
use crate::proxy::transport;
use crate::proxy::{CacheStatus, ProxyResponse};
use crate::webhook::DynodeBroadcastWebhookClient;

mod cache;
pub(crate) mod error;

pub struct JsonRpcHandler;

impl JsonRpcHandler {
    pub async fn forward_request(
        rpc_request: &JsonRpcRequest,
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &Client,
        forward_headers: &HeaderMap,
        broadcast_webhook: &DynodeBroadcastWebhookClient,
        broadcast_providers: &BroadcastProviders,
    ) -> Result<ProxyResponse, BoxError> {
        match rpc_request {
            JsonRpcRequest::Single(call) => Self::forward_call(call, request, cache, metrics, url, client, forward_headers, broadcast_webhook, broadcast_providers).await,
            JsonRpcRequest::Batch(calls) => Self::forward_batch(calls, request, cache, metrics, url, client, forward_headers).await,
        }
    }

    async fn forward_call(
        call: &JsonRpcCall,
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &Client,
        forward_headers: &HeaderMap,
        broadcast_webhook: &DynodeBroadcastWebhookClient,
        broadcast_providers: &BroadcastProviders,
    ) -> Result<ProxyResponse, BoxError> {
        let cache_ttl = cache.should_cache_call(&request.chain, call);
        if cache_ttl.is_some()
            && let Some(response) = cache::get(call, request, cache)
        {
            metrics.add_cache_hit(request.chain.as_ref(), &call.method);
            let request_id = request.id.as_str();
            info_with_fields!("Cache HIT", id = request_id, chain = request.chain.as_ref(), host = request.host.as_str(), method = call.method.as_str());

            let response_latency = request.elapsed();
            let body = Bytes::from(serde_json::to_vec(&response)?);
            return Ok(ProxyResponse::with_content_type(StatusCode::OK.as_u16(), body, JSON_CONTENT_TYPE).with_proxy_headers(request.id.as_str(), response_latency, CacheStatus::Hit));
        }
        if cache_ttl.is_some() {
            metrics.add_cache_miss(request.chain.as_ref(), &call.method);
        }

        let (response, response_status, response_body) = Self::send_call_upstream(call, cache_ttl, request, cache, metrics, url, client, forward_headers).await?;

        metrics.add_proxy_upstream_response(request.chain.as_ref(), &call.method, url.url.host_str().unwrap_or_default(), response_status, request.elapsed().as_millis());

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

        Ok(ProxyResponse::with_content_type(response_status, response_body, JSON_CONTENT_TYPE).with_proxy_headers(request.id.as_str(), request.elapsed(), CacheStatus::Miss))
    }

    async fn forward_batch(calls: &[JsonRpcCall], request: &ProxyRequest, cache: &RequestCache, metrics: &Metrics, url: &RequestUrl, client: &Client, forward_headers: &HeaderMap) -> Result<ProxyResponse, BoxError> {
        let request_ids = calls.iter().map(|call| call.id).collect::<HashSet<_>>();
        let cache_ttls = if request_ids.len() == calls.len() {
            calls.iter().map(|call| cache.should_cache_call(&request.chain, call)).collect::<Vec<_>>()
        } else {
            vec![None; calls.len()]
        };
        let cached_results = cache::get_many(calls, &cache_ttls, request, cache);
        for ((call, ttl), result) in calls.iter().zip(&cache_ttls).zip(&cached_results) {
            if ttl.is_some() {
                if result.is_some() {
                    metrics.add_cache_hit(request.chain.as_ref(), &call.method);
                } else {
                    metrics.add_cache_miss(request.chain.as_ref(), &call.method);
                }
            }
        }

        let cache_hits = cached_results.iter().filter(|result| result.is_some()).count();
        let missing_calls = calls.iter().zip(&cached_results).filter(|(_, result)| result.is_none()).map(|(call, _)| call.clone()).collect::<Vec<_>>();
        let missing_ttls = cache_ttls.iter().zip(&cached_results).filter(|(_, result)| result.is_none()).map(|(ttl, _)| *ttl).collect::<Vec<_>>();

        let (response_body, response_status, cache_status) = if missing_calls.is_empty() {
            let results = cached_results.into_iter().flatten().collect::<Vec<_>>();
            (Bytes::from(serde_json::to_vec(&results)?), StatusCode::OK.as_u16(), CacheStatus::Hit)
        } else {
            let (body, missing_status) = Self::send_upstream(&missing_calls, request, metrics, url, client, forward_headers).await?;
            let response = Self::single_call_batch_response(&missing_calls, Self::parse_response(missing_status, &body)?);
            let (body, status, cache_status) = if let Value::Array(results) = response {
                let results = Self::order_batch(&missing_calls, results, missing_status)?;
                if missing_status == StatusCode::OK.as_u16() {
                    cache::set_many(&missing_calls, &missing_ttls, &results, request, cache)?;
                }
                let cache_status = if cache_hits == 0 { CacheStatus::Miss } else { CacheStatus::Partial };
                (Bytes::from(serde_json::to_vec(&Self::merge_batch_results(cached_results, results, missing_status)?)?), missing_status, cache_status)
            } else if cache_hits > 0 {
                let (body, status) = Self::send_upstream(calls, request, metrics, url, client, forward_headers).await?;
                let body = match Self::parse_response(status, &body)? {
                    Value::Array(results) => Bytes::from(serde_json::to_vec(&Self::order_batch(calls, results, status)?)?),
                    _ => body,
                };
                for call in calls {
                    metrics.add_proxy_upstream_response(request.chain.as_ref(), &call.method, url.url.host_str().unwrap_or_default(), status, request.elapsed().as_millis());
                }
                (body, status, CacheStatus::Miss)
            } else {
                (body, missing_status, CacheStatus::Miss)
            };

            for call in &missing_calls {
                metrics.add_proxy_upstream_response(request.chain.as_ref(), &call.method, url.url.host_str().unwrap_or_default(), missing_status, request.elapsed().as_millis());
            }
            (body, status, cache_status)
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

        Ok(ProxyResponse::with_content_type(response_status, response_body, JSON_CONTENT_TYPE).with_proxy_headers(request.id.as_str(), request.elapsed(), cache_status))
    }

    async fn send_upstream<T: Serialize + ?Sized>(data: &T, request: &ProxyRequest, metrics: &Metrics, url: &RequestUrl, client: &Client, headers: &HeaderMap) -> Result<(Bytes, u16), BoxError> {
        let body = Bytes::from(serde_json::to_vec(data)?);
        let upstream_request = url.build_request(&request.method, body, headers.clone());
        let attempt_start = Instant::now();
        let result = transport::send(client, upstream_request).await;
        metrics.record_node_upstream(
            request.chain,
            url.url.host_str().unwrap_or_default(),
            &request.path,
            result.as_ref().map_or(StatusCode::BAD_GATEWAY.as_u16(), |response| response.status),
            attempt_start.elapsed(),
        );
        let response = result?;
        Ok((response.body, response.status))
    }

    async fn send_call_upstream(
        call: &JsonRpcCall,
        cache_ttl: Option<Duration>,
        request: &ProxyRequest,
        cache: &RequestCache,
        metrics: &Metrics,
        url: &RequestUrl,
        client: &Client,
        forward_headers: &HeaderMap,
    ) -> Result<(JsonRpcResult, u16, Bytes), BoxError> {
        let (body, status) = Self::send_upstream(call, request, metrics, url, client, forward_headers).await?;

        let result = Self::parse_response(status, &body)?;

        if status == StatusCode::OK.as_u16()
            && let (JsonRpcResult::Success(success), Some(ttl)) = (&result, cache_ttl)
        {
            cache::set_result(call, &success.result, ttl, request, cache)?;
        }

        Ok((result, status, body))
    }

    fn single_call_batch_response(calls: &[JsonRpcCall], response: Value) -> Value {
        match (calls, response) {
            ([call], Value::Object(object)) if object.get("id").and_then(Value::as_u64) == Some(call.id) => Value::Array(vec![Value::Object(object)]),
            (_, response) => response,
        }
    }

    fn order_batch(calls: &[JsonRpcCall], results: Vec<Value>, status: u16) -> Result<Vec<Value>, ResponseError> {
        let request_ids = calls.iter().map(|call| call.id).collect::<HashSet<_>>();
        if request_ids.len() != calls.len() {
            return Ok(results);
        }
        if results.len() != calls.len() {
            return Err(ResponseError::InvalidBatch { status, detail: "length" });
        }

        let mut results_by_id = results.into_iter().filter_map(|result| result.get("id").and_then(Value::as_u64).map(|id| (id, result))).collect::<HashMap<_, _>>();
        if results_by_id.len() != calls.len() || !results_by_id.keys().all(|id| request_ids.contains(id)) {
            return Err(ResponseError::InvalidBatch { status, detail: "ids" });
        }

        Ok(calls.iter().filter_map(|call| results_by_id.remove(&call.id)).collect())
    }

    fn merge_batch_results(cached_results: Vec<Option<JsonRpcResult>>, upstream_results: Vec<Value>, status: u16) -> Result<Vec<Value>, BoxError> {
        let mut upstream_results = upstream_results.into_iter();
        let results = cached_results
            .into_iter()
            .map(|result| -> Result<Value, BoxError> {
                match result {
                    Some(result) => Ok(serde_json::to_value(result)?),
                    None => upstream_results.next().ok_or_else(|| ResponseError::InvalidBatch { status, detail: "partial_length" }.into()),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        if upstream_results.next().is_some() {
            return Err(ResponseError::InvalidBatch { status, detail: "partial_length" }.into());
        }
        Ok(results)
    }

    fn parse_response<T: DeserializeOwned>(status: u16, body: &[u8]) -> Result<T, ResponseError> {
        serde_json::from_slice(body).map_err(|error| ResponseError::decode(status, error))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::failure_reason::FailureReason;

    #[test]
    fn test_parse_response_preserves_failure_status_and_excludes_body() {
        for (status, expected_reason) in [(500, "status=500"), (429, "status=429"), (200, "response_decode")] {
            let error = JsonRpcHandler::parse_response::<JsonRpcResult>(status, b"<html>https://node.example/secret-key</html>").unwrap_err();
            assert_eq!(FailureReason::from_error(&error).to_string(), expected_reason);
            assert_eq!(error.to_string(), format!("response_decode status={status} category=syntax line=1 column=1"));
        }

        let error = JsonRpcHandler::parse_response::<JsonRpcResult>(200, br#"{"credential":"secret-key"}"#).unwrap_err();
        assert_eq!(FailureReason::from_error(&error).to_string(), "response_decode");
        assert_eq!(error.to_string(), "response_decode status=200 category=data line=0 column=0");

        let error = JsonRpcHandler::parse_response::<Value>(429, b"Too many requests").unwrap_err();
        assert_eq!(FailureReason::from_error(&error).to_string(), "status=429");
        assert_eq!(FailureReason::error_detail(&error), "response_decode status=429 category=syntax line=1 column=1");

        let response: JsonRpcResult = JsonRpcHandler::parse_response(200, br#"{"jsonrpc":"2.0","id":1,"result":"0x12"}"#).unwrap();
        assert_eq!(serde_json::to_value(response).unwrap(), json!({ "jsonrpc": "2.0", "id": 1, "result": "0x12" }));
    }

    #[test]
    fn test_single_call_batch_response() {
        let calls = vec![JsonRpcCall::mock(1, "eth_call")];
        let result = json!({ "jsonrpc": "2.0", "id": 1, "result": "0x12" });

        assert_eq!(JsonRpcHandler::single_call_batch_response(&calls, result.clone()), json!([result]));
        assert_eq!(JsonRpcHandler::single_call_batch_response(&calls, json!([result])), json!([result]));

        let batch_error = json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32600, "message": "invalid request" } });
        assert_eq!(JsonRpcHandler::single_call_batch_response(&calls, batch_error.clone()), batch_error);
    }

    #[test]
    fn test_order_batch_matches_response_ids() {
        let calls = vec![JsonRpcCall::mock(7, "cached"), JsonRpcCall::mock(3, "failed")];
        let failed = json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32000,
                "message": "upstream error",
                "data": { "retry": true }
            },
            "id": 3
        });
        let cached = json!({
            "jsonrpc": "2.0",
            "result": "cached",
            "id": 7
        });

        let ordered = JsonRpcHandler::order_batch(&calls, vec![failed.clone(), cached.clone()], 200).unwrap();
        assert_eq!(ordered, vec![cached, failed]);

        let duplicate = vec![json!({ "jsonrpc": "2.0", "result": "first", "id": 7 }), json!({ "jsonrpc": "2.0", "result": "second", "id": 7 })];
        let error = JsonRpcHandler::order_batch(&calls, duplicate, 200).unwrap_err();
        assert_eq!(FailureReason::from_error(&error).to_string(), "invalid_rpc_batch");
        assert_eq!(error.to_string(), "invalid_rpc_batch status=200 detail=ids");

        let error = JsonRpcHandler::order_batch(&calls, Vec::new(), 429).unwrap_err();
        assert_eq!(FailureReason::from_error(&error).to_string(), "status=429");
        assert_eq!(FailureReason::error_detail(&error), "invalid_rpc_batch status=429 detail=length");
    }

    #[test]
    fn test_merge_batch_results_preserves_request_order() {
        let cached = serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "result": "cached",
            "id": 1
        }))
        .unwrap();
        let upstream = serde_json::from_value(json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32000,
                "message": "upstream",
                "data": { "retry": true }
            },
            "id": 2
        }))
        .unwrap();

        let merged = JsonRpcHandler::merge_batch_results(vec![Some(cached), None], vec![upstream], 200).unwrap();

        assert_eq!(
            serde_json::to_value(merged).unwrap(),
            json!([
                { "jsonrpc": "2.0", "result": "cached", "id": 1 },
                {
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32000,
                        "message": "upstream",
                        "data": { "retry": true }
                    },
                    "id": 2
                }
            ])
        );
        let error = JsonRpcHandler::merge_batch_results(vec![None], Vec::new(), 200).unwrap_err();
        assert_eq!(FailureReason::from_error(error.as_ref()).to_string(), "invalid_rpc_batch");
        assert_eq!(error.to_string(), "invalid_rpc_batch status=200 detail=partial_length");
    }
}
