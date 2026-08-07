use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SwapPriceImpactType {
    Positive,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct SwapPriceImpact {
    pub percentage: f64,
    pub impact_type: SwapPriceImpactType,
    pub is_high: bool,
}
