use crate::actions::{WalletConnectAction, WalletConnectTransaction, WalletConnectTransactionType};
use crate::sign_type::SignDigestType;
use gem_evm::{eip712::validate_eip712_chain_id, transaction::decode_transaction_kind};
use primitives::{Chain, ValueAccess, WCEthereumTransaction, WalletConnectionMethods};
use serde_json::Value;

pub struct EthereumRequestHandler;

impl EthereumRequestHandler {
    pub fn parse_request(method: WalletConnectionMethods, chain: Chain, params: Value, domain: &str) -> Result<WalletConnectAction, String> {
        match method {
            WalletConnectionMethods::PersonalSign => Self::parse_sign_message(chain, params, domain),
            WalletConnectionMethods::EthSignTypedData | WalletConnectionMethods::EthSignTypedDataV4 => Self::parse_sign_typed_data(chain, params),
            WalletConnectionMethods::EthSignTransaction => Self::parse_sign_transaction(chain, params),
            WalletConnectionMethods::EthSendTransaction => Self::parse_send_transaction(chain, params),
            _ => Err("Method not supported".to_string()),
        }
    }

    pub fn parse_sign_message(chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let data = params.at(0)?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::Eip191,
            data,
        })
    }

    pub fn parse_sign_typed_data(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let typed_data = params.at(1)?;
        let data = if let Some(s) = typed_data.as_str() {
            s.to_string()
        } else {
            serde_json::to_string(typed_data).map_err(|e| format!("Failed to serialize typed data: {}", e))?
        };

        let expected_chain_id = Self::expected_chain_id(chain)?;
        validate_eip712_chain_id(&data, expected_chain_id)?;

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::Eip712,
            data,
        })
    }

    pub fn parse_sign_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let data = Self::parse_transaction_data(chain, params)?;

        Ok(WalletConnectAction::SignTransaction {
            chain,
            transaction_type: WalletConnectTransactionType::Ethereum,
            data,
        })
    }

    pub fn parse_send_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let data = Self::parse_transaction_data(chain, params)?;

        Ok(WalletConnectAction::SendTransaction {
            chain,
            transaction_type: WalletConnectTransactionType::Ethereum,
            data,
        })
    }

    pub fn decode_send_transaction(data: String) -> Result<WalletConnectTransaction, String> {
        let transaction: WCEthereumTransaction = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        let kind = decode_transaction_kind(&transaction.to, transaction.data.as_deref())?;
        Ok(WalletConnectTransaction::Ethereum { data: transaction.into(), kind })
    }

    fn parse_transaction_data(chain: Chain, params: Value) -> Result<String, String> {
        let transaction = params.at(0)?.clone();
        transaction.as_object().ok_or_else(|| "Expected Ethereum transaction object".to_string())?;
        let parsed: WCEthereumTransaction = serde_json::from_value(transaction.clone()).map_err(|error| error.to_string())?;
        Self::validate_transaction_chain_id(chain, parsed.chain_id)?;
        serde_json::to_string(&transaction).map_err(|e| format!("Failed to serialize transaction: {}", e))
    }

    fn validate_transaction_chain_id(chain: Chain, chain_id: Option<u64>) -> Result<(), String> {
        let Some(actual) = chain_id else {
            return Ok(());
        };
        let expected = Self::expected_chain_id(chain)?;
        if actual != expected {
            return Err(format!("Transaction chainId mismatch: expected {expected}, got {actual}"));
        }
        Ok(())
    }

    fn expected_chain_id(chain: Chain) -> Result<u64, String> {
        chain.network_id_value().ok_or_else(|| format!("Chain {} does not have a numeric network ID", chain))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::testkit::eip712_mock::mock_eip712_json;

    fn eip712_params(chain_id: u64) -> Value {
        let eip712_json = mock_eip712_json(chain_id);
        serde_json::json!(["0x123", eip712_json])
    }

    fn eip712_params_object(chain_id: u64) -> Value {
        let eip712_value: Value = serde_json::from_str(&mock_eip712_json(chain_id)).unwrap();
        serde_json::json!(["0x123", eip712_value])
    }

    fn eip712_params_without_domain_chain_id() -> Value {
        serde_json::json!(["0x123", include_str!("../../../gem_evm/testdata/ens_upload_avatar.json")])
    }

    #[test]
    fn test_parse_personal_sign() {
        let params = serde_json::from_str(r#"["0x48656c6c6f"]"#).unwrap();
        let action = EthereumRequestHandler::parse_sign_message(Chain::Ethereum, params, "example.com").unwrap();
        assert_eq!(
            action,
            WalletConnectAction::SignMessage {
                chain: Chain::Ethereum,
                sign_type: SignDigestType::Eip191,
                data: "0x48656c6c6f".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sign_typed_data_matching_chain() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params(1));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_chain_id_mismatch_polygon_on_ethereum() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params(137));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_parse_sign_typed_data_chain_id_mismatch_ethereum_on_polygon() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Polygon, eip712_params(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_parse_sign_typed_data_polygon_matching() {
        assert!(EthereumRequestHandler::parse_sign_typed_data(Chain::Polygon, eip712_params(137)).is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_bsc_matching() {
        assert!(EthereumRequestHandler::parse_sign_typed_data(Chain::SmartChain, eip712_params(56)).is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_arbitrum_matching() {
        assert!(EthereumRequestHandler::parse_sign_typed_data(Chain::Arbitrum, eip712_params(42161)).is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_object_params() {
        assert!(EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params_object(1)).is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_object_params_chain_mismatch() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params_object(137));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_parse_sign_typed_data_without_domain_chain_id() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params_without_domain_chain_id());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_eip712_domain_chain_id_without_schema_rejects() {
        let params = serde_json::json!(["0x123", include_str!("../../../gem_evm/testdata/eip712_domain_chain_id_without_schema_field.json")]);
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, params);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("chainId"));
    }

    #[test]
    fn test_parse_send_transaction() {
        let params = serde_json::from_str(r#"[{"from":"0xsender","to":"0x123","value":"0x0","chainId":"0x1"}]"#).unwrap();
        let action = EthereumRequestHandler::parse_send_transaction(Chain::Ethereum, params).unwrap();
        match action {
            WalletConnectAction::SendTransaction { chain, transaction_type, data } => {
                let transaction: Value = serde_json::from_str(&data).unwrap();
                assert_eq!(chain, Chain::Ethereum);
                assert_eq!(transaction_type, WalletConnectTransactionType::Ethereum);
                assert_eq!(transaction["from"], "0xsender");
                assert_eq!(transaction["chainId"], "0x1");
            }
            _ => panic!("Expected SendTransaction action"),
        }
    }

    #[test]
    fn test_parse_send_transaction_chain_id_mismatch_rejects() {
        let params = serde_json::from_str(r#"[{"from":"0xabc","to":"0x123","value":"0x0","chainId":"0x89"}]"#).unwrap();
        let result = EthereumRequestHandler::parse_send_transaction(Chain::Ethereum, params);
        assert_eq!(result.unwrap_err(), "Transaction chainId mismatch: expected 1, got 137");
    }
}
