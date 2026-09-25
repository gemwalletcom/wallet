use primitives::AssetId;

use crate::services::localization::GemLocalizedText;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GemReceiveWarning {
    AssetNetwork,
    NoDestinationTagRequired,
    NoMemoRequired,
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
