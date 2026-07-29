use gem_client::X_CACHE_TTL;
use primitives::MONTH;
use std::collections::HashMap;

pub(crate) fn static_read_cache_headers() -> HashMap<String, String> {
    HashMap::from([(X_CACHE_TTL.to_string(), MONTH.as_secs().to_string())])
}
