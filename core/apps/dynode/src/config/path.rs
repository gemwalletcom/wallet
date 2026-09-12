use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
pub struct PathAllowlist(Vec<PathRule>);

impl PathAllowlist {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn allows(&self, method: &str, path: &str) -> bool {
        self.is_empty() || self.0.iter().any(|rule| rule.matches(method, path))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PathRule {
    pub path: String,
    pub method: String,
}

impl PathRule {
    pub fn matches(&self, method: &str, path: &str) -> bool {
        if method != self.method {
            return false;
        }
        if let Some(prefix) = self.path.strip_suffix("/**") {
            return path.strip_prefix(prefix).is_some_and(|rest| rest.starts_with('/'));
        }
        path == self.path
    }
}

pub(crate) fn path_without_query(path: &str) -> &str {
    path.split_once('?').map_or(path, |(path, _)| path)
}

#[cfg(test)]
mod tests {
    use config::{Config, File, FileFormat};
    use serde_json::json;

    use super::*;

    fn allowlist() -> PathAllowlist {
        serde_json::from_value(json!([
            { "path": "/quote/v2", "method": "POST" },
            { "path": "/chains", "method": "GET" },
            { "path": "/api/v2/address/**", "method": "GET" }
        ]))
        .unwrap()
    }

    #[test]
    fn test_allows_exact_path_and_method() {
        let allowlist = allowlist();

        assert!(allowlist.allows("POST", "/quote/v2"));
        assert!(allowlist.allows("GET", "/chains"));
        assert!(!allowlist.allows("GET", "/quote/v2"));
        assert!(!allowlist.allows("POST", "/chains"));
        assert!(!allowlist.allows("GET", "/execute"));
    }

    #[test]
    fn test_denies_path_prefix_without_wildcard() {
        let allowlist = allowlist();

        assert!(!allowlist.allows("GET", "/chains/"));
        assert!(!allowlist.allows("GET", "/chains/1"));
        assert!(!allowlist.allows("GET", "//chains"));
    }

    #[test]
    fn test_allows_wildcard_suffix() {
        let allowlist = allowlist();

        assert!(allowlist.allows("GET", "/api/v2/address/bc1qtest"));
        assert!(allowlist.allows("GET", "/api/v2/address/bc1qtest/utxo"));
        assert!(!allowlist.allows("GET", "/api/v2/address"));
        assert!(!allowlist.allows("GET", "/api/v2/addressbook/1"));
    }

    #[test]
    fn test_empty_allowlist_is_unrestricted() {
        let allowlist = PathAllowlist::default();

        assert!(allowlist.is_empty());
        assert!(allowlist.allows("DELETE", "/anything"));
    }

    #[test]
    fn test_deserializes_from_yaml() {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(default)]
            allowlist: PathAllowlist,
        }

        let input = "allowlist:\n  - path: /chains\n    method: GET\n";
        let wrapper = Config::builder()
            .add_source(File::from_str(input, FileFormat::Yaml))
            .build()
            .unwrap()
            .try_deserialize::<Wrapper>()
            .unwrap();

        assert!(wrapper.allowlist.allows("GET", "/chains"));
        assert!(!wrapper.allowlist.allows("GET", "/other"));
    }
}
