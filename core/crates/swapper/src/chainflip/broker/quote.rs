use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub amount: String,
    pub source_asset: String,
    pub destination_asset: String,
    pub commission_bps: u32,
    pub is_vault_swap: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QuoteType {
    Regular,
    Dca,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub egress_amount_native: BigUint,
    pub recommended_slippage_tolerance_percent: f64,
    pub estimated_duration_seconds: f64,
    #[serde(rename = "type")]
    pub quote_type: QuoteType,
    pub boost_quote: Option<BoostQuote>,
    pub estimated_price: f64,
    pub number_of_chunks: Option<u32>,
    pub chunk_interval_blocks: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostQuote {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub egress_amount_native: BigUint,
    pub recommended_slippage_tolerance_percent: f64,
    pub estimated_duration_seconds: f64,
    pub estimated_boost_fee_bps: u32,
    pub estimated_price: f64,
    pub number_of_chunks: Option<u32>,
    pub chunk_interval_blocks: Option<u32>,
}

impl QuoteResponse {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }
}

impl BoostQuote {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }
}
