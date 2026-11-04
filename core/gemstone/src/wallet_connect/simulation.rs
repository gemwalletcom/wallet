use gem_tron::decode_wallet_connect_contract;
use gem_wallet_connect::{
    EvmTransactionKind, SignDigestType, SignMessageValidation, WCEthereumTransactionData, WalletConnectRequestHandler, WalletConnectTransaction, WalletConnectTransactionType, validate_send_transaction, validate_sign_message,
};
use primitives::{Chain, SimulationSeverity, SimulationWarning, SimulationWarningType, hex};

pub(crate) fn parse_eip712_message(data: &str) -> Option<gem_evm::eip712::EIP712Message> {
    serde_json::from_str(data).ok().and_then(|value| gem_evm::eip712::parse_eip712_json(&value).ok())
}

pub(crate) fn sign_message_validation_warnings(chain: Chain, sign_type: &SignDigestType, data: &str, session_domain: &str) -> Vec<SimulationWarning> {
    let input = SignMessageValidation { chain, sign_type, data, session_domain };

    validate_sign_message(&input).err().into_iter().map(SimulationWarning::validation_error).collect()
}

pub(crate) fn send_transaction_validation_warnings(transaction_type: &WalletConnectTransactionType, data: &str) -> Vec<SimulationWarning> {
    let invalid = validate_send_transaction(transaction_type, data).err().map(SimulationWarning::validation_error);
    invalid.into_iter().chain(tron_token_warning(transaction_type, data)).collect()
}

fn tron_token_warning(transaction_type: &WalletConnectTransactionType, data: &str) -> Option<SimulationWarning> {
    let WalletConnectTransactionType::Tron { .. } = transaction_type else {
        return None;
    };
    let token = decode_wallet_connect_contract(data).ok()??.token?;
    let message = format!("This call also sends {} base units of TRC-10 token {}", token.value, token.id);
    Some(SimulationWarning::new(SimulationSeverity::Warning, SimulationWarningType::ValidationError, Some(message)))
}

pub(crate) fn decode_ethereum_transaction(data: &str) -> Result<(WCEthereumTransactionData, EvmTransactionKind), String> {
    let transaction = WalletConnectRequestHandler::decode_send_transaction(WalletConnectTransactionType::Ethereum, data.to_string())?;
    match transaction {
        WalletConnectTransaction::Ethereum { data, kind } => Ok((data, kind)),
        _ => Err("Invalid Ethereum transaction".to_string()),
    }
}

pub(crate) fn decode_ethereum_calldata(transaction: &WCEthereumTransactionData) -> Vec<u8> {
    transaction.data.as_deref().and_then(|calldata| hex::decode_hex(calldata).ok()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::TransferDataOutputType;

    fn tron_call(call_token_value: u64) -> String {
        serde_json::json!({
            "transaction": {
                "raw_data": {
                    "contract": [{
                        "type": "TriggerSmartContract",
                        "parameter": {
                            "value": {
                                "owner_address": "415cd0fb0ab3ce40f3051414c604b27756e69e43db",
                                "contract_address": "41a614f803b6fd780986a42c78ec9c7f77e6ded13c",
                                "data": "deadbeef",
                                "call_token_value": call_token_value,
                                "token_id": 1_002_000,
                            }
                        }
                    }],
                    "expiration": 1_539_331_479_000u64,
                    "ref_block_bytes": "7b3b",
                    "ref_block_hash": "b21ace8d6ac20e7e",
                    "timestamp": 1_539_295_479_000u64,
                }
            }
        })
        .to_string()
    }

    #[test]
    fn test_a_tron_call_spending_a_trc10_token_warns_without_stopping_confirm() {
        let transaction_type = WalletConnectTransactionType::Tron {
            output_type: TransferDataOutputType::EncodedTransaction,
        };

        assert_eq!(send_transaction_validation_warnings(&transaction_type, &tron_call(0)), vec![]);

        let warnings = send_transaction_validation_warnings(&transaction_type, &tron_call(1_500));
        assert_eq!(
            warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::ValidationError,
                Some("This call also sends 1500 base units of TRC-10 token 1002000".to_string())
            )]
        );
    }

    #[test]
    fn test_siws_origin_mismatch_is_a_critical_warning() {
        let data = bs58::encode(include_str!("../../../crates/gem_solana/testdata/siws_sign_in.txt")).into_string();
        assert_eq!(sign_message_validation_warnings(Chain::Solana, &SignDigestType::Base58, &data, "https://example.com"), vec![]);
        assert_eq!(
            sign_message_validation_warnings(Chain::Solana, &SignDigestType::Base58, &data, "https://other.xyz"),
            vec![SimulationWarning::validation_error("SIWS domain does not match session origin")]
        );
    }

    #[test]
    fn decode_ethereum_transaction_with_calldata_decodes_bytes() {
        let data = serde_json::json!({
            "from": "0xF977814e90dA44bFA03b6295A0616a897441aceC",
            "to": "0x1111111111111111111111111111111111111111",
            "data": "0xa9059cbb",
        })
        .to_string();

        let (transaction, _) = decode_ethereum_transaction(&data).unwrap();

        assert_eq!(decode_ethereum_calldata(&transaction), vec![0xa9, 0x05, 0x9c, 0xbb]);
    }

    #[test]
    fn decode_ethereum_transaction_without_calldata_decodes_empty_bytes() {
        let data = serde_json::json!({
            "from": "0xF977814e90dA44bFA03b6295A0616a897441aceC",
            "to": "0x1111111111111111111111111111111111111111",
            "value": "0x0",
        })
        .to_string();

        let (transaction, _) = decode_ethereum_transaction(&data).unwrap();

        assert!(decode_ethereum_calldata(&transaction).is_empty());
    }
}
