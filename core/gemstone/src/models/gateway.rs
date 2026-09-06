use crate::address::checksum_address;
use crate::models::GemTransactionInputType;
use primitives::FeePriority;

use primitives::{BroadcastOptions, FeeRate, GasPriceType, TransactionInputType, TransactionPreloadInput, UTXO};

pub type GemUTXO = UTXO;

pub type GemBroadcastOptions = BroadcastOptions;

#[uniffi::remote(Record)]
pub struct BroadcastOptions {
    pub skip_preflight: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFeeRate {
    pub priority: FeePriority,
    pub gas_price_type: GasPriceType,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPreloadInput {
    pub input_type: GemTransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub references: Vec<String>,
}

impl From<FeeRate> for GemFeeRate {
    fn from(fee: FeeRate) -> Self {
        Self {
            priority: fee.priority,
            gas_price_type: fee.gas_price_type,
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
