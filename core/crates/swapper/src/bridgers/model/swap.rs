use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use super::QuoteRequest;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapRequest {
    #[serde(flatten)]
    pub quote: QuoteRequest,
    pub from_address: String,
    pub to_address: String,
    #[serde(with = "serde_serializers::biguint::string")]
    pub amount_out_min: BigUint,
    pub slippage: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    pub tx_data: EvmTransaction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvmTransaction {
    pub data: String,
    pub to: String,
    pub value: String,
}
