use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SwapPriceImpactType {
    Positive,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SwapPriceImpact {
    pub percentage: f64,
    pub impact_type: SwapPriceImpactType,
    pub is_high: bool,
    pub shows_in_summary: bool,
}
