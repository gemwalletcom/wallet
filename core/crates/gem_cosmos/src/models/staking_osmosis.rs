use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisMintParamsResponse {
    pub params: OsmosisMintParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisMintParams {
    pub epoch_identifier: String,
    pub distribution_proportions: OsmosisDistributionProportions,
}

impl OsmosisMintParams {
    pub fn epochs_per_year(&self) -> f64 {
        if self.epoch_identifier == "day" { 365.0 } else { 52.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisDistributionProportions {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub staking: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmosisEpochProvisionsResponse {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub epoch_provisions: f64,
}
