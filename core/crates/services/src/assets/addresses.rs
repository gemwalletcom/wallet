use std::collections::HashSet;

use primitives::{AssetAddress, AssetBalance, AssetId, AssetVecExt, ChainAddress};
use storage::{AssetsAddressesRepository, AssetsRepository, DatabaseClient, DatabaseError};

use super::address_changes::AssetAddressChanges;

pub(crate) struct TokenAddressesUpdate {
    pub added: usize,
    pub unknown_asset_ids: Vec<AssetId>,
}

pub(crate) fn update_coin_address(client: &mut DatabaseClient, chain_address: &ChainAddress, balance: AssetBalance) -> Result<(), DatabaseError> {
    let changes = AssetAddressChanges::from_coin_balance(chain_address, balance);
    client.delete_assets_addresses(changes.addresses_to_delete)?;
    client.add_assets_addresses(changes.addresses_to_add)?;
    Ok(())
}

pub(crate) fn update_token_addresses(client: &mut DatabaseClient, chain_address: ChainAddress, balances: Vec<AssetBalance>) -> Result<TokenAddressesUpdate, DatabaseError> {
    let existing_addresses = client.get_asset_addresses(chain_address.clone())?;
    let changes = AssetAddressChanges::from_token_balances(&chain_address, existing_addresses, balances);
    let asset_ids = changes.addresses_to_add.iter().map(|address| address.asset_id.clone()).collect();
    let known_ids: HashSet<_> = client.get_assets(asset_ids)?.ids().into_iter().collect();
    let (addresses_to_add, unknown_addresses): (Vec<_>, Vec<_>) = changes.addresses_to_add.into_iter().partition(|address| known_ids.contains(&address.asset_id));

    let added = addresses_to_add.len();
    client.delete_assets_addresses(changes.addresses_to_delete)?;
    client.add_assets_addresses(addresses_to_add)?;
    Ok(TokenAddressesUpdate {
        added,
        unknown_asset_ids: unknown_addresses.into_iter().map(|address| address.asset_id).collect(),
    })
}

pub fn add_transaction_addresses(client: &mut DatabaseClient, addresses: Vec<AssetAddress>) -> Result<usize, DatabaseError> {
    client.add_assets_addresses(addresses)
}
