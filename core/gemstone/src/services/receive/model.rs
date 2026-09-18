use primitives::AssetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum GemReceiveWarning {
    AssetNetwork,
    NoDestinationTagRequired,
    NoMemoRequired,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemReceiveNetworks {
    pub asset_ids: Vec<AssetId>,
    pub shows_selector: bool,
}
