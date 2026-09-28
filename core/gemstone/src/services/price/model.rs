use chrono::{DateTime, Utc};
use primitives::AssetId;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPriceUpdate {
    pub asset_id: AssetId,
    pub price: f64,
    pub price_usd: f64,
    pub price_change_percentage_24h: f64,
    pub updated_at: DateTime<Utc>,
}
