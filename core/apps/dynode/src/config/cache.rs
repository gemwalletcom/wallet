use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;
use serde_serializers::duration;

use super::path_without_query;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(default)]
    pub max_memory_mb: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheRule {
    pub(crate) path: Option<String>,
    pub(crate) method: Option<String>,
    pub(crate) rpc_method: Option<String>,
    #[serde(default, alias = "ttl_seconds", deserialize_with = "duration::deserialize_option")]
    pub(crate) ttl: Option<Duration>,
    #[serde(default)]
    pub(crate) params: HashMap<String, Value>,
}

impl CacheRule {
    pub(crate) fn matches_path_request(&self, path: &str, method: &str, body: Option<&[u8]>) -> bool {
        let Some(rule_method) = self.method.as_ref() else {
            return false;
        };
        if method != rule_method {
            return false;
        }

        let Some(rule_path) = self.path.as_ref() else {
            return false;
        };
        path_without_query(path) == rule_path && self.matches_body(body)
    }

    pub(crate) fn matches_rpc(&self, rpc_method: &str) -> bool {
        self.rpc_method.as_ref().is_some_and(|m| m == rpc_method)
    }

    fn matches_body(&self, body: Option<&[u8]>) -> bool {
        if self.params.is_empty() {
            return true;
        }

        let Some(body_bytes) = body else {
            return false;
        };

        let Ok(value) = serde_json::from_slice::<Value>(body_bytes) else {
            return false;
        };

        let Some(object) = value.as_object() else {
            return false;
        };

        self.params.iter().all(|(key, expected)| object.get(key).map(|actual| actual == expected).unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_default_none() {
        let rule: CacheRule = serde_json::from_value(serde_json::json!({
            "path": "/wallet/getaccount",
            "method": "POST"
        }))
        .unwrap();

        assert_eq!(rule.ttl, None);
    }

    #[test]
    fn test_ttl_duration_string() {
        let rule: CacheRule = serde_json::from_value(serde_json::json!({
            "path": "/api/data",
            "method": "GET",
            "ttl": "1m"
        }))
        .unwrap();

        assert_eq!(rule.ttl, Some(Duration::from_secs(60)));
    }
}
