use crate::actions::{WCBitcoinTransferData, WalletConnectAction, WalletConnectTransaction, WalletConnectTransactionType};
use crate::sign_type::SignDigestType;
use gem_bitcoin::signer::psbt::BitcoinPsbtRequest;
use primitives::{Chain, WalletConnectionMethods, hex};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

const MEMO_MAX_BYTES: usize = 80;

#[derive(Deserialize)]
struct SignMessageParams {
    account: String,
    message: String,
    address: Option<String>,
    #[serde(default)]
    protocol: MessageProtocol,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MessageProtocol {
    #[default]
    Ecdsa,
    Bip322,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SendTransferParams {
    account: String,
    recipient_address: String,
    amount: String,
    memo: Option<String>,
}

pub struct BitcoinRequestHandler;

impl BitcoinRequestHandler {
    pub fn parse_request(method: WalletConnectionMethods, params: Value) -> Result<WalletConnectAction, String> {
        let data = params.to_string();
        match method {
            WalletConnectionMethods::BitcoinGetAccountAddresses => Ok(WalletConnectAction::GetAccounts { chain: Chain::Bitcoin }),
            WalletConnectionMethods::BitcoinSignMessage => Self::parse_sign_message(data),
            WalletConnectionMethods::BitcoinSignPsbt => Self::parse_sign_psbt(data),
            WalletConnectionMethods::BitcoinSendTransfer => Self::parse_send_transfer(data),
            _ => Err("Method not supported".to_string()),
        }
    }

    pub fn decode_psbt(data: String) -> Result<WalletConnectTransaction, String> {
        let request = BitcoinPsbtRequest::parse(&data).map_err(|error| error.to_string())?;
        request.decode_psbt().map_err(|error| error.to_string())?;
        Ok(WalletConnectTransaction::BitcoinPsbt { request, data })
    }

    pub fn decode_transfer(data: String) -> Result<WalletConnectTransaction, String> {
        let params: SendTransferParams = parse(&data)?;
        let memo = params.memo.filter(|memo| !memo.is_empty()).map(|memo| Self::decode_memo(&memo)).transpose()?;
        Ok(WalletConnectTransaction::BitcoinTransfer {
            data: WCBitcoinTransferData {
                account: params.account,
                recipient_address: params.recipient_address,
                amount: params.amount.parse().map_err(|_| format!("invalid amount {}", params.amount))?,
                memo: memo.filter(|memo| !memo.trim().is_empty()),
            },
        })
    }

    pub fn validate_message_account(data: &str, address: &str) -> Result<(), String> {
        let params: SignMessageParams = parse(data)?;
        if params.account != address || params.address.is_some_and(|signer| signer != address) {
            return Err("Bitcoin message address does not match signing account".to_string());
        }
        Ok(())
    }

    pub fn decode_message(data: &str) -> Result<Vec<u8>, String> {
        parse::<SignMessageParams>(data).map(|params| params.message.into_bytes())
    }

    fn parse_sign_message(data: String) -> Result<WalletConnectAction, String> {
        let sign_type = match parse::<SignMessageParams>(&data)?.protocol {
            MessageProtocol::Ecdsa => SignDigestType::BitcoinEcdsa,
            MessageProtocol::Bip322 => SignDigestType::BitcoinBip322,
        };
        Ok(WalletConnectAction::SignMessage { chain: Chain::Bitcoin, sign_type, data })
    }

    fn parse_sign_psbt(data: String) -> Result<WalletConnectAction, String> {
        let request = BitcoinPsbtRequest::parse(&data).map_err(|error| error.to_string())?;
        let transaction_type = WalletConnectTransactionType::BitcoinPsbt;
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

    fn parse_send_transfer(data: String) -> Result<WalletConnectAction, String> {
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

fn parse<T: DeserializeOwned>(data: &str) -> Result<T, String> {
    serde_json::from_str(data).map_err(|error| error.to_string())
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
        assert_eq!(BitcoinRequestHandler::decode_message(&format!(r#"{{"account":"{ACCOUNT}","message":"Hello"}}"#)), Ok(b"Hello".to_vec()));
        assert!(
            BitcoinRequestHandler::decode_message(&format!(r#"{{"account":"{ACCOUNT}"}}"#)).is_err(),
            "a request without a message is never signed as an empty one"
        );
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
