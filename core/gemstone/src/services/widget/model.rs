use primitives::AssetId;

use crate::formatted_number::GemFormattedNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemWidgetSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWidgetCoin {
    pub asset_id: AssetId,
    pub name: String,
    pub symbol: String,
    pub price: GemFormattedNumber,
    pub change: GemFormattedNumber,
}
