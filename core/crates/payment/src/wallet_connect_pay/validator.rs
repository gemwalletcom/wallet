use gem_wallet_connect::WalletConnectAction;
use primitives::{ChainType, SignDigestType, ValueAccess, WCEthereumTransaction, WalletConnectCAIP2};
use serde_json::Value;

use crate::wallet_connect_pay::error::WalletConnectPayError;

pub fn validate_signer(account: &str, params: &Value, action: &WalletConnectAction) -> Result<(), WalletConnectPayError> {
    let chain_address =
        WalletConnectCAIP2::parse_account(account.to_string()).ok_or_else(|| WalletConnectPayError::InvalidRequest(format!("Invalid option account: {account}")))?;
    if chain_address.chain.chain_type() != ChainType::Ethereum {
        return Ok(());
    }

    match action {
        WalletConnectAction::SendTransaction { data, .. } | WalletConnectAction::SignTransaction { data, .. } => {
            let transaction: WCEthereumTransaction =
                serde_json::from_str(data).map_err(|error| WalletConnectPayError::InvalidRequest(format!("Invalid transaction payload: {error}")))?;
            validate_address_match(&chain_address.address, &transaction.from)
        }
        WalletConnectAction::SignMessage {
            sign_type: SignDigestType::Eip712,
            ..
        } => {
            let signer = params.at(0).and_then(Value::string).map_err(WalletConnectPayError::InvalidRequest)?;
            validate_address_match(&chain_address.address, signer)
        }
        WalletConnectAction::SignMessage {
            sign_type:
                SignDigestType::Eip191 | SignDigestType::Base58 | SignDigestType::SuiPersonal | SignDigestType::Siwe | SignDigestType::TonPersonal | SignDigestType::TronPersonal,
            ..
        }
        | WalletConnectAction::SignAllTransactions { .. }
        | WalletConnectAction::ChainOperation { .. }
        | WalletConnectAction::GetAccounts { .. }
        | WalletConnectAction::Unsupported { .. } => Ok(()),
    }
}

fn validate_address_match(expected: &str, actual: &str) -> Result<(), WalletConnectPayError> {
    if actual.to_lowercase() == expected.to_lowercase() {
        return Ok(());
    }
    Err(WalletConnectPayError::InvalidRequest(format!("Signer address mismatch: expected {expected}, got {actual}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::testkit::{TEST_ACCOUNT_ETHEREUM, TEST_ACCOUNT_SOLANA};
    use primitives::SignableTransactionType;
    use primitives::{Chain, TransferDataOutputType};

    fn send_transaction(from: &str) -> WalletConnectAction {
        WalletConnectAction::SendTransaction {
            chain: Chain::Ethereum,
            transaction_type: SignableTransactionType::Ethereum,
            data: serde_json::json!({"from": from, "to": "0x00"}).to_string(),
        }
    }

    fn sign_message(sign_type: SignDigestType) -> WalletConnectAction {
        WalletConnectAction::SignMessage {
            chain: Chain::Ethereum,
            sign_type,
            data: "{}".to_string(),
        }
    }

    #[test]
    fn test_validate_signer_transaction() {
        let address = "0x1085c5f70F7F7591D97da281A64688385455c2bD";

        assert!(validate_signer(TEST_ACCOUNT_ETHEREUM, &Value::Null, &send_transaction(address)).is_ok());
        assert!(validate_signer(TEST_ACCOUNT_ETHEREUM, &Value::Null, &send_transaction(&address.to_lowercase())).is_ok());
        assert!(validate_signer(TEST_ACCOUNT_ETHEREUM, &Value::Null, &send_transaction("0xdead")).is_err());
    }

    #[test]
    fn test_validate_signer_typed_data() {
        let signer = serde_json::json!(["0x1085c5f70f7f7591d97da281a64688385455c2bd", {}]);
        assert!(validate_signer(TEST_ACCOUNT_ETHEREUM, &signer, &sign_message(SignDigestType::Eip712)).is_ok());

        let other_signer = serde_json::json!(["0xdead", {}]);
        assert!(validate_signer(TEST_ACCOUNT_ETHEREUM, &other_signer, &sign_message(SignDigestType::Eip712)).is_err());
        assert!(validate_signer(TEST_ACCOUNT_ETHEREUM, &Value::Null, &sign_message(SignDigestType::Eip712)).is_err());
        assert!(validate_signer(TEST_ACCOUNT_ETHEREUM, &other_signer, &sign_message(SignDigestType::Eip191)).is_ok());
    }

    #[test]
    fn test_validate_signer_skips_non_ethereum() {
        let action = WalletConnectAction::SignTransaction {
            chain: Chain::Solana,
            transaction_type: SignableTransactionType::Solana {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: "base64".to_string(),
        };

        assert!(validate_signer(TEST_ACCOUNT_SOLANA, &Value::Null, &action).is_ok());
        assert!(validate_signer("not-an-account", &Value::Null, &action).is_err());
    }
}
