use std::collections::HashMap;
use std::str::FromStr;

use num_bigint::BigUint;
use primitives::swap::SwapQuoteData;
#[cfg(feature = "rpc")]
use primitives::{Chain, WalletConnectCAIP2};
use serde::{Deserialize, Serialize};

#[cfg(feature = "rpc")]
use crate::Address;

#[derive(Deserialize)]
pub(crate) struct TonConnectRequest {
    pub(crate) from: Option<String>,
    pub(crate) network: Option<String>,
    pub(crate) messages: Vec<TonConnectMessage>,
    pub(crate) valid_until: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TonConnectMessage {
    pub(crate) address: String,
    pub(crate) amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) payload: Option<String>,
    #[serde(rename = "stateInit", skip_serializing_if = "Option::is_none")]
    pub(crate) state_init: Option<String>,
    #[serde(rename = "extraCurrency", skip_serializing_if = "Option::is_none")]
    pub(crate) extra_currency: Option<HashMap<String, String>>,
}

impl TonConnectMessage {
    pub fn new(address: String, amount: &BigUint, payload: Option<String>) -> Self {
        Self {
            address,
            amount: amount.to_string(),
            payload,
            state_init: None,
            extra_currency: None,
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn swap_quote_data(messages: &[Self]) -> Result<SwapQuoteData, String> {
        let first = messages.first().ok_or("TON swap needs at least one message")?;
        let value = Self::total_amount(messages)?;
        let data = serde_json::to_string(messages).map_err(|error| error.to_string())?;
        Ok(SwapQuoteData::new_contract(first.address.clone(), value, data, None, None))
    }

    pub fn total_amount(messages: &[Self]) -> Result<BigUint, String> {
        messages
            .iter()
            .try_fold(BigUint::ZERO, |total, message| BigUint::from_str(&message.amount).map(|amount| total + amount))
            .map_err(|error| error.to_string())
    }
}

#[derive(Debug)]
pub struct TonConnectTransfer {
    pub to: String,
    pub value: BigUint,
}

pub fn decode_wallet_connect_transfer(data: &str) -> Result<TonConnectTransfer, String> {
    let request: TonConnectRequest = serde_json::from_str(data).map_err(|error| error.to_string())?;
    let to = match request.messages.as_slice() {
        [message] => message.address.clone(),
        _ => String::new(),
    };
    Ok(TonConnectTransfer {
        to,
        value: TonConnectMessage::total_amount(&request.messages)?,
    })
}

#[cfg(feature = "rpc")]
impl TonConnectRequest {
    pub(crate) fn validate_for_emulation(&self) -> Result<(), String> {
        if let Some(network) = self.network.as_deref()
            && WalletConnectCAIP2::get_reference(Chain::Ton).as_deref() != Some(network)
        {
            return Err("TON WalletConnect network does not match wallet network".to_string());
        }
        let [message] = self.messages.as_slice() else {
            return Err("TON WalletConnect requires exactly one message".to_string());
        };
        if message.extra_currency.as_ref().is_some_and(|currencies| !currencies.is_empty()) {
            return Err("TON extra currencies are not supported".to_string());
        }
        Address::parse_user_friendly(&message.address).ok_or_else(|| "TON WalletConnect destination must be user-friendly".to_string())?;
        BigUint::from_str(&message.amount).map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rpc")]
    fn request() -> TonConnectRequest {
        serde_json::from_str(include_str!("../../testdata/wallet_connect_dedust_emulation_request.json")).unwrap()
    }

    #[test]
    #[cfg(feature = "rpc")]
    fn test_validate_for_emulation() {
        assert!(request().validate_for_emulation().is_ok());

        let mut multiple = request();
        multiple.messages.push(request().messages.into_iter().next().unwrap());
        assert_eq!(multiple.validate_for_emulation().unwrap_err(), "TON WalletConnect requires exactly one message");

        let mut extra_currency = request();
        extra_currency.messages[0].extra_currency = Some(HashMap::from([("1".to_string(), "100".to_string())]));
        assert_eq!(extra_currency.validate_for_emulation().unwrap_err(), "TON extra currencies are not supported");
    }

    #[test]
    fn test_decode_wallet_connect_transfer() {
        let transfer = decode_wallet_connect_transfer(include_str!("../../testdata/ton_connect_send_transaction.json")).unwrap();
        assert_eq!(transfer.to, "EQCKWpx7cNMpvmcN5ObM5lLUZHZRFKqYA4xmw9jOry0ZsF9M");
        assert_eq!(transfer.value, BigUint::from(5_000_000u32));

        let batch = decode_wallet_connect_transfer(r#"{"messages":[{"address":"a","amount":"1"},{"address":"b","amount":"2"}]}"#).unwrap();
        assert_eq!(batch.to, "");
        assert_eq!(batch.value, BigUint::from(3u32));

        assert!(decode_wallet_connect_transfer(r#"{"messages":[{"address":"a","amount":"-1"}]}"#).is_err());
        assert!(decode_wallet_connect_transfer(r#"{"valid_until":1}"#).is_err());
    }
}
