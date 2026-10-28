use gem_ton::tvm::BagOfCells;
use gem_xrp::models::{Amount, TransactionMemo, memos_hash};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_amount: Option<String>,
}

impl DepositMemo {
    pub fn new(swap: &SwapRequest, to_code: &str, channel: &str) -> Self {
        Self {
            from_token: swap.quote.from_token_address.clone(),
            to_token: format!("{to_code}|{channel}|{}|bridgers|0", swap.slippage),
            sender: swap.from_address.clone(),
            destination: swap.to_address.clone(),
            min_return_amount: swap.amount_out_min.to_string(),
            from_amount: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    pub tx_data: SwapTransaction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct DepositData {
    tx_data: DepositTransaction,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DepositTransaction {
    Evm {
        data: String,
    },
    Ton {
        payload: String,
    },
    Xrp {
        #[serde(rename = "Memos")]
        memos: Vec<TransactionMemo>,
    },
}

impl DepositData {
    pub(super) fn deposit_id(&self) -> Option<String> {
        match &self.tx_data {
            DepositTransaction::Evm { data } => Some(data.clone()).filter(|data| !data.is_empty()),
            DepositTransaction::Ton { payload } => Some(hex::encode(BagOfCells::parse_base64_root(payload).ok()?.hash)),
            DepositTransaction::Xrp { memos } => memos_hash(memos),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SwapTransaction {
    Evm(EvmTransaction),
    Ton(TonTransaction),
    Xrp(XrpTransaction),
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XrpTransaction {
    pub transaction_type: String,
    pub destination: String,
    pub destination_tag: Option<u32>,
    pub amount: Amount,
    pub memos: Vec<TransactionMemo>,
}

#[cfg(test)]
mod tests {
    use crate::bridgers::DepositResponse;

    #[test]
    fn test_deposit_id() {
        for (json, expected) in [
            (include_str!("../testdata/swap_ton_bsc_usdt.json"), "03649dcfca44d7fe76c5a22ff8c955097e5cb60d32fef4feb9c368bee9df9d15"),
            (include_str!("../testdata/swap_xrp_bsc_usdt.json"), "3fed38eafa6b3bf232aaf79bd5ce2e20a8e41769eb1759a44a7f1ede8989a995"),
            (include_str!("../testdata/swap_rlusd_bsc_usdt.json"), "6d4a249eec51b33a5dc11e60d1cc6f7653c2d85474986e7c31ac02b1ae392366"),
        ] {
            let response: DepositResponse = serde_json::from_str(json).unwrap();
            assert_eq!(response.deposit_id().unwrap().as_deref(), Some(expected));

            let mut minimal: serde_json::Value = serde_json::from_str(json).unwrap();
            minimal.as_object_mut().unwrap().remove("resMsg");
            minimal["data"]["txData"].as_object_mut().unwrap().retain(|key, _| key == "payload" || key == "Memos");
            let response: DepositResponse = serde_json::from_value(minimal).unwrap();
            assert_eq!(response.deposit_id().unwrap().as_deref(), Some(expected));
        }
    }
}
