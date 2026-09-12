use gem_proxy::allowlist::PathRule;
use serde::Deserialize;

use crate::jsonrpc_types::{JsonRpcRequest, RequestType};

use super::path_without_query;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
pub struct AllowlistConfig(Vec<AllowlistRule>);

impl AllowlistConfig {
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn allows(&self, request_type: &RequestType) -> bool {
        if self.0.is_empty() {
            return true;
        }

        match request_type {
            RequestType::Regular { path, method, .. } => self.0.iter().any(|rule| rule.matches_path_request(path, method)),
            RequestType::JsonRpc(JsonRpcRequest::Single(call)) => self.0.iter().any(|rule| rule.matches_rpc(&call.method)),
            RequestType::JsonRpc(JsonRpcRequest::Batch(calls)) => calls.iter().all(|call| self.0.iter().any(|rule| rule.matches_rpc(&call.method))),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum AllowlistRule {
    Rpc { rpc_method: String },
    Path(PathRule),
}

impl AllowlistRule {
    fn matches_path_request(&self, path: &str, method: &str) -> bool {
        match self {
            Self::Path(rule) => rule.matches(method, path_without_query(path)),
            Self::Rpc { .. } => false,
        }
    }

    fn matches_rpc(&self, rpc_method: &str) -> bool {
        matches!(self, Self::Rpc { rpc_method: allowed } if allowed == rpc_method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::config::jsonrpc;
    use config::{Config, File, FileFormat};
    use serde_json::json;

    fn config() -> AllowlistConfig {
        serde_json::from_value(json!([
            { "rpc_method": "eth_call" },
            { "rpc_method": "eth_chainId" },
            { "path": "/api/v2/address/**", "method": "GET" },
            { "path": "/api/v2/sendtx/", "method": "POST" }
        ]))
        .unwrap()
    }

    #[test]
    fn test_allows_jsonrpc_single_method() {
        let config = config();

        assert!(config.allows(&jsonrpc("eth_call")));
        assert!(!config.allows(&jsonrpc("unsupported_method")));
    }

    #[test]
    fn test_allows_jsonrpc_batch_only_when_all_methods_allowed() {
        let config = config();
        let allowed = RequestType::from_request(
            "POST",
            "/".to_string(),
            serde_json::to_vec(&json!([
                {"jsonrpc": "2.0", "method": "eth_call", "params": [], "id": 1},
                {"jsonrpc": "2.0", "method": "eth_chainId", "params": [], "id": 2}
            ]))
            .unwrap(),
        );
        let denied = RequestType::from_request(
            "POST",
            "/".to_string(),
            serde_json::to_vec(&json!([
                {"jsonrpc": "2.0", "method": "eth_call", "params": [], "id": 1},
                {"jsonrpc": "2.0", "method": "unsupported_method", "params": [], "id": 2}
            ]))
            .unwrap(),
        );

        assert!(config.allows(&allowed));
        assert!(!config.allows(&denied));
    }

    #[test]
    fn test_allows_http_path_wildcard_without_query() {
        let config = config();
        let request = RequestType::from_request("GET", "/api/v2/address/bc1qtest?pageSize=25&details=txs".to_string(), Vec::new());

        assert!(config.allows(&request));
    }

    #[test]
    fn test_denies_unlisted_bitcoin_block_path() {
        let config = config();
        let request = RequestType::from_request("GET", "/api/v2/block/900000".to_string(), Vec::new());

        assert!(!config.allows(&request));
    }

    #[test]
    fn test_denies_http_path_method_mismatch() {
        let config = config();
        let request = RequestType::from_request("GET", "/api/v2/sendtx/".to_string(), Vec::new());

        assert!(!config.allows(&request));
    }

    #[test]
    fn test_empty_rules_are_unrestricted() {
        let config: AllowlistConfig = serde_json::from_value(json!([])).unwrap();

        assert!(config.allows(&jsonrpc("unknown_method")));
    }

    #[test]
    fn test_deserializes_mixed_rules_from_yaml() {
        #[derive(Deserialize)]
        struct Wrapper {
            allowlist: AllowlistConfig,
        }

        let input = "allowlist:\n  - rpc_method: eth_call\n  - path: /api/v2/address/**\n    method: GET\n";
        let wrapper = Config::builder()
            .add_source(File::from_str(input, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<Wrapper>()
            .unwrap();

        assert!(wrapper.allowlist.allows(&jsonrpc("eth_call")));
        assert!(!wrapper.allowlist.allows(&jsonrpc("eth_chainId")));
        assert!(wrapper.allowlist.allows(&RequestType::from_request("GET", "/api/v2/address/bc1q".to_string(), Vec::new())));
        assert!(!wrapper.allowlist.allows(&RequestType::from_request("GET", "/api/v2/block/1".to_string(), Vec::new())));
    }
}
