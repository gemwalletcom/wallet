use std::collections::HashMap;
#[cfg(feature = "rpc")]
use std::str::FromStr;

#[cfg(feature = "rpc")]
use num_bigint::BigUint;
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

#[derive(Serialize, Deserialize)]
pub(crate) struct TonConnectMessage {
    pub(crate) address: String,
    pub(crate) amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) payload: Option<String>,
    #[serde(rename = "stateInit", skip_serializing_if = "Option::is_none")]
    pub(crate) state_init: Option<String>,
    #[serde(rename = "extraCurrency", skip_serializing_if = "Option::is_none")]
    pub(crate) extra_currency: Option<HashMap<String, String>>,
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

#[cfg(all(test, feature = "rpc"))]
mod tests {
    use super::*;

    fn request() -> TonConnectRequest {
        serde_json::from_str(include_str!("../../testdata/wallet_connect_dedust_emulation_request.json")).unwrap()
    }

    #[test]
    fn test_validate_for_emulation() {
        assert!(request().validate_for_emulation().is_ok());

        let mut multiple = request();
        multiple.messages.push(request().messages.into_iter().next().unwrap());
        assert_eq!(multiple.validate_for_emulation().unwrap_err(), "TON WalletConnect requires exactly one message");

        let mut extra_currency = request();
        extra_currency.messages[0].extra_currency = Some(HashMap::from([("1".to_string(), "100".to_string())]));
        assert_eq!(extra_currency.validate_for_emulation().unwrap_err(), "TON extra currencies are not supported");
    }
}
