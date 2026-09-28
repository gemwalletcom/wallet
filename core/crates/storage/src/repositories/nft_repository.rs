use chrono::NaiveDateTime;
use diesel::prelude::*;
use std::collections::{HashMap, HashSet};

use primitives::{AssetLink, Chain, Diff, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId};

use crate::models::{NewNftAssetAssociationRow, NewNftAssetRow, NewNftCollectionRow, NftAssetRow, NftCollectionRow, nft_link::NftLinkRow, nft_report::NewNftReportRow};
use crate::repositories::devices_repository::device_row;
use crate::sql_types::{ChainRow, NftAssetIdRow, NftCollectionIdRow};
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NftCollectionFilter {
    UpdatedSince(NaiveDateTime),
    Identifiers(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NftAssetAssociationFilter {
    AddressId(i32),
    Chains(Vec<Chain>),
}

pub trait NftRepository {
    fn get_nft_asset_ids(&mut self, identifiers: Vec<String>) -> Result<Vec<NFTAssetId>, DatabaseError>;
    fn get_nft_collection_ids(&mut self, identifiers: Vec<String>) -> Result<Vec<NFTCollectionId>, DatabaseError>;
    fn get_nft_assets(&mut self, identifiers: Vec<String>) -> Result<Vec<NFTAsset>, DatabaseError>;
    fn get_nft_collections(&mut self, filters: Vec<NftCollectionFilter>) -> Result<Vec<NFTCollection>, DatabaseError>;
    fn get_nft_collection(&mut self, identifier: &str) -> Result<NFTCollection, DatabaseError>;
    fn get_nft_asset_ids_for_address(&mut self, chain: Chain, address: &str) -> Result<Vec<NFTAssetId>, DatabaseError>;
    fn add_nft_collections(&mut self, collections: Vec<NFTCollection>) -> Result<usize, DatabaseError>;
    fn upsert_nft_collection(&mut self, collection: NFTCollection) -> Result<(), DatabaseError>;
    fn add_nft_assets(&mut self, assets: Vec<NFTAsset>) -> Result<usize, DatabaseError>;
    fn upsert_nft_asset(&mut self, collection_id: &NFTCollectionId, asset: NFTAsset) -> Result<(), DatabaseError>;
    fn set_nft_asset_associations(&mut self, address: &str, chains: Vec<Chain>, asset_ids: Vec<NFTAssetId>) -> Result<(), DatabaseError>;
    fn count_nft_assets_by_addresses(&mut self, addresses: Vec<String>, chains: Vec<Chain>) -> Result<i64, DatabaseError>;
    fn add_nft_report(&mut self, device_id: &str, collection_id: &str, asset_id: Option<String>, reason: Option<String>) -> Result<usize, DatabaseError>;
}

fn nft_asset_rows(client: &mut DatabaseClient, identifiers: Vec<String>) -> Result<Vec<NftAssetRow>, diesel::result::Error> {
    use crate::schema::nft_assets::dsl::*;
    nft_assets.filter(identifier.eq_any(identifiers)).select(NftAssetRow::as_select()).load(&mut client.connection)
}

fn nft_collection_rows(client: &mut DatabaseClient, filters: Vec<NftCollectionFilter>) -> Result<Vec<NftCollectionRow>, diesel::result::Error> {
    use crate::schema::nft_collections::dsl::*;
    let mut query = nft_collections.into_boxed();
    for filter in filters {
        match filter {
            NftCollectionFilter::UpdatedSince(value) => query = query.filter(updated_at.gt(value)),
            NftCollectionFilter::Identifiers(values) => query = query.filter(identifier.eq_any(values)),
        }
    }
    query.select(NftCollectionRow::as_select()).load(&mut client.connection)
}

fn nft_collection_pks(client: &mut DatabaseClient, identifiers: Vec<String>) -> Result<HashMap<String, i32>, diesel::result::Error> {
    use crate::schema::nft_collections::dsl::*;
    let rows: Vec<(NftCollectionIdRow, i32)> = nft_collections.filter(identifier.eq_any(identifiers)).select((identifier, id)).load(&mut client.connection)?;
    Ok(rows.into_iter().map(|(value, pk)| (value.to_string(), pk)).collect())
}

fn nft_asset_pks(client: &mut DatabaseClient, identifiers: Vec<String>) -> Result<Vec<i32>, diesel::result::Error> {
    use crate::schema::nft_assets::dsl::*;
    nft_assets.filter(identifier.eq_any(identifiers)).select(id).load(&mut client.connection)
}

fn wallet_address_id(client: &mut DatabaseClient, value: &str) -> Result<Option<i32>, diesel::result::Error> {
    use crate::schema::wallets_addresses::dsl::*;
    wallets_addresses.filter(address.eq(value)).select(id).first(&mut client.connection).optional()
}

fn collections_with_links(client: &mut DatabaseClient, rows: Vec<NftCollectionRow>) -> Result<Vec<NFTCollection>, diesel::result::Error> {
    use crate::schema::nft_collections_links::dsl::*;
    let pks: Vec<i32> = rows.iter().map(|row| row.id).collect();
    let mut links_by_collection: HashMap<i32, Vec<AssetLink>> = nft_collections_links
        .filter(collection_id.eq_any(pks))
        .select(NftLinkRow::as_select())
        .load(&mut client.connection)?
        .into_iter()
        .fold(HashMap::new(), |mut acc, link| {
            acc.entry(link.collection_id).or_default().push(link.as_primitive());
            acc
        });
    Ok(rows.into_iter().map(|row| row.as_primitive(links_by_collection.remove(&row.id).unwrap_or_default())).collect())
}

fn collection_link_rows(collection_pk: i32, links: Vec<AssetLink>) -> Vec<NftLinkRow> {
    links.into_iter().filter(|link| !link.url.is_empty()).filter_map(|link| NftLinkRow::from_primitive(collection_pk, link)).collect()
}

fn add_nft_collections_links(client: &mut DatabaseClient, values: Vec<NftLinkRow>) -> Result<usize, diesel::result::Error> {
    use crate::schema::nft_collections_links::dsl::*;
    diesel::insert_into(nft_collections_links).values(values).on_conflict((collection_id, link_type)).do_nothing().execute(&mut client.connection)
}

fn get_nft_asset(client: &mut DatabaseClient, _identifier: &str) -> Result<NftAssetRow, diesel::result::Error> {
    use crate::schema::nft_assets::dsl::*;
    nft_assets.filter(identifier.eq(_identifier)).select(NftAssetRow::as_select()).first(&mut client.connection)
}

fn get_nft_collection(client: &mut DatabaseClient, _identifier: &str) -> Result<NftCollectionRow, diesel::result::Error> {
    use crate::schema::nft_collections::dsl::*;
    nft_collections.filter(identifier.eq(_identifier)).select(NftCollectionRow::as_select()).first(&mut client.connection)
}

fn get_nft_asset_association_ids_by_filter(client: &mut DatabaseClient, filters: Vec<NftAssetAssociationFilter>) -> Result<Vec<i32>, diesel::result::Error> {
    use crate::schema::nft_assets::dsl::{chain as asset_chain, id as asset_pk, nft_assets};
    use crate::schema::nft_assets_associations::dsl::*;
    let mut query = nft_assets_associations.inner_join(nft_assets.on(asset_pk.eq(asset_id))).into_boxed();
    for filter in filters {
        match filter {
            NftAssetAssociationFilter::AddressId(value) => query = query.filter(address_id.eq(value)),
            NftAssetAssociationFilter::Chains(values) => query = query.filter(asset_chain.eq_any(values.into_iter().map(ChainRow::from).collect::<Vec<_>>())),
        }
    }
    query.select(asset_id).load(&mut client.connection)
}

fn add_nft_asset_associations(client: &mut DatabaseClient, values: Vec<NewNftAssetAssociationRow>) -> Result<usize, diesel::result::Error> {
    use crate::schema::nft_assets_associations::dsl::*;
    diesel::insert_into(nft_assets_associations).values(values).on_conflict((address_id, asset_id)).do_nothing().execute(&mut client.connection)
}

fn delete_nft_asset_associations(client: &mut DatabaseClient, _address_id: i32, asset_ids: Vec<i32>) -> Result<usize, diesel::result::Error> {
    use crate::schema::nft_assets_associations::dsl::*;
    diesel::delete(nft_assets_associations.filter(address_id.eq(_address_id)).filter(asset_id.eq_any(asset_ids))).execute(&mut client.connection)
}

impl NftRepository for DatabaseClient {
    fn get_nft_asset_ids(&mut self, identifiers: Vec<String>) -> Result<Vec<NFTAssetId>, DatabaseError> {
        use crate::schema::nft_assets::dsl::*;
        let rows: Vec<NftAssetIdRow> = nft_assets.filter(identifier.eq_any(identifiers)).select(identifier).load(&mut self.connection)?;
        Ok(rows.into_iter().map(|row| row.0).collect())
    }

    fn get_nft_collection_ids(&mut self, identifiers: Vec<String>) -> Result<Vec<NFTCollectionId>, DatabaseError> {
        use crate::schema::nft_collections::dsl::*;
        let rows: Vec<NftCollectionIdRow> = nft_collections.filter(identifier.eq_any(identifiers)).select(identifier).load(&mut self.connection)?;
        Ok(rows.into_iter().map(|row| row.0).collect())
    }

    fn get_nft_assets(&mut self, identifiers: Vec<String>) -> Result<Vec<NFTAsset>, DatabaseError> {
        use crate::schema::nft_collections::dsl::{id as collection_pk, identifier as collection_identifier, nft_collections};
        let assets = nft_asset_rows(self, identifiers)?;
        let collection_pks: Vec<i32> = assets.iter().map(|asset| asset.collection_id).collect::<HashSet<_>>().into_iter().collect();
        let collection_identifiers: HashMap<i32, NftCollectionIdRow> = nft_collections
            .filter(collection_pk.eq_any(collection_pks))
            .select((collection_pk, collection_identifier))
            .load::<(i32, NftCollectionIdRow)>(&mut self.connection)?
            .into_iter()
            .collect();
        Ok(assets
            .into_iter()
            .filter_map(|row| {
                let collection = collection_identifiers.get(&row.collection_id).cloned()?;
                Some(row.as_primitive(collection))
            })
            .collect())
    }

    fn get_nft_collections(&mut self, filters: Vec<NftCollectionFilter>) -> Result<Vec<NFTCollection>, DatabaseError> {
        let rows = nft_collection_rows(self, filters)?;
        Ok(collections_with_links(self, rows)?)
    }

    fn get_nft_collection(&mut self, identifier: &str) -> Result<NFTCollection, DatabaseError> {
        let row = get_nft_collection(self, identifier).or_not_found(identifier.to_string())?;
        Ok(collections_with_links(self, vec![row])?.remove(0))
    }

    fn get_nft_asset_ids_for_address(&mut self, chain_value: Chain, address_value: &str) -> Result<Vec<NFTAssetId>, DatabaseError> {
        use crate::schema::nft_assets::dsl::{chain, id as asset_pk, identifier, nft_assets};
        use crate::schema::nft_assets_associations::dsl::{address_id, asset_id, nft_assets_associations};
        use crate::schema::wallets_addresses::dsl::{address, id as wallet_address_pk, wallets_addresses};
        let rows: Vec<NftAssetIdRow> = nft_assets
            .filter(asset_pk.eq_any(nft_assets_associations.inner_join(wallets_addresses.on(wallet_address_pk.eq(address_id))).filter(address.eq(address_value)).select(asset_id)))
            .filter(chain.eq(ChainRow::from(chain_value)))
            .select(identifier)
            .load(&mut self.connection)?;
        Ok(rows.into_iter().map(|row| row.0).collect())
    }

    fn add_nft_collections(&mut self, collections: Vec<NFTCollection>) -> Result<usize, DatabaseError> {
        use crate::schema::nft_collections::dsl::*;
        let rows: Vec<NewNftCollectionRow> = collections.iter().cloned().map(NewNftCollectionRow::from_primitive).collect();
        let inserted = diesel::insert_into(nft_collections).values(rows).on_conflict_do_nothing().execute(&mut self.connection)?;
        let pks = nft_collection_pks(self, collections.iter().map(|collection| collection.id.to_string()).collect())?;
        let links: Vec<NftLinkRow> = collections
            .into_iter()
            .filter_map(|collection| pks.get(&collection.id.to_string()).map(|&pk| collection_link_rows(pk, collection.links)))
            .flatten()
            .collect();
        add_nft_collections_links(self, links)?;
        Ok(inserted)
    }

    fn upsert_nft_collection(&mut self, collection: NFTCollection) -> Result<(), DatabaseError> {
        use crate::schema::nft_collections::dsl::*;
        use crate::schema::nft_collections_links::dsl::{collection_id, nft_collections_links};
        let links = collection.links.clone();
        let value = NewNftCollectionRow::from_primitive(collection);
        let row = diesel::insert_into(nft_collections)
            .values(&value)
            .on_conflict(identifier)
            .do_update()
            .set(&value)
            .returning(NftCollectionRow::as_returning())
            .get_result(&mut self.connection)?;
        diesel::delete(nft_collections_links.filter(collection_id.eq(row.id))).execute(&mut self.connection)?;
        let links = collection_link_rows(row.id, links);
        if !links.is_empty() {
            add_nft_collections_links(self, links)?;
        }
        Ok(())
    }

    fn add_nft_assets(&mut self, assets: Vec<NFTAsset>) -> Result<usize, DatabaseError> {
        use crate::schema::nft_assets::dsl::*;
        let collection_pks = nft_collection_pks(self, assets.iter().map(|asset| asset.collection_id.to_string()).collect())?;
        let rows: Vec<NewNftAssetRow> = assets
            .into_iter()
            .filter_map(|asset| collection_pks.get(&asset.collection_id.to_string()).map(|&pk| NewNftAssetRow::from_primitive(asset, pk)))
            .collect();
        if rows.is_empty() {
            return Ok(0);
        }
        Ok(diesel::insert_into(nft_assets).values(rows).on_conflict_do_nothing().execute(&mut self.connection)?)
    }

    fn upsert_nft_asset(&mut self, collection_identifier: &NFTCollectionId, asset: NFTAsset) -> Result<(), DatabaseError> {
        use crate::schema::nft_assets::dsl::*;
        let collection_pk = get_nft_collection(self, &collection_identifier.to_string()).or_not_found(collection_identifier.to_string())?.id;
        let value = NewNftAssetRow::from_primitive(asset, collection_pk);
        diesel::insert_into(nft_assets).values(&value).on_conflict(identifier).do_update().set(&value).execute(&mut self.connection)?;
        Ok(())
    }

    fn set_nft_asset_associations(&mut self, address: &str, chains: Vec<Chain>, asset_ids: Vec<NFTAssetId>) -> Result<(), DatabaseError> {
        let Some(address_id) = wallet_address_id(self, address)? else {
            return Ok(());
        };
        let asset_ids = nft_asset_pks(self, asset_ids.iter().map(ToString::to_string).collect())?;
        let existing = get_nft_asset_association_ids_by_filter(self, vec![NftAssetAssociationFilter::AddressId(address_id), NftAssetAssociationFilter::Chains(chains)])?;
        let diff = Diff::compare(asset_ids, existing);
        let to_insert: Vec<NewNftAssetAssociationRow> = diff.different.into_iter().map(|asset_id| NewNftAssetAssociationRow { address_id, asset_id }).collect();
        if !to_insert.is_empty() {
            add_nft_asset_associations(self, to_insert)?;
        }
        if !diff.missing.is_empty() {
            delete_nft_asset_associations(self, address_id, diff.missing)?;
        }
        Ok(())
    }

    fn add_nft_report(&mut self, device_identifier: &str, collection_identifier: &str, asset_identifier: Option<String>, reason_value: Option<String>) -> Result<usize, DatabaseError> {
        use crate::schema::nft_reports::dsl::*;
        let device = device_row(self, device_identifier).or_not_found(device_identifier.to_string())?;
        let collection = get_nft_collection(self, collection_identifier).or_not_found(collection_identifier.to_string())?;
        let asset = asset_identifier.and_then(|value| get_nft_asset(self, &value).ok());
        let report = NewNftReportRow {
            device_id: device.id,
            collection_id: collection.id,
            asset_id: asset.map(|row| row.id),
            reason: reason_value,
        };
        Ok(diesel::insert_into(nft_reports).values(report).on_conflict_do_nothing().execute(&mut self.connection)?)
    }

    fn count_nft_assets_by_addresses(&mut self, addresses: Vec<String>, chains: Vec<Chain>) -> Result<i64, DatabaseError> {
        use crate::schema::nft_assets::dsl::{chain as asset_chain, id as asset_pk, nft_assets};
        use crate::schema::nft_assets_associations::dsl::*;
        use crate::schema::wallets_addresses::dsl::{address as wallet_address, id as wallet_address_pk, wallets_addresses};

        if addresses.is_empty() || chains.is_empty() {
            return Ok(0);
        }

        Ok(nft_assets_associations
            .inner_join(nft_assets.on(asset_pk.eq(asset_id)))
            .inner_join(wallets_addresses.on(wallet_address_pk.eq(address_id)))
            .filter(wallet_address.eq_any(addresses))
            .filter(asset_chain.eq_any(chains.into_iter().map(ChainRow::from).collect::<Vec<_>>()))
            .select(diesel::dsl::count(asset_id).aggregate_distinct())
            .first(&mut self.connection)?)
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use primitives::currency::Currency;
    use primitives::{AssetLink, Chain, Device, FiatRate, FiatRateProvider, LinkType, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, WalletId, WalletSource, WalletType};

    use crate::{ChainsRepository, Database, DatabaseError, DevicesRepository, FiatRepository, NewWallet, NftCollectionFilter, NftRepository, WalletsRepository};

    const CONTRACT: &str = "0xnftcontract";
    const OWNER: &str = "0xnftowner";

    fn collection() -> NFTCollection {
        NFTCollection {
            id: NFTCollectionId::new(Chain::Ethereum, CONTRACT),
            contract_address: CONTRACT.to_string(),
            links: vec![AssetLink::new("https://x.com/nft", LinkType::X)],
            ..NFTCollection::mock()
        }
    }

    fn asset(token_id: &str) -> NFTAsset {
        let id = NFTAssetId::new(Chain::Ethereum, CONTRACT, token_id);
        NFTAsset {
            collection_id: id.get_collection_id(),
            id,
            contract_address: Some(CONTRACT.to_string()),
            token_id: token_id.to_string(),
            ..NFTAsset::mock()
        }
    }

    #[tokio::test]
    async fn test_nft_collections_assets_and_associations() {
        let database = Database::mock();
        let collection_id = collection().id.to_string();
        let owned = vec![asset("1").id, asset("2").id];
        let (collections, assets, owned_ids, count) = database
            .run(move |client| -> Result<_, DatabaseError> {
                client.add_chains(vec![Chain::Ethereum])?;
                client.set_fiat_rates(FiatRateProvider::Coingecko, vec![FiatRate { symbol: Currency::USD, rate: 1.0 }])?;
                client.add_nft_collections(vec![collection()])?;
                client.add_nft_assets(vec![asset("1"), asset("2"), asset("3")])?;

                client.add_device(Device {
                    id: "nft-test-device".to_string(),
                    ..Device::mock()
                })?;
                let device_id = client.get_device_row_id("nft-test-device")?;
                let wallet = client.get_or_create_wallet(NewWallet {
                    wallet_id: WalletId::Multicoin(OWNER.to_string()),
                    wallet_type: WalletType::Multicoin,
                    source: WalletSource::Import,
                })?;
                client.add_subscriptions(device_id, vec![(wallet.id, Chain::Ethereum, OWNER.to_string())])?;
                client.set_nft_asset_associations(OWNER, vec![Chain::Ethereum], owned)?;

                let collections = client.get_nft_collections(vec![NftCollectionFilter::Identifiers(vec![collection_id])])?;
                let assets = client.get_nft_assets(vec![asset("1").id.to_string()])?;
                let owned_ids = client.get_nft_asset_ids_for_address(Chain::Ethereum, OWNER)?;
                let count = client.count_nft_assets_by_addresses(vec![OWNER.to_string()], vec![Chain::Ethereum])?;
                Ok((collections, assets, owned_ids, count))
            })
            .await
            .unwrap();

        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].id, collection().id);
        assert_eq!(collections[0].links, collection().links);
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, asset("1").id);
        assert_eq!(assets[0].collection_id, collection().id);
        let mut owned_ids = owned_ids;
        owned_ids.sort_by_key(|id| id.to_string());
        assert_eq!(owned_ids, vec![asset("1").id, asset("2").id]);
        assert_eq!(count, 2);
    }
}
