use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisPoolManagerParamsResponse {
    pub params: OsmosisPoolManagerParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisPoolManagerParams {
    pub taker_fee_params: OsmosisTakerFeeParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisTakerFeeParams {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub daily_staking_rewards_smoothing_factor: f64,
}

impl OsmosisTakerFeeParams {
    pub fn annual_staking_rewards(&self, staking_rewards_buffer: f64) -> f64 {
        staking_rewards_buffer / self.daily_staking_rewards_smoothing_factor * 365.0
    }
}
