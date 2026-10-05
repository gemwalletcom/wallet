use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Asset, FeePriority, FeeUnitType};

pub type FeeEstimatesByPriority = BTreeMap<FeePriority, FeeEstimate>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainFeeEstimates {
    pub asset: Asset,
    pub rate_unit: FeeUnitType,
    pub transfer: FeeEstimatesByPriority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_transfer: Option<FeeEstimatesByPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap: Option<FeeEstimatesByPriority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeEstimate {
    pub base: String,
    pub priority_fee: String,
    pub value: String,
    pub fiat_value: String,
}
