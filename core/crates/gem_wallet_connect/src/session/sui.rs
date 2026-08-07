use std::collections::HashMap;

use primitives::Account;
use serde_json::Value;

use crate::accounts::{SUI_GET_ACCOUNTS_PROPERTY, map_sui_get_accounts};

pub(super) fn sui_session_properties(properties: &mut HashMap<String, String>, accounts: &[Account]) {
    let sui_accounts = map_sui_get_accounts(accounts);
    if !sui_accounts.is_empty() {
        properties.insert(SUI_GET_ACCOUNTS_PROPERTY.to_string(), Value::Array(sui_accounts).to_string());
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::*;
    use crate::{
        session::config_session_properties,
        testkit::{TEST_SUI_ADDRESS, TEST_SUI_PUBLIC_KEY_BASE64, mock_sui_account},
    };

    #[test]
    fn test_config_session_properties_sui() {
        let result = config_session_properties(HashMap::new(), &[Chain::Sui], &[mock_sui_account()]);
        assert_eq!(
            result.get(SUI_GET_ACCOUNTS_PROPERTY).unwrap(),
            &format!(r#"[{{"pubkey":"{TEST_SUI_PUBLIC_KEY_BASE64}","address":"{TEST_SUI_ADDRESS}"}}]"#)
        );

        let account = Account {
            extended_public_key: None,
            ..mock_sui_account()
        };
        let result = config_session_properties(HashMap::new(), &[Chain::Sui], &[account]);
        assert_eq!(result.get(SUI_GET_ACCOUNTS_PROPERTY), None);

        let result = config_session_properties(HashMap::new(), &[Chain::Ethereum], &[mock_sui_account()]);
        assert_eq!(result.get(SUI_GET_ACCOUNTS_PROPERTY), None);
    }
}
