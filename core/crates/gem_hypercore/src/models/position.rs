use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_f64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPositions {
    pub asset_positions: Vec<AssetPosition>,
    pub margin_summary: MarginSummary,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub withdrawable: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub account_value: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub total_ntl_pos: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub total_margin_used: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPosition {
    #[serde(rename = "type")]
    pub position_type: PositionType,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PositionType {
    OneWay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub coin: String,
    pub szi: String,
    pub leverage: Leverage,
    pub entry_px: String,
    pub position_value: String,
    pub unrealized_pnl: String,
    pub return_on_equity: String,
    pub liquidation_px: Option<String>,
    pub margin_used: String,
    pub max_leverage: u32,
    pub cum_funding: CumulativeFunding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leverage {
    #[serde(rename = "type")]
    pub leverage_type: LeverageType,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LeverageType {
    Cross,
    Isolated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CumulativeFunding {
    pub all_time: String,
    pub since_open: String,
}
