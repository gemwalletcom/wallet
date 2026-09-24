use std::collections::HashMap;
use std::slice;
use std::time::Duration;

use chrono::NaiveDateTime;
use diesel::{prelude::*, upsert::excluded};
use primitives::{Asset, AssetAssociation, AssetBasic, AssetFull, AssetId, AssetIdVecExt, AssetPriceMetadata};

use crate::models::{AssetAssociationRow, AssetRow, NewAssetRow};
use crate::repositories::assets_links_repository::AssetsLinksRepository;
use crate::repositories::perpetuals_repository::PerpetualsRepository;
use crate::repositories::prices_repository::primary_price_rows;
use crate::repositories::tag_repository::asset_tag_ids;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

#[derive(Debug, Clone)]
pub enum AssetUpdate {
    IsEnabled(bool),
    IsSwappable(bool),
    IsBuyable(bool),
    IsSellable(bool),
    Rank(i32),
    StakingApr(Option<f64>),
    HasImage(bool),
    HasPrice(bool),
    Supply { circulating_supply: Option<f64>, total_supply: Option<f64>, max_supply: Option<f64> },
}

impl AssetUpdate {
    pub fn supply(circulating: Option<f64>, total: Option<f64>, max: Option<f64>) -> Option<Self> {
        (circulating.is_some() || total.is_some() || max.is_some()).then_some(Self::Supply {
            circulating_supply: circulating,
            total_supply: total,
            max_supply: max,
        })
    }
}

#[derive(Debug, Clone)]
pub enum AssetFilter {
    Ids(Vec<String>),
    IsEnabled(bool),
    IsSwappable(bool),
    IsBuyable(bool),
    IsSellable(bool),
    HasImage(bool),
    HasPrice(bool),
    Chain(String),
    RankLte(i32),
    RankGt(i32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssetSupply {
    pub circulating: Option<f64>,
    pub total: Option<f64>,
    pub max: Option<f64>,
}

pub trait AssetsRepository {
    fn add_assets(&mut self, values: Vec<AssetBasic>) -> Result<usize, DatabaseError>;
    fn update_assets(&mut self, asset_ids: Vec<AssetId>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError>;
    fn upsert_assets(&mut self, values: Vec<Asset>) -> Result<usize, DatabaseError>;
    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetBasic>, DatabaseError>;
    fn get_asset_ids_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError>;
    fn get_asset(&mut self, asset_id: &AssetId) -> Result<Asset, DatabaseError>;
    fn upsert_asset_associations(&mut self, id: &str, values: Vec<AssetAssociation>) -> Result<usize, DatabaseError>;
    fn get_asset_full(&mut self, asset_id: &AssetId, max_age: Duration) -> Result<AssetFull, DatabaseError>;
    fn get_assets(&mut self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError>;
    fn get_assets_supply(&mut self, asset_ids: Vec<AssetId>) -> Result<Vec<(AssetId, AssetSupply)>, DatabaseError>;
    fn get_assets_basic(&mut self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetBasic>, DatabaseError>;
    fn get_assets_with_prices(&mut self, filters: Vec<AssetFilter>, max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError>;
    fn get_swap_assets(&mut self) -> Result<Vec<String>, DatabaseError>;
}

fn filter_assets(filters: Vec<AssetFilter>) -> crate::schema::assets::BoxedQuery<'static, diesel::pg::Pg> {
    use crate::schema::assets::dsl::*;
    let mut query = assets.into_boxed();

    for filter in filters {
        match filter {
            AssetFilter::Ids(values) => {
                query = query.filter(id.eq_any(values));
            }
            AssetFilter::IsEnabled(value) => {
                query = query.filter(is_enabled.eq(value));
            }
            AssetFilter::IsBuyable(value) => {
                query = query.filter(is_buyable.eq(value));
            }
            AssetFilter::IsSellable(value) => {
                query = query.filter(is_sellable.eq(value));
            }
            AssetFilter::IsSwappable(value) => {
                query = query.filter(is_swappable.eq(value));
            }
            AssetFilter::HasImage(value) => {
                query = query.filter(has_image.eq(value));
            }
            AssetFilter::HasPrice(value) => {
                query = query.filter(has_price.eq(value));
            }
            AssetFilter::Chain(value) => {
                query = query.filter(chain.eq(value));
            }
            AssetFilter::RankLte(value) => {
                query = query.filter(rank.le(value));
            }
            AssetFilter::RankGt(value) => {
                query = query.filter(rank.gt(value));
            }
        }
    }

    query
}

fn asset_row(client: &mut DatabaseClient, asset_id: &str) -> Result<AssetRow, diesel::result::Error> {
    use crate::schema::assets::dsl::*;
    assets.find(asset_id).select(AssetRow::as_select()).first(&mut client.connection)
}

pub(crate) fn asset_rows(client: &mut DatabaseClient, asset_ids: Vec<String>) -> Result<Vec<AssetRow>, diesel::result::Error> {
    use crate::schema::assets::dsl::*;
    assets.filter(id.eq_any(asset_ids)).select(AssetRow::as_select()).load(&mut client.connection)
}

pub(crate) fn all_asset_ids(client: &mut DatabaseClient) -> Result<Vec<String>, diesel::result::Error> {
    use crate::schema::assets::dsl::*;
    assets.select(id).load(&mut client.connection)
}

pub(crate) fn asset_ids_updated_since(client: &mut DatabaseClient, since: NaiveDateTime) -> Result<Vec<String>, diesel::result::Error> {
    use crate::schema::assets::dsl::*;
    assets.filter(updated_at.gt(since)).select(id).load(&mut client.connection)
}

fn asset_associations(client: &mut DatabaseClient, requested_asset_id: &str) -> Result<Vec<AssetAssociationRow>, diesel::result::Error> {
    use crate::schema::{assets, assets_associations};

    let Some(requested_id) = assets_associations::table
        .filter(assets_associations::asset_id.eq(requested_asset_id))
        .select(assets_associations::id)
        .first::<String>(&mut client.connection)
        .optional()?
    else {
        return Ok(vec![]);
    };

    assets_associations::table
        .inner_join(assets::table.on(assets::id.eq(assets_associations::asset_id)))
        .filter(assets_associations::id.eq(requested_id))
        .filter(assets_associations::asset_id.ne(requested_asset_id))
        .order((assets::rank.desc(), assets_associations::asset_id.asc()))
        .select(AssetAssociationRow::as_select())
        .load(&mut client.connection)
}

impl AssetsRepository for DatabaseClient {
    fn add_assets(&mut self, values: Vec<AssetBasic>) -> Result<usize, DatabaseError> {
        use crate::schema::assets::dsl::*;
        if values.is_empty() {
            return Ok(0);
        }
        let rows = values.into_iter().map(|x| NewAssetRow::from_primitive(x.asset, x.score, x.properties)).collect::<Vec<_>>();
        Ok(diesel::insert_into(assets).values(rows).on_conflict_do_nothing().execute(&mut self.connection)?)
    }

    fn update_assets(&mut self, asset_ids: Vec<AssetId>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError> {
        use crate::schema::assets::dsl::*;
        let asset_ids = asset_ids.ids();
        if asset_ids.is_empty() || updates.is_empty() {
            return Ok(0);
        }

        Ok(updates.into_iter().try_fold(0, |total, update| {
            let target = assets.filter(id.eq_any(&asset_ids));
            let updated = match update {
                AssetUpdate::IsEnabled(value) => diesel::update(target).set(is_enabled.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::IsSwappable(value) => diesel::update(target).set(is_swappable.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::IsBuyable(value) => diesel::update(target).set(is_buyable.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::IsSellable(value) => diesel::update(target).set(is_sellable.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::Rank(value) => diesel::update(target).set(rank.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::StakingApr(value) => diesel::update(target).set(staking_apr.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::HasImage(value) => diesel::update(target).set(has_image.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::HasPrice(value) => diesel::update(target).set(has_price.eq(value)).execute(&mut self.connection)?,
                AssetUpdate::Supply {
                    circulating_supply: c,
                    total_supply: t,
                    max_supply: m,
                } => diesel::update(target).set((circulating_supply.eq(c), total_supply.eq(t), max_supply.eq(m))).execute(&mut self.connection)?,
            };
            Ok::<_, diesel::result::Error>(total + updated)
        })?)
    }

    fn upsert_assets(&mut self, values: Vec<Asset>) -> Result<usize, DatabaseError> {
        use crate::schema::assets::dsl::*;
        let rows = values.into_iter().map(NewAssetRow::from_primitive_default).collect::<Vec<_>>();
        Ok(diesel::insert_into(assets).values(rows).on_conflict(id).do_update().set((rank.eq(excluded(rank)),)).execute(&mut self.connection)?)
    }

    fn get_assets_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetBasic>, DatabaseError> {
        Ok(filter_assets(filters).select(AssetRow::as_select()).load(&mut self.connection)?.into_iter().map(|x| x.as_basic_primitive()).collect())
    }

    fn get_asset_ids_by_filter(&mut self, filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        use crate::schema::assets::dsl::*;
        let ids: Vec<String> = filter_assets(filters).select(id).load(&mut self.connection)?;
        Ok(ids.into_iter().filter_map(|value| AssetId::new(&value)).collect())
    }

    fn get_asset(&mut self, asset_id: &AssetId) -> Result<Asset, DatabaseError> {
        let id = asset_id.to_string();
        Ok(asset_row(self, &id).or_not_found(id.clone())?.as_primitive())
    }

    fn upsert_asset_associations(&mut self, association_id: &str, values: Vec<AssetAssociation>) -> Result<usize, DatabaseError> {
        use crate::schema::assets_associations::dsl::*;

        if values.is_empty() {
            return Ok(0);
        }

        let rows = values.into_iter().map(|value| AssetAssociationRow::from_primitive(association_id, value)).collect::<Vec<_>>();
        Ok(diesel::insert_into(assets_associations)
            .values(rows)
            .on_conflict(asset_id)
            .do_update()
            .set((id.eq(excluded(id)), association_type.eq(excluded(association_type))))
            .execute(&mut self.connection)?)
    }

    fn get_asset_full(&mut self, asset_id: &AssetId, max_age: Duration) -> Result<AssetFull, DatabaseError> {
        let id = asset_id.to_string();
        let asset = asset_row(self, &id).or_not_found(id.clone())?;
        let price_row = primary_price_rows(self, slice::from_ref(asset_id), max_age)?.into_iter().next().map(|(_, row)| row);
        let market = price_row.as_ref().map(|x| x.as_market_primitive(&asset));
        let price = price_row.as_ref().map(|x| x.as_primitive());
        let links = self.get_asset_links(asset_id)?;
        let associations = asset_associations(self, &id)?.into_iter().map(AssetAssociationRow::into_primitive).collect();
        let tags = asset_tag_ids(self, asset_id)?;
        let perpetuals = self.get_perpetuals_for_asset(asset_id)?;
        let perpetuals = perpetuals.into_iter().map(|x| x.as_basic()).collect();

        Ok(AssetFull {
            price,
            market,
            asset: asset.as_primitive(),
            properties: asset.as_property_primitive(),
            score: asset.as_score_primitive(),
            links,
            associations,
            tags,
            perpetuals,
        })
    }

    fn get_assets(&mut self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError> {
        Ok(asset_rows(self, asset_ids.ids())?.into_iter().map(|x| x.as_primitive()).collect())
    }

    fn get_assets_supply(&mut self, asset_ids: Vec<AssetId>) -> Result<Vec<(AssetId, AssetSupply)>, DatabaseError> {
        Ok(asset_rows(self, asset_ids.ids())?
            .into_iter()
            .map(|asset| {
                let supply = AssetSupply {
                    circulating: asset.circulating_supply,
                    total: asset.total_supply,
                    max: asset.max_supply,
                };
                (asset.as_asset_id(), supply)
            })
            .collect())
    }

    fn get_assets_basic(&mut self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetBasic>, DatabaseError> {
        Ok(asset_rows(self, asset_ids.ids())?.into_iter().map(|x| x.as_basic_primitive()).collect())
    }

    fn get_assets_with_prices(&mut self, filters: Vec<AssetFilter>, max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        let assets: Vec<AssetRow> = filter_assets(filters).select(AssetRow::as_select()).load(&mut self.connection)?;
        let prices = primary_price_rows(self, &assets.iter().map(|asset| asset.as_asset_id()).collect::<Vec<_>>(), max_age)?
            .into_iter()
            .map(|(asset_id, price)| (asset_id, price.as_primitive()))
            .collect::<HashMap<_, _>>();

        Ok(assets
            .into_iter()
            .map(|asset| {
                let price = prices.get(&asset.as_asset_id()).cloned();
                AssetPriceMetadata { asset: asset.as_basic_primitive(), price }
            })
            .collect())
    }

    fn get_swap_assets(&mut self) -> Result<Vec<String>, DatabaseError> {
        use crate::schema::assets::dsl::*;
        Ok(assets.filter(rank.gt(21)).filter(is_swappable.eq(true)).select(id).order(rank.desc()).load(&mut self.connection)?)
    }
}
