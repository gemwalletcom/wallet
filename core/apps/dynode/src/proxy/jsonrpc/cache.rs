use std::time::Duration;

use gem_tracing::{DurationMs, info_with_fields};
use reqwest::StatusCode;
use serde_json::Value;

use crate::cache::{CacheProvider, RequestCache};
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcResponse, JsonRpcResult};
use crate::proxy::CachedResponse;
use crate::proxy::constants::JSON_CONTENT_TYPE;
use crate::proxy::proxy_request::ProxyRequest;

pub(super) async fn get(call: &JsonRpcCall, request: &ProxyRequest, cache: &RequestCache) -> Option<JsonRpcResult> {
    cache
        .get(&request.chain, &call.cache_key(&request.host, &request.path_with_query))
        .await
        .and_then(|response| result(call, &response))
}

pub(super) async fn get_many(calls: &[JsonRpcCall], ttls: &[Option<Duration>], request: &ProxyRequest, cache: &RequestCache) -> Vec<Option<JsonRpcResult>> {
    let mut results = Vec::with_capacity(calls.len());
    for (call, ttl) in calls.iter().zip(ttls) {
        results.push(if ttl.is_some() { get(call, request, cache).await } else { None });
    }
    results
}

pub(super) async fn set(
    call: &JsonRpcCall,
    response: &JsonRpcResponse,
    ttl: Duration,
    request: &ProxyRequest,
    cache: &RequestCache,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    set_result(call, &response.result, ttl, request, cache).await
}

async fn set_result(call: &JsonRpcCall, result: &Value, ttl: Duration, request: &ProxyRequest, cache: &RequestCache) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let result = serde_json::to_vec(result)?;
    let size = result.len();
    let cached = CachedResponse::new(result, StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string());
    cache.set(&request.chain, call.cache_key(&request.host, &request.path_with_query), cached, ttl).await;

    info_with_fields!(
        "Cache SET",
        id = request.id.as_str(),
        chain = request.chain.as_ref(),
        host = request.host.as_str(),
        method = call.method.as_str(),
        ttl_ms = ttl.as_millis(),
        size_bytes = size,
        latency = DurationMs(request.elapsed()),
    );
    Ok(())
}

pub(super) async fn set_many(
    calls: &[JsonRpcCall],
    ttls: &[Option<Duration>],
    responses: &[Value],
    request: &ProxyRequest,
    cache: &RequestCache,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if calls.len() != ttls.len() || calls.len() != responses.len() {
        return Ok(());
    }
    if calls
        .iter()
        .zip(responses)
        .any(|(call, response)| response.get("id").and_then(Value::as_u64) != Some(call.id))
    {
        return Ok(());
    }

    for ((call, ttl), response) in calls.iter().zip(ttls).zip(responses) {
        if let (Some(ttl), Some(result)) = (ttl, response.get("result")) {
            set_result(call, result, *ttl, request, cache).await?;
        }
    }
    Ok(())
}

fn result(call: &JsonRpcCall, cached: &CachedResponse) -> Option<JsonRpcResult> {
    Some(JsonRpcResult::Success(JsonRpcResponse {
        jsonrpc: call.jsonrpc.clone(),
        result: serde_json::from_slice(&cached.body).ok()?,
        id: Some(call.id),
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_cached_result_uses_request_id_and_rejects_invalid_json() {
        let call = JsonRpcCall::mock(42, "eth_chainId");
        let cached = CachedResponse::new(br#""0x1""#.to_vec(), 200, JSON_CONTENT_TYPE.to_string());
        let invalid = CachedResponse::new(b"invalid".to_vec(), 200, JSON_CONTENT_TYPE.to_string());

        assert_eq!(
            serde_json::to_value(result(&call, &cached).unwrap()).unwrap(),
            json!({ "jsonrpc": "2.0", "result": "0x1", "id": 42 })
        );
        assert!(result(&call, &invalid).is_none());
    }
}
