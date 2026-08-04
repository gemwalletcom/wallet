use crate::actions::WalletConnectAction;
use primitives::SignDigestType;
use primitives::{Chain, TransferDataOutputType, ValueAccess, WalletConnectionMethods};
use primitives::{SignableTransaction, SignableTransactionType, SuiTransactionData};
use serde_json::Value;

pub struct SuiRequestHandler;

impl SuiRequestHandler {
    pub fn parse_request(method: WalletConnectionMethods, chain: Chain, params: Value, domain: &str) -> Result<WalletConnectAction, String> {
        match method {
            WalletConnectionMethods::SuiGetAccounts => Ok(WalletConnectAction::GetAccounts { chain }),
            WalletConnectionMethods::SuiSignPersonalMessage => Self::parse_sign_message(chain, params, domain),
            WalletConnectionMethods::SuiSignTransaction => Self::parse_sign_transaction(chain, params),
            WalletConnectionMethods::SuiSignAndExecuteTransaction => Self::parse_send_transaction(chain, params),
            _ => Err("Method not supported".to_string()),
        }
    }

    pub fn parse_sign_message(_chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let message = params.get_value("message")?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Sui,
            sign_type: SignDigestType::SuiPersonal,
            data: message,
        })
    }

    pub fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?.string()?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Sui,
            transaction_type: SignableTransactionType::Sui {
                output_type: TransferDataOutputType::Signature,
            },
            data: params.to_string(),
        })
    }

    pub fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("transaction")?.string()?;

        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Sui,
            transaction_type: SignableTransactionType::Sui {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: params.to_string(),
        })
    }

    pub fn decode_send_transaction(data: String, output_type: TransferDataOutputType) -> Result<SignableTransaction, String> {
        let json: Value = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        let transaction = json
            .get("transaction")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "Missing transaction field".to_string())?
            .to_string();
        let wallet_address = json
            .get("account")
            .or_else(|| json.get("address"))
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        Ok(SignableTransaction::Sui {
            data: SuiTransactionData { transaction, wallet_address },
            output_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sign_message() {
        let params = serde_json::from_str(r#"{"message":"Hello Sui"}"#).unwrap();
        assert_eq!(
            SuiRequestHandler::parse_sign_message(Chain::Sui, params, "example.com").unwrap(),
            WalletConnectAction::SignMessage {
                chain: Chain::Sui,
                sign_type: SignDigestType::SuiPersonal,
                data: "Hello Sui".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sign_transaction() {
        let params: Value = serde_json::from_str(r#"{"address":"0xfa92fe9555eeb34d3d922dae643483cbd18bd607bf900a1df5e82dc22804698e","transaction":"AAACAAhkAAA"}"#).unwrap();
        let expected_data = params.to_string();
        assert_eq!(
            SuiRequestHandler::parse_sign_transaction(Chain::Sui, params).unwrap(),
            WalletConnectAction::SignTransaction {
                chain: Chain::Sui,
                transaction_type: SignableTransactionType::Sui {
                    output_type: TransferDataOutputType::Signature,
                },
                data: expected_data,
            }
        );
    }

    #[test]
    fn test_parse_send_transaction() {
        let params: Value = serde_json::from_str(r#"{"address":"0xfa92fe9555eeb34d3d922dae643483cbd18bd607bf900a1df5e82dc22804698e","transaction":"AAACAAhkAAA"}"#).unwrap();
        let expected_data = params.to_string();
        assert_eq!(
            SuiRequestHandler::parse_send_transaction(Chain::Sui, params).unwrap(),
            WalletConnectAction::SendTransaction {
                chain: Chain::Sui,
                transaction_type: SignableTransactionType::Sui {
                    output_type: TransferDataOutputType::EncodedTransaction,
                },
                data: expected_data,
            }
        );
    }
}
