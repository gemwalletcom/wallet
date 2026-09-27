use model_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetFiatValue {
    pub amount: f64,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}
