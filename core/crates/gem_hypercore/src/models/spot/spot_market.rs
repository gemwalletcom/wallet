use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_f64_from_str, deserialize_option_f64_from_str};

use super::super::token::SpotToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotMarket {
    pub tokens: Vec<i32>,
    pub name: String,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotMeta {
    pub tokens: Vec<SpotToken>,
    pub universe: Vec<SpotMarket>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetContext {
    pub coin: String,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub mark_px: f64,
    #[serde(default, deserialize_with = "deserialize_option_f64_from_str")]
    pub mid_px: Option<f64>,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub prev_day_px: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotMetaAndAssetContexts(pub SpotMeta, pub Vec<SpotAssetContext>);
