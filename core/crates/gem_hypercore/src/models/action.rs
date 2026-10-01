use primitives::decode_hex;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;

pub const ACTION_ID_KEY: &str = "action";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRequest {
    pub action: ExchangeAction,
    pub nonce: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ExchangeAction {
    Order,
    CDeposit {
        wei: u64,
    },
    CWithdraw {
        wei: u64,
    },
    TokenDelegate {
        wei: u64,
        is_undelegate: bool,
    },
    #[serde(other)]
    Other,
}

impl ExchangeAction {
    pub fn places_orders(&self) -> bool {
        match self {
            Self::Order => true,
            Self::CDeposit { .. } | Self::CWithdraw { .. } | Self::TokenDelegate { .. } | Self::Other => false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedExchangeRequest {
    pub action: Value,
    pub nonce: u64,
    pub signature: ExchangeSignature,
    pub vault_address: Option<String>,
    pub expires_after: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeSignature {
    pub r: String,
    pub s: String,
    pub v: u8,
}

impl ExchangeSignature {
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        Ok([decode_hex(&self.r)?, decode_hex(&self.s)?, vec![self.v]].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_request_parses_nonce() {
        let request = include_str!("../../testdata/hl_action_update_position_tp_sl.json").trim();
        assert_eq!(serde_json::from_str::<ExchangeRequest>(request).unwrap().nonce, 1755132472149);
    }

    #[test]
    fn test_exchange_request_rejects_invalid_json() {
        assert!(serde_json::from_str::<ExchangeRequest>("not-json").is_err());
    }
}
