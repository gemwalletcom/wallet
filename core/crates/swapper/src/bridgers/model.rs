use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str_or_int;

pub(super) mod response_code {
    pub const SUCCESS: u64 = 100;
    pub const QUOTE_FAIL: u64 = 412;
}

const CHANNEL: &str = "ht6zut";

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

pub(super) fn get_to_token(code: &str, slippage: &str) -> String {
    format!("{code}|{CHANNEL}|{slippage}|bridgers|0")
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
pub(super) struct Record {
    pub hash: String,
    pub status: RecordStatus,
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
    use super::*;

    #[test]
    fn test_record_status() {
        let response: BridgersResponse = serde_json::from_str(include_str!("testdata/records_opbnb_ton.json")).unwrap();
        let record = serde_json::from_value::<RecordsData>(response.data).unwrap().list.remove(0);

        assert_eq!(record.hash, "0xb55fa0488d55291f1036b1b707309b1e6101f57dddb3b43fc5d3d336fd960db5");
        assert_eq!(record.status, RecordStatus::ReceiveComplete);
        assert_eq!(serde_json::from_str::<RecordStatus>(r#""refund_complete""#).unwrap(), RecordStatus::RefundComplete);
        assert_eq!(serde_json::from_str::<RecordStatus>(r#""wait_receive_send""#).unwrap(), RecordStatus::Pending);
    }
}
