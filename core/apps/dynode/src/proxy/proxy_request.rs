use std::time::{Duration, Instant};

use primitives::Chain;
use reqwest::Method;
use reqwest::header::{HOST, HeaderMap, USER_AGENT};
use rocket::http::Status;
use url::Url;
use uuid::Uuid;

use crate::config::path::path_without_query;
use crate::jsonrpc_types::RequestType;

fn generate_request_id() -> String {
    format!("{:016x}", Uuid::new_v4().as_u128() as u64)
}

#[derive(Debug)]
pub struct ProxyRequest {
    pub id: String,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub path: String,
    pub path_with_query: String,
    pub host: String,
    pub user_agent: String,
    pub chain: Chain,
    pub request_start: Instant,
    request_type: RequestType,
}

impl ProxyRequest {
    pub fn from_http(method: Method, headers: HeaderMap, body: Vec<u8>, uri: &str, chain: Chain) -> Result<Self, Status> {
        let host = headers.get(HOST).and_then(|header| header.to_str().ok()).ok_or(Status::BadRequest)?;
        let host = Self::parse_hostname(host);
        let user_agent = headers.get(USER_AGENT).and_then(|header| header.to_str().ok()).unwrap_or_default().to_string();
        let (path, path_with_query) = Self::prepare_paths(uri);
        Ok(Self::new(method, headers, body, path, path_with_query, host, user_agent, chain))
    }

    pub fn new(method: Method, headers: HeaderMap, body: Vec<u8>, path: String, path_with_query: String, host: String, user_agent: String, chain: Chain) -> Self {
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

    fn prepare_paths(uri: &str) -> (String, String) {
        let path_with_query = Self::canonicalize_path(&Self::remove_chain_from_path(uri));
        let path = path_without_query(&path_with_query).to_string();
        (path, path_with_query)
    }

    fn canonicalize_path(path_with_query: &str) -> String {
        Url::parse("http://0.0.0.0")
            .and_then(|base| base.join(path_with_query))
            .map(|resolved| match resolved.query() {
                Some(query) => format!("{}?{}", resolved.path(), query),
                None => resolved.path().to_string(),
            })
            .unwrap_or_else(|_| path_with_query.to_string())
    }

    fn parse_hostname(host_header: &str) -> String {
        let candidate = format!("http://{}", host_header);
        Url::parse(&candidate)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string))
            .unwrap_or_else(|| host_header.to_string())
    }

    fn remove_chain_from_path(uri: &str) -> String {
        let (path_part, query_part) = uri.split_once('?').unwrap_or((uri, ""));

        let remaining = path_part
            .trim_start_matches('/')
            .split_once('/')
            .map(|(_, rest)| format!("/{}", rest))
            .unwrap_or_else(|| "/".to_string());

        if query_part.is_empty() { remaining } else { format!("{}?{}", remaining, query_part) }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_request_creation() {
        let ctx = ProxyRequest::new(
            Method::GET,
            HeaderMap::new(),
            vec![],
            "/test".to_string(),
            "/test?param=1".to_string(),
            "example.com".to_string(),
            "test-agent".to_string(),
            Chain::Ethereum,
        );

        assert_eq!(ctx.method, Method::GET);
        assert_eq!(ctx.path, "/test");
        assert_eq!(ctx.host, "example.com");
        assert_eq!(ctx.user_agent, "test-agent");
        assert_eq!(ctx.chain, Chain::Ethereum);
    }

    #[test]
    fn test_elapsed_time() {
        let ctx = ProxyRequest::new(
            Method::GET,
            HeaderMap::new(),
            vec![],
            "/test".to_string(),
            "/test".to_string(),
            "example.com".to_string(),
            "test-agent".to_string(),
            Chain::Ethereum,
        );

        thread::sleep(Duration::from_millis(1));

        let elapsed = ctx.elapsed();
        assert!(elapsed.as_millis() > 0);
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
        assert_eq!(
            ProxyRequest::prepare_paths("/bitcoin/api/v2/address/../block/900000"),
            ("/api/v2/block/900000".to_string(), "/api/v2/block/900000".to_string())
        );
        assert_eq!(
            ProxyRequest::prepare_paths("/bitcoin/api/v2/address/%2e%2e/block"),
            ("/api/v2/block".to_string(), "/api/v2/block".to_string())
        );
        assert_eq!(ProxyRequest::prepare_paths("/ethereum/../secret"), ("/secret".to_string(), "/secret".to_string()));
        assert_eq!(
            ProxyRequest::prepare_paths("/bitcoin/api/v2/address/bc1qtest?page=1"),
            ("/api/v2/address/bc1qtest".to_string(), "/api/v2/address/bc1qtest?page=1".to_string())
        );
    }

    #[test]
    fn test_parse_hostname() {
        assert_eq!(ProxyRequest::parse_hostname("example.com"), "example.com");
        assert_eq!(ProxyRequest::parse_hostname("example.com:8080"), "example.com");
        assert_eq!(ProxyRequest::parse_hostname("localhost:3000"), "localhost");
    }
}
