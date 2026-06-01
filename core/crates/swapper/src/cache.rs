use gem_client::X_CACHE_TTL;
use std::collections::HashMap;

pub(crate) const STATIC_READ_CACHE_TTL_SECONDS: u64 = 30 * primitives::duration::DAY.as_secs();

pub(crate) fn cache_headers(ttl_seconds: u64) -> HashMap<String, String> {
    HashMap::from([(X_CACHE_TTL.to_string(), ttl_seconds.to_string())])
}

pub(crate) fn static_read_cache_headers() -> HashMap<String, String> {
    cache_headers(STATIC_READ_CACHE_TTL_SECONDS)
}
