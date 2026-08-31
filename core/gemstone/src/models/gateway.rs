use crate::address::checksum_address;
use crate::models::GemTransactionInputType;
use crate::models::custom_types::GemBigInt;
use primitives::FeePriority;

#[uniffi::remote(Enum)]
pub enum FeePriority {
    Normal,
    Fast,
}

use primitives::{BroadcastOptions, FeeRate, GasPriceType, TransactionInputType, TransactionPreloadInput, UTXO};

pub type GemUTXO = UTXO;

pub type GemBroadcastOptions = BroadcastOptions;

#[uniffi::remote(Record)]
pub struct BroadcastOptions {
    pub skip_preflight: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, uniffi::Enum)]
pub enum GemGasPriceType {
    Regular {
        #[serde(with = "crate::models::custom_types::decimal_string")]
        gas_price: GemBigInt,
    },
    Eip1559 {
        #[serde(with = "crate::models::custom_types::decimal_string")]
        gas_price: GemBigInt,
        #[serde(with = "crate::models::custom_types::decimal_string")]
        priority_fee: GemBigInt,
    },
    Solana {
        #[serde(with = "crate::models::custom_types::decimal_string")]
        gas_price: GemBigInt,
        #[serde(with = "crate::models::custom_types::decimal_string")]
        priority_fee: GemBigInt,
        #[serde(with = "crate::models::custom_types::decimal_string")]
        unit_price: GemBigInt,
    },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFeeRate {
    pub priority: FeePriority,
    pub gas_price_type: GemGasPriceType,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPreloadInput {
    pub input_type: GemTransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub references: Vec<String>,
}

impl From<GasPriceType> for GemGasPriceType {
    fn from(value: GasPriceType) -> Self {
        match value {
            GasPriceType::Regular { gas_price } => GemGasPriceType::Regular { gas_price },
            GasPriceType::Eip1559 { gas_price, priority_fee } => GemGasPriceType::Eip1559 { gas_price, priority_fee },
            GasPriceType::Solana {
                gas_price,
                priority_fee,
                unit_price,
            } => GemGasPriceType::Solana {
                gas_price,
                priority_fee,
                unit_price,
            },
        }
    }
}

impl From<FeeRate> for GemFeeRate {
    fn from(fee: FeeRate) -> Self {
        Self {
            priority: fee.priority,
            gas_price_type: fee.gas_price_type.into(),
        }
    }
}

impl From<TransactionPreloadInput> for GemTransactionPreloadInput {
    fn from(input: TransactionPreloadInput) -> Self {
        Self {
            input_type: input.input_type.into(),
            sender_address: input.sender_address,
            destination_address: input.destination_address,
            references: input.references,
        }
    }
}

impl From<GemTransactionPreloadInput> for TransactionPreloadInput {
    fn from(input: GemTransactionPreloadInput) -> Self {
        let input_type: TransactionInputType = input.input_type.into();
        let destination_address = checksum_address(&input.destination_address, input_type.get_asset().chain());
        Self {
            input_type,
            sender_address: input.sender_address,
            destination_address,
            references: input.references,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_price_keeps_the_decimal_string_wire_format() {
        let json = r#"{"Eip1559":{"gas_price":"1000000000","priority_fee":"25"}}"#;

        let decoded: GemGasPriceType = serde_json::from_str(json).unwrap();
        assert!(matches!(
            &decoded,
            GemGasPriceType::Eip1559 { gas_price, priority_fee }
                if *gas_price == GemBigInt::from(1_000_000_000) && *priority_fee == GemBigInt::from(25)
        ));
        assert_eq!(serde_json::to_string(&decoded).unwrap(), json);
    }

    #[test]
    fn test_a_malformed_gas_price_is_rejected_rather_than_read_as_zero() {
        assert!(serde_json::from_str::<GemGasPriceType>(r#"{"Regular":{"gas_price":"not-a-number"}}"#).is_err());
        assert!(serde_json::from_str::<GemGasPriceType>(r#"{"Regular":{"gas_price":""}}"#).is_err());
    }
}
