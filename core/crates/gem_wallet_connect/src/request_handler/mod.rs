mod ethereum;
mod solana;
mod sui;
mod ton;
mod tron;

use crate::actions::{WCSolanaTransactionData, WCSuiTransactionData, WalletConnectAction, WalletConnectChainOperation, WalletConnectTransaction, WalletConnectTransactionType};
use ethereum::EthereumRequestHandler;
use primitives::{Chain, ChainType, ValueAccess, WCEthereumTransaction, WalletConnectCAIP2, WalletConnectRequest, WalletConnectionMethods, hex};
use serde_json::Value;
use solana::SolanaRequestHandler;
use sui::SuiRequestHandler;
use ton::TonRequestHandler;
use tron::TronRequestHandler;

pub struct WalletConnectRequestHandler;

impl WalletConnectRequestHandler {
    pub fn parse_request(request: WalletConnectRequest) -> Result<WalletConnectAction, String> {
        let WalletConnectRequest {
            method, params, chain_id, domain, ..
        } = request;
        let method = match serde_json::from_value::<WalletConnectionMethods>(serde_json::Value::String(method.clone())) {
            Ok(m) => m,
            Err(_) => return Ok(WalletConnectAction::Unsupported { method }),
        };
        let params = serde_json::from_str::<Value>(&params).map_err(|e| format!("Failed to parse params: {}", e))?;
        let params = match params {
            Value::String(raw_json) => serde_json::from_str::<Value>(&raw_json).unwrap_or(Value::String(raw_json)),
            value => value,
        };

        match method {
            method @ (WalletConnectionMethods::PersonalSign
            | WalletConnectionMethods::EthSignTypedData
            | WalletConnectionMethods::EthSignTypedDataV4
            | WalletConnectionMethods::EthSignTransaction
            | WalletConnectionMethods::EthSendTransaction) => {
                EthereumRequestHandler::parse_request(method, Self::parse_required_chain(chain_id, ChainType::Ethereum)?, params, &domain)
            }
            WalletConnectionMethods::EthSendRawTransaction => Err("Method not supported".to_string()),
            WalletConnectionMethods::EthChainId => Ok(WalletConnectAction::ChainOperation {
                operation: WalletConnectChainOperation::GetChainId,
            }),
            WalletConnectionMethods::WalletAddEthereumChain => Ok(WalletConnectAction::ChainOperation {
                operation: WalletConnectChainOperation::AddChain,
            }),
            WalletConnectionMethods::WalletSwitchEthereumChain => {
                let chain = Self::parse_switch_chain_id(&params)?;
                Ok(WalletConnectAction::ChainOperation {
                    operation: WalletConnectChainOperation::SwitchChain { chain },
                })
            }
            method @ (WalletConnectionMethods::SolanaSignMessage
            | WalletConnectionMethods::SolanaSignTransaction
            | WalletConnectionMethods::SolanaSignAndSendTransaction
            | WalletConnectionMethods::SolanaSignAllTransactions) => {
                SolanaRequestHandler::parse_request(method, Self::parse_required_chain(chain_id, ChainType::Solana)?, params, &domain)
            }
            method @ (WalletConnectionMethods::SuiGetAccounts
            | WalletConnectionMethods::SuiSignPersonalMessage
            | WalletConnectionMethods::SuiSignTransaction
            | WalletConnectionMethods::SuiSignAndExecuteTransaction) => {
                SuiRequestHandler::parse_request(method, Self::parse_required_chain(chain_id, ChainType::Sui)?, params, &domain)
            }
            method @ (WalletConnectionMethods::TonSignData | WalletConnectionMethods::TonSendMessage) => {
                TonRequestHandler::parse_request(method, Self::parse_required_chain(chain_id, ChainType::Ton)?, params, &domain)
            }
            method @ (WalletConnectionMethods::TronSignMessage | WalletConnectionMethods::TronSignTransaction | WalletConnectionMethods::TronSendTransaction) => {
                TronRequestHandler::parse_request(method, Self::parse_required_chain(chain_id, ChainType::Tron)?, params, &domain)
            }
        }
    }

    pub fn decode_send_transaction(transaction_type: WalletConnectTransactionType, data: String) -> Result<WalletConnectTransaction, String> {
        match transaction_type {
            WalletConnectTransactionType::Ethereum => {
                let tx: WCEthereumTransaction = serde_json::from_str(&data).map_err(|e| e.to_string())?;
                Ok(WalletConnectTransaction::Ethereum { data: tx.into() })
            }
            WalletConnectTransactionType::Solana { output_type } => {
                let json: serde_json::Value = serde_json::from_str(&data).map_err(|e| e.to_string())?;
                let transaction = json
                    .get("transaction")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing transaction field".to_string())?
                    .to_string();
                Ok(WalletConnectTransaction::Solana {
                    data: WCSolanaTransactionData { transaction },
                    output_type,
                })
            }
            WalletConnectTransactionType::Sui { output_type } => {
                let json: serde_json::Value = serde_json::from_str(&data).map_err(|e| e.to_string())?;
                let transaction = json
                    .get("transaction")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing transaction field".to_string())?
                    .to_string();
                let wallet_address = json.get("account").or_else(|| json.get("address")).and_then(|v| v.as_str()).unwrap_or_default().to_string();
                Ok(WalletConnectTransaction::Sui {
                    data: WCSuiTransactionData { transaction, wallet_address },
                    output_type,
                })
            }
            WalletConnectTransactionType::Ton { output_type } => Ok(WalletConnectTransaction::Ton { data, output_type }),
            WalletConnectTransactionType::Tron { output_type } => Ok(WalletConnectTransaction::Tron { data, output_type }),
        }
    }

    fn parse_required_chain(chain_id: Option<String>, required: ChainType) -> Result<Chain, String> {
        let chain = WalletConnectCAIP2::get_chain_from_id(chain_id)?;
        if chain.chain_type() != required {
            if required == ChainType::Ethereum {
                return Err(format!("WalletConnect method requires an EVM chain, got {chain}"));
            }
            return Err(format!("WalletConnect method requires {}, got {chain}", required.as_ref()));
        }
        Ok(chain)
    }

    fn parse_switch_chain_id(params: &Value) -> Result<Chain, String> {
        let chain_id_text = params.at(0)?.get_value("chainId")?.string()?;
        let chain_id = hex::parse_u64_from_hex_or_decimal(chain_id_text).map_err(|error| error.to_string())?;
        Chain::from_chain_id(chain_id).ok_or_else(|| "Unsupported chain".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sign_type::SignDigestType;
    use gem_evm::testkit::eip712_mock::mock_eip712_json;
    use primitives::TransferDataOutputType;

    #[test]
    fn test_unsupported_method() {
        let request = WalletConnectRequest::mock("unknown_method", "{}", None);
        let action = WalletConnectRequestHandler::parse_request(request).unwrap();
        assert_eq!(
            action,
            WalletConnectAction::Unsupported {
                method: "unknown_method".to_string()
            }
        );
    }

    #[test]
    fn test_bitcoin_methods_are_unsupported() {
        let request = WalletConnectRequest::mock("signMessage", "{}", Some("bip122:000000000019d6689c085ae165831e93"));
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap(),
            WalletConnectAction::Unsupported {
                method: "signMessage".to_string()
            }
        );

        let request = WalletConnectRequest::mock("sendTransfer", "{}", Some("bip122:000000000019d6689c085ae165831e93"));
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap(),
            WalletConnectAction::Unsupported {
                method: "sendTransfer".to_string()
            }
        );
    }

    #[test]
    fn test_sui_get_accounts() {
        let request = WalletConnectRequest::mock("sui_getAccounts", "{}", Some("sui:mainnet"));
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap(),
            WalletConnectAction::GetAccounts { chain: Chain::Sui }
        );
    }

    #[test]
    fn test_sui_get_accounts_rejects_wrong_namespace() {
        let request = WalletConnectRequest::mock("sui_getAccounts", "{}", Some("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"));
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap_err(),
            "WalletConnect method requires sui, got solana"
        );
    }

    #[test]
    fn test_solana_method_rejects_wrong_namespace() {
        let request = WalletConnectRequest::mock("solana_signMessage", r#"["hello"]"#, Some("sui:mainnet"));
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap_err(),
            "WalletConnect method requires solana, got sui"
        );
    }

    #[test]
    fn test_evm_method_rejects_non_evm_namespace() {
        let request = WalletConnectRequest::mock("personal_sign", r#"["0x48656c6c6f"]"#, Some("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"));
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap_err(),
            "WalletConnect method requires an EVM chain, got solana"
        );
    }

    #[test]
    fn test_chain_operation_add_chain() {
        let request = WalletConnectRequest::mock("wallet_addEthereumChain", "{}", None);
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap(),
            WalletConnectAction::ChainOperation {
                operation: WalletConnectChainOperation::AddChain,
            }
        );
    }

    #[test]
    fn test_chain_operation_switch_chain() {
        let params = r#"[{"chainId":"0x1"}]"#;
        let request = WalletConnectRequest::mock("wallet_switchEthereumChain", params, None);
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap(),
            WalletConnectAction::ChainOperation {
                operation: WalletConnectChainOperation::SwitchChain { chain: Chain::Ethereum },
            }
        );
    }

    #[test]
    fn test_chain_operation_switch_chain_missing_chain_id() {
        let params = r#"[{}]"#;
        let request = WalletConnectRequest::mock("wallet_switchEthereumChain", params, None);
        assert!(WalletConnectRequestHandler::parse_request(request).is_err());
    }

    #[test]
    fn test_parse_request_eip712_chain_match() {
        let eip712_json = mock_eip712_json(1);
        let params = serde_json::to_string(&serde_json::json!(["0x123", eip712_json])).unwrap();
        let request = WalletConnectRequest::mock("eth_signTypedData_v4", &params, Some("eip155:1"));
        let result = WalletConnectRequestHandler::parse_request(request);
        assert!(result.is_ok());

        match result.unwrap() {
            WalletConnectAction::SignMessage { chain, sign_type, .. } => {
                assert_eq!(chain, Chain::Ethereum);
                assert_eq!(sign_type, SignDigestType::Eip712);
            }
            _ => panic!("Expected SignMessage action"),
        }
    }

    #[test]
    fn test_parse_request_eip712_chain_mismatch_rejects() {
        let eip712_json = mock_eip712_json(137);
        let params = serde_json::to_string(&serde_json::json!(["0x123", eip712_json])).unwrap();
        let request = WalletConnectRequest::mock("eth_signTypedData_v4", &params, Some("eip155:1"));
        let result = WalletConnectRequestHandler::parse_request(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_parse_request_eip712_cross_chain_attack() {
        let eip712_json = mock_eip712_json(137);
        let params = serde_json::to_string(&serde_json::json!(["0x123", eip712_json])).unwrap();

        let request = WalletConnectRequest::mock("eth_signTypedData_v4", &params, Some("eip155:1"));
        assert!(WalletConnectRequestHandler::parse_request(request).is_err());

        let request = WalletConnectRequest::mock("eth_signTypedData_v4", &params, Some("eip155:137"));
        assert!(WalletConnectRequestHandler::parse_request(request).is_ok());
    }

    #[test]
    fn test_solana_sign_all_transactions_roundtrip() {
        let params = include_str!("../../testdata/solana_sign_all_transactions.json");
        let request = WalletConnectRequest::mock("solana_signAllTransactions", params, Some("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"));
        let action = WalletConnectRequestHandler::parse_request(request).unwrap();
        match &action {
            WalletConnectAction::SignAllTransactions {
                chain,
                transaction_type,
                transactions,
            } => {
                assert_eq!(*chain, Chain::Solana);
                assert_eq!(transactions.len(), 1);
                let decoded = WalletConnectRequestHandler::decode_send_transaction(transaction_type.clone(), transactions[0].clone()).unwrap();
                match decoded {
                    WalletConnectTransaction::Solana { data, output_type } => {
                        assert!(data.transaction.starts_with("AQAAAAAAAAA"));
                        assert_eq!(output_type, TransferDataOutputType::EncodedTransaction);
                    }
                    _ => panic!("Expected Solana transaction"),
                }
            }
            _ => panic!("Expected SignAllTransactions action"),
        }
    }

    #[test]
    fn test_decode_ethereum_transaction_accepts_numeric_and_hex_string_chain_id() {
        for (chain_id, expected) in [(r#"4663"#, 4663), (r#""0x1237""#, 4663)] {
            let decoded = WalletConnectRequestHandler::decode_send_transaction(
                WalletConnectTransactionType::Ethereum,
                format!(r#"{{"chainId":{chain_id},"from":"0xsender","to":"0xrouter","data":"0x1234","value":"0x0"}}"#),
            )
            .unwrap();

            match decoded {
                WalletConnectTransaction::Ethereum { data } => {
                    assert_eq!(data.chain_id, Some(expected));
                    assert_eq!(data.data, Some("0x1234".to_string()));
                }
                _ => panic!("Expected Ethereum transaction"),
            }
        }
    }

    #[test]
    fn test_parse_request_eth_sign_typed_data_v3_chain_mismatch() {
        let eip712_json = mock_eip712_json(56);
        let params = serde_json::to_string(&serde_json::json!(["0x123", eip712_json])).unwrap();
        let request = WalletConnectRequest::mock("eth_signTypedData", &params, Some("eip155:1"));
        let result = WalletConnectRequestHandler::parse_request(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }
}
