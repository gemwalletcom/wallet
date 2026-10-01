use std::str::FromStr;

use num_bigint::BigUint;
use primitives::{TransactionSwapMetadata, swap::SwapStatus};
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str_or_int;

use super::asset::get_asset_id;
use crate::SwapperProvider;

pub(super) mod response_code {
    pub const SUCCESS: u64 = 100;
    pub const QUOTE_FAIL: u64 = 412;
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BridgersResponse {
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub res_code: u64,
    pub res_msg: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct QuoteRequest {
    pub source_flag: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub from_token_amount: String,
    pub from_token_chain: String,
    pub to_token_chain: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct QuoteData {
    pub tx_data: QuoteTxData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct QuoteTxData {
    pub amount_out_min: String,
    pub to_token_amount: String,
    pub deposit_min: String,
    pub deposit_max: String,
    pub chain_fee: String,
    pub estimated_time: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SwapRequest {
    #[serde(flatten)]
    pub quote: QuoteRequest,
    pub from_address: String,
    pub to_address: String,
    pub amount_out_min: String,
    pub slippage: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SwapData {
    pub tx_data: EvmTransaction,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct EvmTransaction {
    pub data: String,
    pub to: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) struct RouteData {
    pub amount_out_min: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RecordsRequest {
    pub from_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct RecordsData {
    pub list: Vec<Record>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Record {
    pub hash: String,
    pub status: RecordStatus,
    pub from_chain: String,
    pub to_chain: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub from_amount: String,
    pub to_amount: String,
}

impl Record {
    pub fn swap_metadata(&self) -> Option<TransactionSwapMetadata> {
        Some(TransactionSwapMetadata::new(
            get_asset_id(&self.from_chain, &self.from_token_address)?,
            BigUint::from_str(&self.from_amount).ok()?,
            get_asset_id(&self.to_chain, &self.to_token_address)?,
            BigUint::from_str(&self.to_amount).ok()?,
            SwapperProvider::Bridgers,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RecordStatus {
    ReceiveComplete,
    RefundComplete,
    Timeout,
    #[serde(other)]
    Pending,
}

impl RecordStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self {
            Self::ReceiveComplete => SwapStatus::Completed,
            Self::RefundComplete => SwapStatus::Refunded,
            Self::Timeout => SwapStatus::Failed,
            Self::Pending => SwapStatus::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use primitives::{AssetId, Chain, asset_constants::ARBITRUM_USDC_ASSET_ID};

    use super::*;

    #[test]
    fn test_record() {
        let response: BridgersResponse = serde_json::from_str(include_str!("testdata/records.json")).unwrap();
        let records = serde_json::from_value::<RecordsData>(response.data).unwrap().list;
        let arbitrum_to_opbnb = &records[0];
        let opbnb_to_ton = &records[1];

        assert_eq!(arbitrum_to_opbnb.hash, "0x8cb98cd501b2e0b5da96a97492c1f1819fe2a72de6db8a8c40c6d5b862a7cd28");
        assert_eq!(arbitrum_to_opbnb.status, RecordStatus::ReceiveComplete);
        assert_eq!(
            arbitrum_to_opbnb.swap_metadata(),
            Some(TransactionSwapMetadata {
                from_asset: ARBITRUM_USDC_ASSET_ID.clone(),
                from_value: BigUint::from(20_000_000u64),
                to_asset: AssetId::from_chain(Chain::OpBNB),
                to_value: BigUint::from(25_126_000_000_000_000u64),
                provider: Some("bridgers".to_string()),
                referral_fee: None,
            })
        );
        assert_eq!(opbnb_to_ton.swap_metadata(), None);
        assert_eq!(serde_json::from_str::<RecordStatus>(r#""refund_complete""#).unwrap(), RecordStatus::RefundComplete);
        assert_eq!(serde_json::from_str::<RecordStatus>(r#""wait_receive_send""#).unwrap(), RecordStatus::Pending);
    }
}
