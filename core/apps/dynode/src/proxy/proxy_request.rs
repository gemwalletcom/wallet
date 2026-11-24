use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::time::{Duration, Instant};

use bytes::Bytes;
use chain_providers::BroadcastProviders;
use http::StatusCode;
use primitives::{Chain, ChainRequest, ChainRequestProtocol, ChainRequestType};
use reqwest::Method;
use reqwest::header::{HOST, HeaderMap, USER_AGENT};
use url::Url;

use crate::config::path::path_without_query;
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcRequest, RequestType};

fn generate_request_id() -> String {
    format!("{:016x}", RandomState::new().hash_one(0u8))
}

#[derive(Debug)]
pub struct ProxyRequest {
    pub id: String,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub path: String,
    pub path_with_query: String,
    pub host: String,
    pub user_agent: String,
    pub chain: Chain,
    pub request_start: Instant,
    request_type: RequestType,
}

impl ProxyRequest {
    pub fn from_http(method: Method, headers: HeaderMap, body: Bytes, uri: &str, chain: Chain) -> Result<Self, StatusCode> {
        let host = headers.get(HOST).and_then(|header| header.to_str().ok()).ok_or(StatusCode::BAD_REQUEST)?;
        let host = Self::parse_hostname(host);
        let user_agent = headers.get(USER_AGENT).and_then(|header| header.to_str().ok()).unwrap_or_default().to_string();
        let (path, path_with_query) = Self::prepare_paths(uri)?;
        Ok(Self::new(method, headers, body, path, path_with_query, host, user_agent, chain))
    }

    pub fn new(method: Method, headers: HeaderMap, body: Bytes, path: String, path_with_query: String, host: String, user_agent: String, chain: Chain) -> Self {
        let request_type = RequestType::from_request(method.as_str(), path_with_query.clone(), body.clone());
        Self {
            id: generate_request_id(),
            method,
            headers,
            body,
            path,
            path_with_query,
            host,
            user_agent,
            chain,
            request_start: Instant::now(),
            request_type,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.request_start.elapsed()
    }

    pub fn request_type(&self) -> &RequestType {
        &self.request_type
    }

    pub(crate) fn rpc_method(&self) -> Option<&str> {
        match self.request_type() {
            RequestType::JsonRpc(JsonRpcRequest::Single(call)) => Some(call.method.as_str()),
            RequestType::JsonRpc(JsonRpcRequest::Batch(calls)) => {
                let method = calls.first()?.method.as_str();
                calls.iter().all(|call| call.method == method).then_some(method)
            }
            RequestType::Regular { .. } => None,
        }
    }

    pub(crate) fn with_calls(&self, calls: Vec<JsonRpcCall>) -> Self {
        let (body, rpc_request) = match <[JsonRpcCall; 1]>::try_from(calls) {
            Ok([call]) => (serde_json::to_vec(&call), JsonRpcRequest::Single(call)),
            Err(calls) => (serde_json::to_vec(&calls), JsonRpcRequest::Batch(calls)),
        };
        Self {
            id: self.id.clone(),
            method: self.method.clone(),
            headers: self.headers.clone(),
            body: Bytes::from(body.unwrap_or_default()),
            path: self.path.clone(),
            path_with_query: self.path_with_query.clone(),
            host: self.host.clone(),
            user_agent: self.user_agent.clone(),
            chain: self.chain,
            request_start: self.request_start,
            request_type: RequestType::JsonRpc(rpc_request),
        }
    }

    pub(crate) fn is_broadcast(&self, providers: &BroadcastProviders) -> bool {
        let request = match self.request_type() {
            RequestType::JsonRpc(JsonRpcRequest::Single(call)) => ChainRequest::new(ChainRequestProtocol::JsonRpc, &call.method, &self.path, &self.body),
            RequestType::Regular { .. } => ChainRequest::new(ChainRequestProtocol::Http, self.method.as_str(), &self.path, &self.body),
            RequestType::JsonRpc(JsonRpcRequest::Batch(_)) => return false,
        };
        providers.classify_request(self.chain, request) == ChainRequestType::Broadcast
    }

    fn prepare_paths(uri: &str) -> Result<(String, String), StatusCode> {
        let path_with_query = Self::canonicalize_path(&Self::remove_chain_from_path(uri))?;
        let path = path_without_query(&path_with_query).to_string();
        Ok((path, path_with_query))
    }

    fn canonicalize_path(path_with_query: &str) -> Result<String, StatusCode> {
        Url::parse("http://0.0.0.0")
            .and_then(|base| base.join(path_with_query))
            .map(|resolved| match resolved.query() {
                Some(query) => format!("{}?{}", resolved.path(), query),
                None => resolved.path().to_string(),
            })
            .map_err(|_| StatusCode::BAD_REQUEST)
    }

    fn parse_hostname(host_header: &str) -> String {
        let candidate = format!("http://{}", host_header);
        Url::parse(&candidate).ok().and_then(|url| url.host_str().map(str::to_string)).unwrap_or_else(|| host_header.to_string())
    }

    fn remove_chain_from_path(uri: &str) -> String {
        let (path_part, query_part) = uri.split_once('?').unwrap_or((uri, ""));

        let remaining = path_part.trim_start_matches('/').split_once('/').map(|(_, rest)| format!("/{}", rest)).unwrap_or_else(|| "/".to_string());

        if query_part.is_empty() { remaining } else { format!("{}?{}", remaining, query_part) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_broadcast_jsonrpc_single() {
        let request = ProxyRequest::mock(Chain::Ethereum, Method::POST, "/rpc", br#"{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0xdeadbeef"],"id":1}"#);

        assert!(request.is_broadcast(&BroadcastProviders::from_chains([Chain::Ethereum, Chain::Tron])));
    }

    #[test]
    fn test_detect_broadcast_batch_jsonrpc_skipped() {
        let request = ProxyRequest::mock(
            Chain::Ethereum,
            Method::POST,
            "/rpc",
            br#"[{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0x1"],"id":1},{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0x2"],"id":2}]"#,
        );

        assert!(!request.is_broadcast(&BroadcastProviders::from_chains([Chain::Ethereum, Chain::Tron])));
    }

    #[test]
    fn test_detect_broadcast_http_path() {
        let request = ProxyRequest::mock(Chain::Tron, Method::POST, "/wallet/broadcasttransaction", br#"{"txID":"abc"}"#);

        assert!(request.is_broadcast(&BroadcastProviders::from_chains([Chain::Ethereum, Chain::Tron])));
    }

    #[test]
    fn test_request_creation() {
        let request = ProxyRequest::new(
            Method::GET,
            HeaderMap::new(),
            Bytes::new(),
            "/test".to_string(),
            "/test?param=1".to_string(),
            "example.com".to_string(),
            "test-agent".to_string(),
            Chain::Ethereum,
        );

        assert_eq!(request.method, Method::GET);
        assert_eq!(request.path, "/test");
        assert_eq!(request.host, "example.com");
        assert_eq!(request.user_agent, "test-agent");
        assert_eq!(request.chain, Chain::Ethereum);
    }

    #[test]
    fn test_generate_request_id_unique() {
        let id1 = super::generate_request_id();
        let id2 = super::generate_request_id();
        let id3 = super::generate_request_id();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_eq!(id1.len(), 16);
        assert_eq!(id2.len(), 16);
        assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(id2.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(id3.chars().all(|c| c.is_ascii_hexdigit()));
    }
    #[test]
    fn test_remove_chain_from_path() {
        assert_eq!(ProxyRequest::remove_chain_from_path("/tron/wallet/getchainparameters"), "/wallet/getchainparameters");
        assert_eq!(ProxyRequest::remove_chain_from_path("/ethereum/v1/some/path"), "/v1/some/path");
        assert_eq!(ProxyRequest::remove_chain_from_path("/bitcoin"), "/");
        assert_eq!(ProxyRequest::remove_chain_from_path("/solana?query=1"), "/?query=1");
        assert_eq!(ProxyRequest::remove_chain_from_path("/chain/path?foo=bar&baz=qux"), "/path?foo=bar&baz=qux");
    }

    #[test]
    fn test_prepare_paths() {
        assert_eq!(ProxyRequest::prepare_paths("/bitcoin/api/v2/address/../block/900000"), Ok(("/api/v2/block/900000".to_string(), "/api/v2/block/900000".to_string())));
        assert_eq!(ProxyRequest::prepare_paths("/bitcoin/api/v2/address/%2e%2e/block"), Ok(("/api/v2/block".to_string(), "/api/v2/block".to_string())));
        assert_eq!(ProxyRequest::prepare_paths("/ethereum/../secret"), Ok(("/secret".to_string(), "/secret".to_string())));
        assert_eq!(
            ProxyRequest::prepare_paths("/bitcoin/api/v2/address/bc1qtest?page=1"),
            Ok(("/api/v2/address/bc1qtest".to_string(), "/api/v2/address/bc1qtest?page=1".to_string()))
        );
        for uri in ["/ethereum//x:y/../../../../admin", "/ethereum/\\x:y\\..\\..\\..\\..\\admin", "/ethereum//[/../../admin"] {
            assert_eq!(ProxyRequest::prepare_paths(uri), Err(StatusCode::BAD_REQUEST));
        }
    }

    #[test]
    fn test_parse_hostname() {
        assert_eq!(ProxyRequest::parse_hostname("example.com"), "example.com");
        assert_eq!(ProxyRequest::parse_hostname("example.com:8080"), "example.com");
        assert_eq!(ProxyRequest::parse_hostname("localhost:3000"), "localhost");
    }
}
