use primitives::AssetId;

use crate::services::assets::model::GemAssetText;
use crate::services::localization::GemLocalizedText;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GemReceiveWarning {
    AssetNetwork { symbol: String, network: String },
    NoDestinationTagRequired,
    NoMemoRequired,
}

/// The asset the receive screen shows, with the warnings under its address.
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
    pub standard: Option<GemLocalizedText>,
}
