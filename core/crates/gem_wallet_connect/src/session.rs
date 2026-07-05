use std::collections::HashMap;

use primitives::Chain;
use serde_json::Value;

use crate::accounts::{SUI_GET_ACCOUNTS_PROPERTY, map_sui_get_accounts};
use primitives::Account;

const TRON_METHOD_VERSION_KEY: &str = "tron_method_version";
const TRON_METHOD_VERSION_VALUE: &str = "v1";

pub fn config_session_properties(mut properties: HashMap<String, String>, chains: &[Chain], accounts: &[Account]) -> HashMap<String, String> {
    if chains.contains(&Chain::Tron) {
        properties = tron_session_properties(properties);
    }
    if chains.contains(&Chain::Sui) {
        properties = sui_session_properties(properties, accounts);
    }
    properties
}

fn tron_session_properties(mut properties: HashMap<String, String>) -> HashMap<String, String> {
    if !properties.contains_key(TRON_METHOD_VERSION_KEY) {
        properties.insert(TRON_METHOD_VERSION_KEY.to_string(), TRON_METHOD_VERSION_VALUE.to_string());
    }
    properties
}

fn sui_session_properties(mut properties: HashMap<String, String>, accounts: &[Account]) -> HashMap<String, String> {
    let sui_accounts = map_sui_get_accounts(accounts);
    if !sui_accounts.is_empty() {
        properties.insert(SUI_GET_ACCOUNTS_PROPERTY.to_string(), Value::Array(sui_accounts).to_string());
    }
    properties
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_SUI_ADDRESS, TEST_SUI_PUBLIC_KEY_BASE64, mock_sui_account};

    #[test]
    fn test_config_session_properties_adds_tron() {
        let result = config_session_properties(HashMap::new(), &[Chain::Tron], &[]);
        assert_eq!(result.get("tron_method_version").unwrap(), "v1");
    }

    #[test]
    fn test_config_session_properties_preserves_existing() {
        let mut props = HashMap::new();
        props.insert("tron_method_version".to_string(), "v2".to_string());
        let result = config_session_properties(props, &[Chain::Tron], &[]);
        assert_eq!(result.get("tron_method_version").unwrap(), "v2");
    }

    #[test]
    fn test_config_session_properties_no_tron() {
        let result = config_session_properties(HashMap::new(), &[Chain::Ethereum], &[]);
        assert!(!result.contains_key("tron_method_version"));
    }

    #[test]
    fn test_config_session_properties_adds_sui_get_accounts() {
        let result = config_session_properties(HashMap::new(), &[Chain::Sui], &[mock_sui_account()]);
        assert_eq!(
            result.get("sui_getAccounts").unwrap(),
            &format!(r#"[{{"pubkey":"{TEST_SUI_PUBLIC_KEY_BASE64}","address":"{TEST_SUI_ADDRESS}"}}]"#)
        );
    }

    #[test]
    fn test_config_session_properties_skips_sui_without_public_key() {
        let account = Account {
            extended_public_key: None,
            ..mock_sui_account()
        };
        let result = config_session_properties(HashMap::new(), &[Chain::Sui], &[account]);
        assert!(!result.contains_key("sui_getAccounts"));

        let result = config_session_properties(HashMap::new(), &[Chain::Ethereum], &[mock_sui_account()]);
        assert!(!result.contains_key("sui_getAccounts"));
    }
}
