use crate::jsonrpc_types::RequestType;
use primitives::Chain;
use reqwest::{Method, header::HeaderMap};
use std::time::Instant;
use uuid::Uuid;

fn generate_request_id() -> String {
    format!("{:016x}", Uuid::new_v4().as_u128() as u64)
}

#[derive(Debug, Clone)]
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

    pub fn elapsed(&self) -> std::time::Duration {
        self.request_start.elapsed()
    }

    pub fn request_type(&self) -> &RequestType {
        &self.request_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn request_creation() {
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
    fn elapsed_time() {
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
    fn generate_request_id_unique() {
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
}
