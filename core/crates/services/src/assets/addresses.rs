use primitives::{AssetAddress, AssetId};
use storage::{AssetsAddressesRepository, DatabaseClient, DatabaseError};

pub(crate) struct TokenAddressesUpdate {
    pub added: usize,
    pub unknown_asset_ids: Vec<AssetId>,
}

pub fn add_transaction_addresses(client: &mut DatabaseClient, addresses: Vec<AssetAddress>) -> Result<usize, DatabaseError> {
    client.add_assets_addresses(addresses)
}
