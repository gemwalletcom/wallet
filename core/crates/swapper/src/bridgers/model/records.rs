use std::str::FromStr;

use num_bigint::BigUint;
use primitives::{TransactionSwapMetadata, swap::SwapStatus};
use serde::{Deserialize, Serialize};

use crate::{SwapperProvider, bridgers::asset::get_asset_id};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordsRequest {
    pub from_address: String,
    pub page_no: u32,
    pub page_size: u32,
}

impl RecordsRequest {
    pub fn new(from_address: String) -> Self {
        Self { from_address, page_no: 1, page_size: 50 }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordsData {
    pub list: Vec<Record>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
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
pub enum RecordStatus {
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
    use crate::bridgers::model::BridgersResponse;

    #[test]
    fn test_record() {
        let response: BridgersResponse = serde_json::from_str(include_str!("../testdata/records.json")).unwrap();
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
