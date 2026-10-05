use std::str::FromStr;

use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{TransactionSwapMetadata, swap::SwapStatus};
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str_or_int;

use super::asset::get_asset_id;
use crate::{SwapperError, SwapperProvider};

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
    #[serde(with = "serde_serializers::biguint::string")]
    pub from_token_amount: BigUint,
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
    #[serde(with = "serde_serializers::biguint::string")]
    pub amount_out_min: BigUint,
    pub to_token_amount: String,
    pub deposit_min: String,
    pub deposit_max: String,
    pub chain_fee: String,
    pub estimated_time: u32,
}

impl QuoteTxData {
    pub fn eta_in_seconds(&self) -> Option<u32> {
        match self.estimated_time {
            1 => Some(180),
            2 | 10 => Some(600),
            3 => Some(1800),
            _ => None,
        }
    }

    pub fn get_deposit_range(&self, decimals: u32) -> Result<(BigUint, BigUint), SwapperError> {
        Ok((
            BigNumberFormatter::value_from_amount_biguint(&self.deposit_min, decimals)?,
            BigNumberFormatter::value_from_amount_biguint(&self.deposit_max, decimals)?,
        ))
    }

    pub fn get_to_value(&self, decimals: u32) -> Result<BigUint, SwapperError> {
        let to_amount = BigNumberFormatter::value_from_amount_biguint(&self.to_token_amount, decimals)?;
        let chain_fee = BigNumberFormatter::value_from_amount_biguint(&self.chain_fee, decimals)?;
        if to_amount <= chain_fee {
            return Err(SwapperError::NoQuoteAvailable);
        }
        Ok(to_amount - chain_fee)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SwapRequest {
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
    #[serde(with = "serde_serializers::biguint::string")]
    pub amount_out_min: BigUint,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RecordsRequest {
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
    fn test_quote_values() {
        let quote = QuoteTxData {
            amount_out_min: BigUint::ZERO,
            to_token_amount: "0.599919".to_string(),
            deposit_min: "147.8".to_string(),
            deposit_max: "20000".to_string(),
            chain_fee: "0.0001".to_string(),
            estimated_time: 10,
        };

        assert_eq!(quote.get_deposit_range(18).unwrap(), (BigUint::from(147_800_000_000_000_000_000u128), BigUint::from(20_000_000_000_000_000_000_000u128)));
        assert_eq!(quote.get_to_value(8).unwrap(), BigUint::from(59_981_900u64));
        assert_eq!(QuoteTxData { chain_fee: "1".to_string(), ..quote }.get_to_value(8).unwrap_err(), SwapperError::NoQuoteAvailable);
    }

    #[test]
    fn test_quote_eta_in_seconds() {
        let quote = |estimated_time| QuoteTxData {
            amount_out_min: BigUint::ZERO,
            to_token_amount: String::new(),
            deposit_min: String::new(),
            deposit_max: String::new(),
            chain_fee: String::new(),
            estimated_time,
        };

        assert_eq!(quote(1).eta_in_seconds(), Some(180));
        assert_eq!(quote(2).eta_in_seconds(), Some(600));
        assert_eq!(quote(3).eta_in_seconds(), Some(1800));
        assert_eq!(quote(10).eta_in_seconds(), Some(600));
        assert_eq!(quote(4).eta_in_seconds(), None);
    }

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
