use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_option_biguint_from_str, serialize_biguint, serialize_option_biguint};
use typeshare::typeshare;

use crate::{SwapProvider, TransactionState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub is_unlimited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SwapQuoteDataType {
    Contract,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteData {
    pub to: String,
    pub data_type: SwapQuoteDataType,
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub data: String,
    pub memo: Option<String>,
    pub approval: Option<ApprovalData>,
    pub gas_limit: Option<String>,
}

impl SwapQuoteData {
    pub fn gas_limit_as_u32(&self) -> Result<u32, &'static str> {
        self.gas_limit.as_ref().ok_or("gas_limit is required")?.parse().map_err(|_| "invalid gas_limit")
    }

    pub fn new_contract(to: String, value: BigUint, data: String, approval: Option<ApprovalData>, gas_limit: Option<String>) -> Self {
        Self {
            to,
            data_type: SwapQuoteDataType::Contract,
            value,
            data,
            memo: None,
            approval,
            gas_limit,
        }
    }

    pub fn new_transfer(to: String, value: BigUint, memo: Option<String>) -> Self {
        Self {
            to,
            data_type: SwapQuoteDataType::Transfer,
            value,
            data: "".to_string(),
            memo,
            approval: None,
            gas_limit: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    pub quote: SwapQuote,
    pub data: SwapQuoteData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapQuote {
    pub from_address: String,
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub from_value: BigUint,
    #[serde(default, serialize_with = "serialize_option_biguint", deserialize_with = "deserialize_option_biguint_from_str")]
    pub min_from_value: Option<BigUint>,
    pub to_address: String,
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub to_value: BigUint,
    pub provider_data: SwapProviderData,
    pub slippage_bps: u32,
    pub eta_in_seconds: Option<u32>,
    pub use_max_amount: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapProviderData {
    pub provider: SwapProvider,
    pub name: String,
    pub protocol_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SwapStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

impl SwapStatus {
    pub fn transaction_state(&self) -> Option<TransactionState> {
        match self {
            SwapStatus::Completed => Some(TransactionState::Confirmed),
            SwapStatus::Failed => Some(TransactionState::Failed),
            SwapStatus::Refunded => Some(TransactionState::Refunded),
            SwapStatus::Pending => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_quote_json_keeps_values_as_decimal_strings() {
        let quote = SwapQuote {
            from_address: "from".to_string(),
            from_value: BigUint::from(1_000_000u32),
            min_from_value: Some(BigUint::from(900_000u32)),
            to_address: "to".to_string(),
            to_value: BigUint::from(2_500_000u32),
            provider_data: SwapProviderData {
                provider: SwapProvider::Jupiter,
                name: "Jupiter".to_string(),
                protocol_name: "Jupiter".to_string(),
            },
            slippage_bps: 50,
            eta_in_seconds: None,
            use_max_amount: None,
        };
        let json = serde_json::to_value(&quote).unwrap();

        assert_eq!(json["fromValue"], serde_json::json!("1000000"));
        assert_eq!(json["minFromValue"], serde_json::json!("900000"));
        assert_eq!(json["toValue"], serde_json::json!("2500000"));
        assert_eq!(serde_json::from_value::<SwapQuote>(json).unwrap().from_value, quote.from_value);
    }

    #[test]
    fn test_swap_quote_data_json_keeps_values_as_decimal_strings() {
        let data = SwapQuoteData::new_contract(
            "0xrouter".to_string(),
            BigUint::from(1_000_000_000_000_000_000u64),
            "0xdata".to_string(),
            Some(ApprovalData {
                token: "0xtoken".to_string(),
                spender: "0xspender".to_string(),
                value: BigUint::from(42u32),
                is_unlimited: false,
            }),
            None,
        );
        let json = serde_json::to_value(&data).unwrap();

        assert_eq!(json["value"], serde_json::json!("1000000000000000000"));
        assert_eq!(json["approval"]["value"], serde_json::json!("42"));
        assert_eq!(serde_json::from_value::<SwapQuoteData>(json).unwrap(), data);
    }
}
