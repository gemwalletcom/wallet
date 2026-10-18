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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositMemo {
    pub from_token: String,
    pub to_token: String,
    pub sender: String,
    pub destination: String,
    pub min_return_amount: String,
}

impl DepositMemo {
    pub fn new(swap: &SwapRequest, to_code: &str, channel: &str) -> Self {
        Self {
            from_token: swap.quote.from_token_address.clone(),
            to_token: format!("{to_code}|{channel}|{}|bridgers|0", swap.slippage),
            sender: swap.from_address.clone(),
            destination: swap.to_address.clone(),
            min_return_amount: swap.amount_out_min.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    pub tx_data: SwapTransaction,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SwapTransaction {
    Evm(EvmTransaction),
    Ton(TonTransaction),
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvmTransaction {
    pub data: String,
    pub to: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TonTransaction {
    pub address: String,
    pub amount: String,
    pub payload: String,
}
