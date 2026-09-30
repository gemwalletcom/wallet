use primitives::{Asset, AssetId, Wallet};

use crate::services::assets::model::GemAssetText;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GemReceiveWarning {
    AssetNetwork { symbol: String, network: String },
    NoDestinationTagRequired,
    NoMemoRequired,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemReceiveAssetState {
    pub asset: GemAssetText,
    pub warnings: Vec<GemReceiveWarning>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemReceiveNetworks {
    pub networks: Vec<GemReceiveNetwork>,
    pub shows_selector: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemReceiveNetwork {
    pub asset_id: AssetId,
    pub row: crate::services::chain::GemChainRow,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemReceiveSession {
    pub source: Asset,
    pub associations: Vec<AssetId>,
}

#[uniffi::export]
pub fn new_receive_session(asset: Asset) -> GemReceiveSession {
    GemReceiveSession { source: asset, associations: vec![] }
}

#[uniffi::export]
impl GemReceiveSession {
    pub fn on_associations(&self, associations: Vec<AssetId>) -> Self {
        Self { associations, ..self.clone() }
    }

    pub fn networks(&self, wallet: Wallet) -> GemReceiveNetworks {
        super::rules::networks(&self.source, self.associations.clone(), &wallet)
    }
}
