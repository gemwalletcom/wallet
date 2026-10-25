use num_bigint::BigUint;
use primitives::{AssetId, Chain, TransactionSwapReferralFee, asset_constants::XRP_RLUSD_TOKEN_ID};

use crate::models::{XrpPathStep, XrpSwap};

impl XrpSwap {
    pub fn mock() -> Self {
        Self {
            send_max: BigUint::from(995_000_000u64),
            amount: BigUint::from(1_306_046_206_636_212_000u64),
            deliver_min: BigUint::from(1_292_985_744_569_849_000u64),
            paths: vec![vec![XrpPathStep {
                account: None,
                currency: Some("524C555344000000000000000000000000000000".to_string()),
                issuer: Some(XRP_RLUSD_TOKEN_ID.to_string()),
            }]],
            fee: Some(TransactionSwapReferralFee {
                asset_id: AssetId::from_chain(Chain::Xrp),
                value: BigUint::from(5_000_000u64),
            }),
            trust_line: false,
        }
    }
}
