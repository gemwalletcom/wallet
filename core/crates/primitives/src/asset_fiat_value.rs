use model_derive::Model;
use serde::{Deserialize, Serialize};

use crate::AssetId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AssetFiatValue {
    pub asset_id: AssetId,
    pub amount: f64,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}
