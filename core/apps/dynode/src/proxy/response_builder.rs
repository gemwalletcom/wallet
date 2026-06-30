use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use std::time::Duration;

use super::constants::{JSON_CONTENT_TYPE, JSON_HEADER};

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
const X_UPSTREAM_LATENCY: HeaderName = HeaderName::from_static("x-upstream-latency");

#[derive(Debug, Clone)]
pub struct ProxyResponse {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl ProxyResponse {
    pub fn new(status: u16, headers: HeaderMap, body: Vec<u8>) -> Self {
        Self { status, headers, body }
    }
}

pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn create_proxy_headers(request_id: &str, latency: Duration) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(X_REQUEST_ID, HeaderValue::from_str(request_id).unwrap_or_else(|_| HeaderValue::from_static("unknown")));
        headers.insert(
            X_UPSTREAM_LATENCY,
            HeaderValue::from_str(&format!("{}ms", latency.as_millis())).unwrap_or_else(|_| HeaderValue::from_static("0ms")),
        );

        headers
    }

    pub fn build_with_headers(data: Vec<u8>, status: u16, content_type: &str, additional_headers: HeaderMap) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = HeaderMap::new();

        let content_header = if content_type == JSON_CONTENT_TYPE {
            JSON_HEADER.clone()
        } else {
            HeaderValue::from_str(content_type).unwrap_or_else(|_| JSON_HEADER.clone())
        };

        headers.insert(header::CONTENT_TYPE, content_header);
        headers.extend(additional_headers);

        Ok(ProxyResponse::new(status, headers, data))
    }

    pub fn build_cached_with_headers(cached: super::CachedResponse, additional_headers: HeaderMap) -> ProxyResponse {
        let mut headers = HeaderMap::new();

        let content_header = if cached.content_type == JSON_CONTENT_TYPE {
            JSON_HEADER.clone()
        } else {
            HeaderValue::from_str(&cached.content_type).unwrap_or_else(|_| JSON_HEADER.clone())
        };

        headers.insert(header::CONTENT_TYPE, content_header);
        headers.extend(additional_headers);

        ProxyResponse::new(cached.status, headers, cached.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_proxy_headers() {
        let headers = ResponseBuilder::create_proxy_headers("request-id", Duration::from_millis(42));

        assert_eq!(headers.get("x-request-id").unwrap(), "request-id");
        assert_eq!(headers.get("x-upstream-latency").unwrap(), "42ms");
        assert!(headers.get("x-upstream-host").is_none());
    }
}
