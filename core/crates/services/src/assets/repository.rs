use std::collections::HashSet;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{
    Asset, AssetAssociation, AssetBalance, AssetBasic, AssetFull, AssetId, AssetIdVecExt, AssetPriceMetadata, AssetVecExt, Chain, ChainAddress, ListId, Perpetual, ScanAddress, asset_score::AssetRank, fiat_assets::AssetCatalog,
};
use storage::{
    AssetFilter, AssetUpdate, AssetsAddressesRepository, AssetsRepository, AssetsUsageRanksRepository, Database, DatabaseError, PerpetualsRepository, PricesRepository, ScanAddressesRepository, Tag, TagRepository, TransactionsRepository,
    WalletsRepository,
};

use super::address_changes::AssetAddressChanges;
pub(crate) use super::addresses::TokenAddressesUpdate;
use super::usage_rank_updater::usage_ranks;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RankChange {
    pub(crate) asset_ids: Vec<AssetId>,
    pub(crate) rank: AssetRank,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn get_enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError>;
    async fn update_asset_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError>;
    async fn get_asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError>;
    async fn get_asset_full(&self, asset_id: AssetId, price_max_age: Duration) -> Result<AssetFull, DatabaseError>;
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError>;
    async fn get_assets_with_prices(&self, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError>;
    async fn get_wallet_assets_with_prices(&self, device_id: i32, wallet_id: i32, since: Option<NaiveDateTime>, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError>;
    async fn add_assets(&self, assets: Vec<AssetBasic>) -> Result<usize, DatabaseError>;
    async fn update_assets(&self, filters: Vec<AssetFilter>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError>;
    async fn set_asset_associations(&self, id: String, associations: Vec<AssetAssociation>) -> Result<usize, DatabaseError>;
    async fn add_scan_addresses(&self, addresses: Vec<ScanAddress>) -> Result<usize, DatabaseError>;
    async fn update_coin_address(&self, chain_address: ChainAddress, balance: AssetBalance) -> Result<(), DatabaseError>;
    async fn update_token_addresses(&self, chain_address: ChainAddress, balances: Vec<AssetBalance>) -> Result<TokenAddressesUpdate, DatabaseError>;
    async fn update_image_flags(&self, chain: Chain, asset_ids: HashSet<AssetId>) -> Result<(usize, usize), DatabaseError>;
    async fn update_price_flags(&self) -> Result<(usize, usize), DatabaseError>;
    async fn update_usage_ranks(&self, windows: Vec<(NaiveDateTime, i64)>, retain_since: NaiveDateTime, batch_size: usize) -> Result<usize, DatabaseError>;
    async fn update_perpetuals(&self, assets: Vec<Asset>, asset_updates: Vec<AssetUpdate>, perpetuals: Vec<Perpetual>) -> Result<Result<usize, DatabaseError>, DatabaseError>;
    async fn get_asset_catalog(&self) -> Result<AssetCatalog, DatabaseError>;
    async fn get_list_tag(&self, tag_id: String) -> Result<Option<Tag>, DatabaseError>;
    async fn get_list_tags(&self) -> Result<Vec<Tag>, DatabaseError>;
    async fn set_list_assets(&self, tag_id: String, list_name: String, list_id: ListId, asset_ids: Vec<AssetId>, is_new_tag: bool) -> Result<Option<usize>, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn get_enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError> {
        self.database.run(move |client| client.get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::RankLte(rank.threshold())])).await
    }

    async fn update_asset_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                changes.into_iter().try_fold(0, |count, change| {
                    let updated = client.update_assets(vec![AssetFilter::Ids(change.asset_ids.ids())], vec![AssetUpdate::Rank(change.rank.threshold()), AssetUpdate::IsEnabled(false)])?;
                    Ok::<_, DatabaseError>(count + updated)
                })
            })
            .await
    }

    async fn get_asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        self.database.run(move |client| client.get_asset(&asset_id)).await
    }

    async fn get_asset_full(&self, asset_id: AssetId, price_max_age: Duration) -> Result<AssetFull, DatabaseError> {
        self.database.run(move |client| client.get_asset_full(&asset_id, price_max_age)).await
    }

    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError> {
        self.database.run(move |client| client.get_assets(asset_ids)).await
    }

    async fn get_assets_with_prices(&self, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        self.database.run(move |client| client.get_assets_with_prices(filters, price_max_age)).await
    }

    async fn get_wallet_assets_with_prices(&self, device_id: i32, wallet_id: i32, since: Option<NaiveDateTime>, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        self.database
            .run(move |client| {
                let chain_addresses = client.get_subscriptions_by_wallet_id(device_id, wallet_id)?;
                let asset_ids = client.get_assets_by_addresses(chain_addresses, since)?;
                if asset_ids.is_empty() {
                    return Ok(vec![]);
                }
                client.get_assets_with_prices([filters, vec![AssetFilter::Ids(asset_ids.iter().map(ToString::to_string).collect())]].concat(), price_max_age)
            })
            .await
    }

    async fn add_assets(&self, assets: Vec<AssetBasic>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_assets(assets)).await
    }

    async fn update_assets(&self, filters: Vec<AssetFilter>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_assets(filters, updates)).await
    }

    async fn set_asset_associations(&self, id: String, associations: Vec<AssetAssociation>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_asset_associations(&id, associations)).await
    }

    async fn add_scan_addresses(&self, addresses: Vec<ScanAddress>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_scan_addresses(addresses)).await
    }

    async fn update_coin_address(&self, chain_address: ChainAddress, balance: AssetBalance) -> Result<(), DatabaseError> {
        self.database
            .run(move |client| {
                let changes = AssetAddressChanges::from_coin_balance(&chain_address, balance);
                client.delete_assets_addresses(changes.addresses_to_delete)?;
                client.add_assets_addresses(changes.addresses_to_add)?;
                Ok(())
            })
            .await
    }

    async fn update_token_addresses(&self, chain_address: ChainAddress, balances: Vec<AssetBalance>) -> Result<TokenAddressesUpdate, DatabaseError> {
        self.database
            .run(move |client| {
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
            })
            .await
    }

    async fn update_image_flags(&self, chain: Chain, asset_ids: HashSet<AssetId>) -> Result<(usize, usize), DatabaseError> {
        self.database
            .run(move |client| {
                let ids: Vec<String> = asset_ids.iter().map(ToString::to_string).collect();
                let added = client.update_assets(vec![AssetFilter::Ids(ids.clone()), AssetFilter::HasImage(false)], vec![AssetUpdate::HasImage(true)])?;
                let flagged = vec![AssetFilter::Chain(chain.as_ref().to_string()), AssetFilter::IsEnabled(true), AssetFilter::HasImage(true), AssetFilter::ExcludeIds(ids)];
                let removed = client.update_assets(flagged, vec![AssetUpdate::HasImage(false)])?;
                Ok((added, removed))
            })
            .await
    }

    async fn update_price_flags(&self) -> Result<(usize, usize), DatabaseError> {
        self.database
            .run(|client| {
                let ids: Vec<String> = client.get_prices_asset_ids()?.iter().map(ToString::to_string).collect();
                let added = client.update_assets(vec![AssetFilter::Ids(ids.clone()), AssetFilter::HasPrice(false)], vec![AssetUpdate::HasPrice(true)])?;
                let removed = client.update_assets(vec![AssetFilter::IsEnabled(true), AssetFilter::HasPrice(true), AssetFilter::ExcludeIds(ids)], vec![AssetUpdate::HasPrice(false)])?;
                Ok((added, removed))
            })
            .await
    }

    async fn update_usage_ranks(&self, windows: Vec<(NaiveDateTime, i64)>, retain_since: NaiveDateTime, batch_size: usize) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                let counts = windows.into_iter().map(|(since, weight)| Ok((client.get_asset_usage_counts(since)?, weight))).collect::<Result<Vec<_>, DatabaseError>>()?;
                let rows = usage_ranks(counts);
                client.delete_usage_ranks_before(retain_since)?;
                rows.chunks(batch_size).try_fold(0, |total, batch| Ok(total + client.set_usage_ranks(batch)?))
            })
            .await
    }

    async fn update_perpetuals(&self, assets: Vec<Asset>, asset_updates: Vec<AssetUpdate>, perpetuals: Vec<Perpetual>) -> Result<Result<usize, DatabaseError>, DatabaseError> {
        self.database
            .run(move |client| {
                let asset_ids = assets.ids().ids();
                client.set_assets(assets)?;
                client.update_assets(vec![AssetFilter::Ids(asset_ids)], asset_updates)?;
                Ok(client.update_perpetuals(perpetuals))
            })
            .await
    }

    async fn get_asset_catalog(&self) -> Result<AssetCatalog, DatabaseError> {
        self.database.run(AssetsRepository::get_asset_catalog).await
    }

    async fn get_list_tag(&self, tag_id: String) -> Result<Option<Tag>, DatabaseError> {
        self.database.run(move |client| client.get_tag(&tag_id)).await
    }

    async fn get_list_tags(&self) -> Result<Vec<Tag>, DatabaseError> {
        self.database.run(TagRepository::get_list_tags).await
    }

    async fn set_list_assets(&self, tag_id: String, list_name: String, list_id: ListId, asset_ids: Vec<AssetId>, is_new_tag: bool) -> Result<Option<usize>, DatabaseError> {
        self.database
            .run(move |client| {
                let existing = if asset_ids.is_empty() {
                    HashSet::new()
                } else {
                    client.get_asset_ids_by_filter(vec![AssetFilter::Ids(asset_ids.iter().map(ToString::to_string).collect())])?.into_iter().collect()
                };
                let asset_ids = known_asset_ids(asset_ids, &existing);
                if is_new_tag && client.add_list_tag(&tag_id, &list_name, list_id)? == 0 {
                    return Ok(None);
                }
                if !asset_ids.is_empty() {
                    client.set_assets_tags_for_tag(&tag_id, asset_ids)?;
                }
                Ok(Some(client.get_asset_ids_for_tag(&tag_id)?.len()))
            })
            .await
    }
}

fn known_asset_ids(asset_ids: Vec<AssetId>, existing: &HashSet<AssetId>) -> Vec<AssetId> {
    asset_ids
        .iter()
        .enumerate()
        .filter(|(index, asset_id)| existing.contains(asset_id) && !asset_ids[..*index].contains(asset_id))
        .map(|(_, asset_id)| asset_id.clone())
        .collect()
}
