use serde::{Deserialize, Serialize};

use crate::{AssetId, SwapProvider, swap::SwapStatus};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwapPartnerTransaction {
    pub provider: SwapProvider,
    pub provider_transaction_id: String,
    pub status: SwapStatus,
    pub from_asset_id: AssetId,
    pub from_value: String,
    pub from_amount_usd: Option<f64>,
    pub to_asset_id: AssetId,
    pub to_value: String,
    pub to_amount_usd: Option<f64>,
    pub referral_fee: Option<SwapReferralFee>,
    pub from_transaction_hash: Option<String>,
    pub to_transaction_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwapReferralFee {
    pub asset_id: AssetId,
    pub value: String,
    pub amount_usd: Option<f64>,
}

impl SwapPartnerTransaction {
    pub fn asset_ids(&self) -> Vec<AssetId> {
        [Some(&self.from_asset_id), Some(&self.to_asset_id), self.referral_fee.as_ref().map(|fee| &fee.asset_id)]
            .into_iter()
            .flatten()
            .cloned()
            .collect()
    }
}
