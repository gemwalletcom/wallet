use primitives::AssetId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StellarSwapData {
    pub send_amount: u64,
    pub destination_min: u64,
    pub path: Vec<AssetId>,
    pub change_trust: bool,
    pub fee: Option<StellarSwapFee>,
    pub max_time: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StellarSwapFee {
    pub destination: String,
    pub asset: AssetId,
    pub amount: u64,
}

impl StellarSwapData {
    pub fn operation_count(&self) -> u64 {
        1 + u64::from(self.change_trust) + u64::from(self.fee.is_some())
    }
}
