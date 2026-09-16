use std::time::{Duration, Instant};

use crate::proxy::ProxyResponse;

#[derive(Debug)]
pub struct CacheEntry {
    pub response: ProxyResponse,
    pub expires_at: Option<Instant>,
    pub created_at: Instant,
}

impl CacheEntry {
    pub fn new(response: ProxyResponse, ttl: Duration) -> Self {
        let created_at = Instant::now();
        let expires_at = if ttl.is_zero() { None } else { Some(created_at + ttl) };
        Self { response, expires_at, created_at }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| Instant::now() > exp)
    }

    pub fn size(&self) -> usize {
        self.response.body.len()
            + self
                .response
                .headers
                .iter()
                .map(|(name, value)| name.as_str().len() + value.as_bytes().len())
                .sum::<usize>()
            + 64
    }
}

#[cfg(test)]
mod tests {
    use primitives::MINUTE;
    use reqwest::StatusCode;

    use super::*;
    use crate::proxy::constants::JSON_CONTENT_TYPE;

    #[test]
    fn test_cache_entry_with_ttl() {
        let response = ProxyResponse::with_content_type(StatusCode::OK.as_u16(), b"test".to_vec(), JSON_CONTENT_TYPE);
        let entry = CacheEntry::new(response, MINUTE);

        assert!(entry.expires_at.is_some());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_without_ttl() {
        let response = ProxyResponse::with_content_type(StatusCode::OK.as_u16(), b"test".to_vec(), JSON_CONTENT_TYPE);
        let entry = CacheEntry::new(response, Duration::ZERO);

        assert!(entry.expires_at.is_none());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_size() {
        let body = b"hello world".to_vec();
        let content_type = "application/json".to_string();
        let response = ProxyResponse::with_content_type(StatusCode::OK.as_u16(), body.clone(), &content_type);
        let entry = CacheEntry::new(response, MINUTE);

        assert_eq!(entry.size(), body.len() + "content-type".len() + content_type.len() + 64);
    }
}
