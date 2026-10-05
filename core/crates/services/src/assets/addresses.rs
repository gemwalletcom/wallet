use primitives::AssetId;

pub(crate) struct TokenAddressesUpdate {
    pub added: usize,
    pub unknown_asset_ids: Vec<AssetId>,
}
