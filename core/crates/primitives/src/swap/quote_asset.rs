use serde::{Deserialize, Serialize};

use crate::{AssetId, AssetType, Chain};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuoteAsset {
    pub id: String,
    pub symbol: String,
    pub decimals: u32,
    pub asset_type: AssetType,
}

impl QuoteAsset {
    pub fn asset_id(&self) -> AssetId {
        AssetId::new(&self.id).unwrap()
    }

    pub fn is_native(&self) -> bool {
        self.asset_id().is_native()
    }

    pub fn chain(&self) -> Chain {
        self.asset_id().chain
    }
}

impl From<AssetId> for QuoteAsset {
    fn from(id: AssetId) -> Self {
        let asset_type = id.token_id.as_ref().and_then(|_| id.chain.default_asset_type()).unwrap_or(AssetType::NATIVE);
        Self {
            id: id.to_string(),
            symbol: String::new(),
            decimals: 0,
            asset_type,
        }
    }
}
