use crate::{AssetId, swap::QuoteAsset};

impl QuoteAsset {
    pub fn mock_with_asset_id(id: AssetId, symbol: &str, decimals: u32) -> Self {
        Self {
            symbol: symbol.to_string(),
            decimals,
            ..Self::from(id)
        }
    }
}
