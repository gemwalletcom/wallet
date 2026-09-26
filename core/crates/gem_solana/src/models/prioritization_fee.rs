use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaPrioritizationFee {
    pub prioritization_fee: i64,
}
