use base64::{Engine, engine::general_purpose::STANDARD};
use primitives::{Account, Chain, hex};
use serde_json::Value;

pub const SUI_GET_ACCOUNTS_PROPERTY: &str = "sui_getAccounts";
const SUI_ED25519_SCHEME_FLAG: u8 = 0x00;
const ED25519_PUBLIC_KEY_LENGTH: usize = 32;

/// sui_getAccounts payload: pubkey is base64 of the ed25519 scheme flag + 32-byte public key.
pub fn map_sui_get_accounts(accounts: &[Account]) -> Vec<Value> {
    accounts.iter().filter(|account| account.chain == Chain::Sui).filter_map(sui_account_value).collect()
}

fn sui_account_value(account: &Account) -> Option<Value> {
    let public_key = hex::decode_hex(account.extended_public_key.as_deref()?).ok()?;
    if public_key.len() != ED25519_PUBLIC_KEY_LENGTH {
        return None;
    }
    let flagged_key = [&[SUI_ED25519_SCHEME_FLAG][..], &public_key].concat();
    Some(serde_json::json!({ "pubkey": STANDARD.encode(flagged_key), "address": account.address }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_SUI_ADDRESS, TEST_SUI_PUBLIC_KEY_BASE64, mock_sui_account};
    use primitives::testkit::signer_mock::TEST_EVM_SENDER;

    #[test]
    fn test_map_sui_get_accounts() {
        let accounts = vec![
            mock_sui_account(),
            Account {
                chain: Chain::Ethereum,
                address: TEST_EVM_SENDER.to_string(),
                extended_public_key: Some("11".repeat(32)),
                ..mock_sui_account()
            },
            Account {
                extended_public_key: None,
                ..mock_sui_account()
            },
            Account {
                extended_public_key: Some("0xdeadbeef".to_string()),
                ..mock_sui_account()
            },
        ];

        assert_eq!(
            map_sui_get_accounts(&accounts),
            vec![serde_json::json!({ "pubkey": TEST_SUI_PUBLIC_KEY_BASE64, "address": TEST_SUI_ADDRESS })]
        );
        assert_eq!(map_sui_get_accounts(&[]), Vec::<Value>::new());
    }
}
