use gem_encoding::encode_base64;
use primitives::{Account, Chain, hex};
use serde_json::Value;

pub const SUI_GET_ACCOUNTS_PROPERTY: &str = "sui_getAccounts";
const SUI_ED25519_SCHEME_FLAG: u8 = 0x00;
const ED25519_PUBLIC_KEY_LENGTH: usize = 32;

pub const BITCOIN_ACCOUNT_ADDRESSES_PROPERTY: &str = "bip122_getAccountAddresses";
const BITCOIN_PAYMENT_INTENTION: &str = "payment";

pub fn map_bitcoin_account_addresses(accounts: &[Account]) -> Vec<Value> {
    accounts
        .iter()
        .filter(|account| account.chain == Chain::Bitcoin)
        .map(|account| serde_json::json!({ "address": account.address, "path": account.derivation_path, "intention": BITCOIN_PAYMENT_INTENTION }))
        .collect()
}

pub fn map_sui_get_accounts(accounts: &[Account]) -> Vec<Value> {
    accounts.iter().filter(|account| account.chain == Chain::Sui).filter_map(sui_account_value).collect()
}

fn sui_account_value(account: &Account) -> Option<Value> {
    let public_key = hex::decode_hex(account.extended_public_key.as_deref()?).ok()?;
    if public_key.len() != ED25519_PUBLIC_KEY_LENGTH {
        return None;
    }
    let flagged_key = [&[SUI_ED25519_SCHEME_FLAG][..], &public_key].concat();
    Some(serde_json::json!({ "pubkey": encode_base64(&flagged_key), "address": account.address }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_SUI_ADDRESS, TEST_SUI_PUBLIC_KEY_BASE64, mock_sui_account};
    use primitives::testkit::signer_mock::TEST_EVM_SENDER;

    #[test]
    fn test_map_bitcoin_account_addresses() {
        let bitcoin = Account {
            chain: Chain::Bitcoin,
            address: "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu".to_string(),
            derivation_path: "m/84'/0'/0'/0/0".to_string(),
            extended_public_key: None,
        };
        assert_eq!(
            map_bitcoin_account_addresses(&[bitcoin, mock_sui_account()]),
            vec![serde_json::json!({ "address": "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu", "path": "m/84'/0'/0'/0/0", "intention": "payment" })]
        );
    }

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

        assert_eq!(map_sui_get_accounts(&accounts), vec![serde_json::json!({ "pubkey": TEST_SUI_PUBLIC_KEY_BASE64, "address": TEST_SUI_ADDRESS })]);
        assert_eq!(map_sui_get_accounts(&[]), Vec::<Value>::new());
    }
}
