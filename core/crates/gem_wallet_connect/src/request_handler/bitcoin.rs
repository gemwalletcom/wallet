use crate::actions::{WCBitcoinTransferData, WalletConnectAction, WalletConnectTransaction, WalletConnectTransactionType};
use crate::sign_type::SignDigestType;
use gem_bitcoin::signer::psbt::BitcoinPsbtRequest;
use primitives::{Chain, ValueAccess, WalletConnectionMethods, hex};
use serde_json::Value;

const MEMO_MAX_BYTES: usize = 80;
const PROTOCOL_ECDSA: &str = "ecdsa";
const PROTOCOL_BIP322: &str = "bip322";

pub struct BitcoinRequestHandler;

impl BitcoinRequestHandler {
    pub fn parse_request(method: WalletConnectionMethods, params: Value) -> Result<WalletConnectAction, String> {
        match method {
            WalletConnectionMethods::BitcoinGetAccountAddresses => Ok(WalletConnectAction::GetAccounts { chain: Chain::Bitcoin }),
            WalletConnectionMethods::BitcoinSignMessage => Self::parse_sign_message(params),
            WalletConnectionMethods::BitcoinSignPsbt => Self::parse_sign_psbt(params),
            WalletConnectionMethods::BitcoinSendTransfer => Self::parse_send_transfer(params),
            _ => Err("Method not supported".to_string()),
        }
    }

    pub fn decode_psbt(data: String) -> Result<WalletConnectTransaction, String> {
        let request = BitcoinPsbtRequest::parse(&data).map_err(|error| error.to_string())?;
        request.decode_psbt().map_err(|error| error.to_string())?;
        Ok(WalletConnectTransaction::BitcoinPsbt { request, data })
    }

    pub fn decode_transfer(data: String) -> Result<WalletConnectTransaction, String> {
        let params: Value = serde_json::from_str(&data).map_err(|error| error.to_string())?;
        let amount = params.get_value("amount")?.string()?;
        let memo = match params.get("memo").and_then(Value::as_str).filter(|memo| !memo.is_empty()) {
            Some(memo) => Some(Self::decode_memo(memo)?).filter(|memo| !memo.trim().is_empty()),
            None => None,
        };
        Ok(WalletConnectTransaction::BitcoinTransfer {
            data: WCBitcoinTransferData {
                account: params.get_value("account")?.string()?.to_string(),
                recipient_address: params.get_value("recipientAddress")?.string()?.to_string(),
                amount: amount.parse().map_err(|_| format!("invalid amount {amount}"))?,
                memo,
            },
        })
    }

    pub fn validate_message_account(data: &str, address: &str) -> Result<(), String> {
        let params: Value = serde_json::from_str(data).map_err(|error| error.to_string())?;
        let account = params.get_value("account")?.string()?;
        let signer = params.get("address").and_then(Value::as_str).unwrap_or(account);
        if account != address || signer != address {
            return Err("Bitcoin message address does not match signing account".to_string());
        }
        Ok(())
    }

    pub fn decode_message(data: &str) -> Vec<u8> {
        serde_json::from_str::<Value>(data)
            .ok()
            .and_then(|params| params.get("message").and_then(Value::as_str).map(|message| message.as_bytes().to_vec()))
            .unwrap_or_default()
    }

    fn parse_sign_message(params: Value) -> Result<WalletConnectAction, String> {
        params.get_value("account")?.string()?;
        params.get_value("message")?.string()?;
        let sign_type = match params.get("protocol").and_then(Value::as_str) {
            None | Some(PROTOCOL_ECDSA) => SignDigestType::BitcoinEcdsa,
            Some(PROTOCOL_BIP322) => SignDigestType::BitcoinBip322,
            Some(protocol) => return Err(format!("unsupported Bitcoin message protocol {protocol}")),
        };
        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Bitcoin,
            sign_type,
            data: params.to_string(),
        })
    }

    fn parse_sign_psbt(params: Value) -> Result<WalletConnectAction, String> {
        let request = BitcoinPsbtRequest::parse(&params.to_string()).map_err(|error| error.to_string())?;
        let transaction_type = WalletConnectTransactionType::BitcoinPsbt;
        let data = params.to_string();
        Ok(match request.broadcast {
            true => WalletConnectAction::SendTransaction {
                chain: Chain::Bitcoin,
                transaction_type,
                data,
            },
            false => WalletConnectAction::SignTransaction {
                chain: Chain::Bitcoin,
                transaction_type,
                data,
            },
        })
    }

    fn parse_send_transfer(params: Value) -> Result<WalletConnectAction, String> {
        let data = params.to_string();
        Self::decode_transfer(data.clone())?;
        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Bitcoin,
            transaction_type: WalletConnectTransactionType::BitcoinTransfer,
            data,
        })
    }

    fn decode_memo(memo: &str) -> Result<String, String> {
        let bytes = hex::decode(memo).map_err(|_| "memo is not hex".to_string())?;
        if bytes.len() > MEMO_MAX_BYTES {
            return Err(format!("memo is longer than {MEMO_MAX_BYTES} bytes"));
        }
        String::from_utf8(bytes).map_err(|_| "memo is not text".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACCOUNT: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

    fn params(json: &str) -> Value {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_sign_message_picks_the_requested_protocol() {
        let sign_type = |protocol: &str| match BitcoinRequestHandler::parse_request(WalletConnectionMethods::BitcoinSignMessage, params(&format!(r#"{{"account":"{ACCOUNT}","message":"Hello"{protocol}}}"#))) {
            Ok(WalletConnectAction::SignMessage { sign_type, .. }) => Ok(sign_type),
            Ok(other) => panic!("{other:?}"),
            Err(error) => Err(error),
        };

        assert_eq!(sign_type(""), Ok(SignDigestType::BitcoinEcdsa), "ECDSA is the default");
        assert_eq!(sign_type(r#","protocol":"ecdsa""#), Ok(SignDigestType::BitcoinEcdsa));
        assert_eq!(sign_type(r#","protocol":"bip322""#), Ok(SignDigestType::BitcoinBip322));
        assert!(sign_type(r#","protocol":"schnorr""#).is_err());
        assert_eq!(BitcoinRequestHandler::decode_message(&format!(r#"{{"account":"{ACCOUNT}","message":"Hello"}}"#)), b"Hello".to_vec());
    }

    #[test]
    fn test_a_message_signs_only_for_the_session_account() {
        assert!(BitcoinRequestHandler::validate_message_account(&format!(r#"{{"account":"{ACCOUNT}","message":"Hi"}}"#), ACCOUNT).is_ok());
        assert!(BitcoinRequestHandler::validate_message_account(&format!(r#"{{"account":"{ACCOUNT}","address":"{ACCOUNT}","message":"Hi"}}"#), ACCOUNT).is_ok());
        assert!(BitcoinRequestHandler::validate_message_account(&format!(r#"{{"account":"{ACCOUNT}","address":"bc1qother","message":"Hi"}}"#), ACCOUNT).is_err());
        assert!(BitcoinRequestHandler::validate_message_account(r#"{"account":"bc1qother","message":"Hi"}"#, ACCOUNT).is_err());
    }

    #[test]
    fn test_send_transfer_reads_amount_and_text_memo() {
        let transfer = |memo: &str| {
            BitcoinRequestHandler::decode_transfer(format!(r#"{{"account":"{ACCOUNT}","recipientAddress":"bc1qrecipient","amount":"123000000"{memo}}}"#)).map(|transaction| match transaction {
                WalletConnectTransaction::BitcoinTransfer { data } => data,
                other => panic!("{other:?}"),
            })
        };

        assert_eq!(
            transfer(r#","memo":"636861726c6579206c6f766573206865""#).unwrap(),
            WCBitcoinTransferData {
                account: ACCOUNT.to_string(),
                recipient_address: "bc1qrecipient".to_string(),
                amount: 123_000_000,
                memo: Some("charley loves he".to_string()),
            }
        );
        assert_eq!(transfer("").unwrap().memo, None);
        assert!(transfer(r#","memo":"zz""#).is_err());
        assert!(transfer(r#","memo":"ff""#).is_err(), "a memo that isn't text can't be shown before it is sent");
        assert!(transfer(&format!(r#","memo":"{}""#, "61".repeat(81))).is_err());
        assert!(BitcoinRequestHandler::decode_transfer(format!(r#"{{"account":"{ACCOUNT}","recipientAddress":"bc1q","amount":"1.5"}}"#)).is_err());
    }

    #[test]
    fn test_sign_psbt_sends_only_when_asked_to_broadcast() {
        let psbt = "cHNidP8B";
        let action = |broadcast: bool| {
            BitcoinRequestHandler::parse_request(
                WalletConnectionMethods::BitcoinSignPsbt,
                params(&format!(r#"{{"account":"{ACCOUNT}","psbt":"{psbt}","signInputs":[{{"address":"{ACCOUNT}","index":0}}],"broadcast":{broadcast}}}"#)),
            )
            .unwrap()
        };

        assert!(matches!(
            action(false),
            WalletConnectAction::SignTransaction {
                transaction_type: WalletConnectTransactionType::BitcoinPsbt,
                ..
            }
        ));
        assert!(matches!(
            action(true),
            WalletConnectAction::SendTransaction {
                transaction_type: WalletConnectTransactionType::BitcoinPsbt,
                ..
            }
        ));
        assert!(BitcoinRequestHandler::parse_request(WalletConnectionMethods::BitcoinSignPsbt, params(&format!(r#"{{"account":"{ACCOUNT}","psbt":"not a psbt"}}"#))).is_ok());
        assert!(
            BitcoinRequestHandler::decode_psbt(format!(r#"{{"account":"{ACCOUNT}","psbt":"not a psbt"}}"#)).is_err(),
            "an unreadable PSBT never reaches a review"
        );
    }
}
