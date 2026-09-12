use std::time::Duration;

use gem_tracing::{DurationMs, info_with_fields};
use reqwest::StatusCode;
use serde_json::Value;

use crate::BoxError;
use crate::cache::RequestCache;
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcResponse, JsonRpcResult};
use crate::proxy::ProxyResponse;
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

pub(super) async fn set_result(call: &JsonRpcCall, result: &Value, ttl: Duration, request: &ProxyRequest, cache: &RequestCache) -> Result<(), BoxError> {
    let result = serde_json::to_vec(result)?;
    let size = result.len();
    let cached = ProxyResponse::with_content_type(StatusCode::OK.as_u16(), result, JSON_CONTENT_TYPE);
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

pub(super) async fn set_many(calls: &[JsonRpcCall], ttls: &[Option<Duration>], responses: &[Value], request: &ProxyRequest, cache: &RequestCache) -> Result<(), BoxError> {
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

fn result(call: &JsonRpcCall, cached: &ProxyResponse) -> Option<JsonRpcResult> {
    Some(JsonRpcResult::Success(JsonRpcResponse {
        jsonrpc: call.jsonrpc.clone(),
        result: serde_json::from_slice(&cached.body).ok()?,
        id: Some(call.id),
    }))
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, MINUTE};
    use reqwest::Method;
    use reqwest::header::{CONTENT_TYPE, HOST, HeaderMap, HeaderValue};
    use serde_json::json;

    use super::*;
    use crate::config::{CacheConfig, ChainTypesConfig, MemoryConfig};
    use crate::testkit::config::chain_config;

    #[tokio::test]
    async fn test_result_cache_roundtrip_rebuilds_request_id() {
        let policies: ChainTypesConfig = serde_json::from_value(json!({
            "ethereum": { "cache": [{ "rpc_method": "eth_chainId", "ttl": "1m" }] }
        }))
        .unwrap();
        let chains = [chain_config(Chain::Ethereum, "https://example.com")];
        let cache = RequestCache::for_chains(
            &CacheConfig {
                memory: MemoryConfig { max: 1_000_000 },
            },
            &policies,
            chains.iter(),
        );
        let request = ProxyRequest::from_http(
            Method::POST,
            HeaderMap::from_iter([(HOST, HeaderValue::from_static("example.com"))]),
            Vec::new(),
            "/ethereum",
            Chain::Ethereum,
        )
        .unwrap();
        let call = JsonRpcCall::mock(1, "eth_chainId");
        set_result(&call, &json!("0x1"), MINUTE, &request, &cache).await.unwrap();
        let stored = cache.get(&Chain::Ethereum, &call.cache_key(&request.host, &request.path_with_query)).await.unwrap();
        assert_eq!(stored.body, br#""0x1""#.to_vec());
        assert_eq!(stored.headers, HeaderMap::from_iter([(CONTENT_TYPE, HeaderValue::from_static(JSON_CONTENT_TYPE))]));
        for id in [1, 42] {
            assert_eq!(
                serde_json::to_value(get(&JsonRpcCall::mock(id, "eth_chainId"), &request, &cache).await.unwrap()).unwrap(),
                json!({ "jsonrpc": "2.0", "result": "0x1", "id": id })
            );
        }
    }

    #[test]
    fn test_cached_result_uses_request_id_and_rejects_invalid_json() {
        let call = JsonRpcCall::mock(42, "eth_chainId");
        let cached = ProxyResponse::with_content_type(200, br#""0x1""#.to_vec(), JSON_CONTENT_TYPE);
        let invalid = ProxyResponse::with_content_type(200, b"invalid".to_vec(), JSON_CONTENT_TYPE);

        assert_eq!(
            serde_json::to_value(result(&call, &cached).unwrap()).unwrap(),
            json!({ "jsonrpc": "2.0", "result": "0x1", "id": 42 })
        );
        assert!(result(&call, &invalid).is_none());
    }
}
