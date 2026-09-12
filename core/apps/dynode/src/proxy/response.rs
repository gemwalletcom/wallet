use std::time::Duration;

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};

use super::constants::{JSON_CONTENT_TYPE, JSON_HEADER};

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
const X_UPSTREAM_LATENCY: HeaderName = HeaderName::from_static("x-upstream-latency");
const X_CACHE: HeaderName = HeaderName::from_static("x-cache");

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CacheStatus {
    Hit,
    Miss,
    Partial,
}

impl CacheStatus {
    fn value(self) -> HeaderValue {
        match self {
            Self::Hit => HeaderValue::from_static("HIT"),
            Self::Miss => HeaderValue::from_static("MISS"),
            Self::Partial => HeaderValue::from_static("PARTIAL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProxyResponse {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    from_cache: bool,
}

impl ProxyResponse {
    pub fn new(status: u16, headers: HeaderMap, body: Vec<u8>) -> Self {
        Self {
            status,
            headers,
            body,
            from_cache: false,
        }
    }

    pub(crate) fn with_content_type(status: u16, body: Vec<u8>, content_type: &str) -> Self {
        let content_type = if content_type == JSON_CONTENT_TYPE {
            JSON_HEADER.clone()
        } else {
            HeaderValue::from_str(content_type).unwrap_or_else(|_| JSON_HEADER.clone())
        };
        Self::new(status, HeaderMap::from_iter([(CONTENT_TYPE, content_type)]), body)
    }

    pub(crate) fn with_proxy_headers(mut self, request_id: &str, latency: Duration, cache_status: CacheStatus) -> Self {
        self.headers
            .insert(X_REQUEST_ID, HeaderValue::from_str(request_id).unwrap_or_else(|_| HeaderValue::from_static("unknown")));
        self.headers.insert(X_CACHE, cache_status.value());
        self.headers.insert(
            X_UPSTREAM_LATENCY,
            HeaderValue::from_str(&format!("{}ms", latency.as_millis())).unwrap_or_else(|_| HeaderValue::from_static("0ms")),
        );
        self.from_cache = cache_status == CacheStatus::Hit;
        self
    }

    pub(crate) fn into_cached(mut self) -> Self {
        self.from_cache = true;
        self
    }

    pub(crate) fn is_from_cache(&self) -> bool {
        self.from_cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_headers_preserve_payload_and_cache_source() {
        let response = ProxyResponse::with_content_type(200, b"body".to_vec(), JSON_CONTENT_TYPE);
        for (status, value, cached) in [
            (CacheStatus::Hit, "HIT", true),
            (CacheStatus::Miss, "MISS", false),
            (CacheStatus::Partial, "PARTIAL", false),
        ] {
            let annotated = response.clone().with_proxy_headers("request-id", Duration::from_millis(42), status);
            assert_eq!(annotated.status, response.status);
            assert_eq!(annotated.body, response.body);
            assert_eq!(annotated.is_from_cache(), cached);
            assert_eq!(
                annotated.headers,
                HeaderMap::from_iter([
                    (CONTENT_TYPE, JSON_HEADER.clone()),
                    (X_REQUEST_ID, HeaderValue::from_static("request-id")),
                    (X_UPSTREAM_LATENCY, HeaderValue::from_static("42ms")),
                    (X_CACHE, HeaderValue::from_static(value)),
                ])
            );
        }
    }

    #[test]
    fn test_content_type_preserves_valid_values_and_json_fallback() {
        for (content_type, expected) in [
            (JSON_CONTENT_TYPE, JSON_CONTENT_TYPE),
            ("text/plain; charset=utf-8", "text/plain; charset=utf-8"),
            ("invalid\nheader", JSON_CONTENT_TYPE),
        ] {
            let response = ProxyResponse::with_content_type(201, b"body".to_vec(), content_type);
            assert_eq!(response.status, 201);
            assert_eq!(response.body, b"body".to_vec());
            assert_eq!(response.headers, HeaderMap::from_iter([(CONTENT_TYPE, HeaderValue::from_str(expected).unwrap())]));
            assert!(!response.is_from_cache());
        }
    }
}
