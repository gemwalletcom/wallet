use std::collections::{HashMap, HashSet};
use std::time::Duration;

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{Nullable, SingleValue, SqlType};
use diesel::upsert::excluded;
use primitives::{AssetBasic, AssetId, AssetMarket, AssetPriceInfo, ChartTimeframe, Price, PriceData, PriceId, PriceProvider};

use crate::error::ResourceName;
use crate::models::min_max::MinMax;
use crate::models::{ChartRow, PriceAssetRow, PriceProviderConfigRow, PriceRow, price::NewPriceRow, price::PricesChangeset};
use crate::repositories::assets_repository::{all_asset_ids, asset_ids_updated_since, asset_rows};
use crate::repositories::charts_repository::{ChartResult, chart_extremes, chart_price_at, insert_chart_rows};
use crate::repositories::prices_providers_repository::price_provider_rows;
use crate::sql_types::PriceProviderRow;
use crate::{DatabaseClient, DatabaseError, DieselResultExt};

diesel::define_sql_function! {
    fn coalesce<T: SqlType + SingleValue>(a: Nullable<T>, b: Nullable<T>) -> Nullable<T>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetsWithPricesFilter {
    Ids(Vec<String>),
    UpdatedSince(NaiveDateTime),
}

#[derive(Debug, Clone)]
pub enum PriceFilter {
    Provider(PriceProvider),
    UpdatedBefore(NaiveDateTime),
    UpdatedAfter(NaiveDateTime),
    Ids(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceAsset {
    pub asset_id: AssetId,
    pub price_id: PriceId,
}

#[derive(Debug, Clone)]
pub struct AssetWithMarket {
    pub asset: AssetBasic,
    pub market: Option<AssetMarket>,
}

#[derive(Debug, Clone)]
pub enum PriceUpdate {
    AllTimeHigh { value: f64, date: Option<NaiveDateTime> },
    AllTimeLow { value: f64, date: Option<NaiveDateTime> },
    PriceChangePercentage24h(f64),
}

pub trait PricesRepository {
    fn add_prices(&mut self, values: Vec<PriceData>) -> Result<usize, DatabaseError>;
    fn set_prices(&mut self, prices: Vec<PriceData>) -> Result<Vec<AssetId>, DatabaseError>;
    fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, DatabaseError>;
    fn get_prices_by_filter(&mut self, filters: Vec<PriceFilter>) -> Result<Vec<PriceData>, DatabaseError>;
    fn get_prices_asset_ids(&mut self) -> Result<Vec<AssetId>, DatabaseError>;
    fn get_prices_assets_by_provider(&mut self, provider: PriceProvider) -> Result<Vec<PriceAsset>, DatabaseError>;
    fn get_primary_price_key(&mut self, asset_id: &AssetId, max_age: Duration) -> Result<PriceId, DatabaseError>;
    fn get_primary_prices(&mut self, asset_ids: &[AssetId], max_age: Duration) -> Result<Vec<(AssetId, PriceData)>, DatabaseError>;
    fn get_primary_price_infos(&mut self, asset_ids: &[AssetId], max_age: Duration) -> Result<Vec<AssetPriceInfo>, DatabaseError>;
    fn get_price_by_id(&mut self, price_id: &str) -> Result<Price, DatabaseError>;
    fn get_prices_for_asset(&mut self, asset_id: &AssetId) -> Result<Vec<PriceData>, DatabaseError>;
    fn get_price_at(&mut self, asset_id: &AssetId, at: NaiveDateTime) -> Result<Option<ChartResult>, DatabaseError>;
    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError>;
    fn delete_prices(&mut self, ids: Vec<String>) -> Result<usize, DatabaseError>;
    fn get_assets_markets(&mut self, filters: Vec<AssetsWithPricesFilter>, max_age: Duration) -> Result<Vec<AssetWithMarket>, DatabaseError>;
    fn update_prices(&mut self, price_ids: Vec<String>, updates: Vec<PriceUpdate>) -> Result<usize, DatabaseError>;
    fn update_extremes_for_price(&mut self, price_id: &str) -> Result<usize, DatabaseError>;
}

fn prices_for_asset_ids(client: &mut DatabaseClient, asset_ids: &[String]) -> Result<Vec<(String, PriceRow)>, diesel::result::Error> {
    use crate::schema::{prices, prices_assets};

    prices_assets::table
        .inner_join(prices::table.on(prices_assets::price_id.eq(prices::id)))
        .filter(prices_assets::asset_id.eq_any(asset_ids))
        .select((prices_assets::asset_id, PriceRow::as_select()))
        .load::<(crate::sql_types::AssetId, PriceRow)>(&mut client.connection)
        .map(|rows| rows.into_iter().map(|(id, row)| (id.0.to_string(), row)).collect())
}

fn price_row(client: &mut DatabaseClient, price_id: &str) -> Result<PriceRow, diesel::result::Error> {
    use crate::schema::prices::dsl::*;
    prices.filter(id.eq(price_id)).select(PriceRow::as_select()).first(&mut client.connection)
}

fn price_asset_ids_updated_since(client: &mut DatabaseClient, since: NaiveDateTime) -> Result<Vec<String>, diesel::result::Error> {
    use crate::schema::{prices, prices_assets};

    prices_assets::table
        .inner_join(prices::table.on(prices_assets::price_id.eq(prices::id)))
        .filter(prices::last_updated_at.gt(since))
        .select(prices_assets::asset_id)
        .load::<crate::sql_types::AssetId>(&mut client.connection)
        .map(|ids| ids.into_iter().map(|id| id.0.to_string()).collect())
}

fn price_asset(row: PriceAssetRow) -> PriceAsset {
    PriceAsset {
        asset_id: row.asset_id.0,
        price_id: row.price_id.0,
    }
}

fn price_assets_for_price_ids(client: &mut DatabaseClient, ids: Vec<String>) -> Result<Vec<PriceAssetRow>, diesel::result::Error> {
    use crate::schema::prices_assets::dsl::*;
    prices_assets.filter(price_id.eq_any(ids)).select(PriceAssetRow::as_select()).load(&mut client.connection)
}

fn prices_by_filter(client: &mut DatabaseClient, filters: Vec<PriceFilter>) -> Result<Vec<PriceRow>, diesel::result::Error> {
    use crate::schema::prices::dsl::*;
    let query = filters.into_iter().fold(prices.into_boxed(), |q, filter| match filter {
        PriceFilter::Provider(p) => q.filter(provider.eq(PriceProviderRow::from(p))),
        PriceFilter::UpdatedBefore(time) => q.filter(last_updated_at.lt(time).or(last_updated_at.is_null())),
        PriceFilter::UpdatedAfter(time) => q.filter(last_updated_at.ge(time)),
        PriceFilter::Ids(ids) => q.filter(id.eq_any(ids)),
    });
    query.order(market_cap_rank.asc().nulls_last()).select(PriceRow::as_select()).load(&mut client.connection)
}

pub(crate) fn primary_price_rows(client: &mut DatabaseClient, asset_ids: &[AssetId], max_age: Duration) -> Result<Vec<(AssetId, PriceRow)>, DatabaseError> {
    if asset_ids.is_empty() {
        return Ok(vec![]);
    }
    let providers = price_provider_rows(client)?;
    let string_ids: Vec<String> = asset_ids.iter().map(ToString::to_string).collect();
    let mut rows_by_asset: HashMap<String, Vec<PriceRow>> = prices_for_asset_ids(client, &string_ids)?.into_iter().fold(HashMap::new(), |mut acc, (id, row)| {
        acc.entry(id).or_default().push(row);
        acc
    });
    Ok(asset_ids
        .iter()
        .filter_map(|asset_id| {
            let rows = rows_by_asset.remove(&asset_id.to_string())?;
            let row = primary_price(&providers, &rows, max_age)?.clone();
            Some((asset_id.clone(), row))
        })
        .collect())
}

fn upsert_prices(client: &mut DatabaseClient, values: Vec<PriceRow>) -> Result<usize, diesel::result::Error> {
    use crate::schema::prices::dsl::*;
    if values.is_empty() {
        return Ok(0);
    }
    diesel::insert_into(prices)
        .values(&values)
        .on_conflict(id)
        .do_update()
        .set((
            price.eq(excluded(price)),
            price_change_percentage_24h.eq(coalesce(excluded(price_change_percentage_24h), price_change_percentage_24h)),
            market_cap.eq(excluded(market_cap)),
            market_cap_fdv.eq(excluded(market_cap_fdv)),
            market_cap_rank.eq(excluded(market_cap_rank)),
            total_volume.eq(excluded(total_volume)),
            circulating_supply.eq(excluded(circulating_supply)),
            total_supply.eq(excluded(total_supply)),
            max_supply.eq(excluded(max_supply)),
            last_updated_at.eq(excluded(last_updated_at)),
        ))
        .execute(&mut client.connection)
}

impl PricesRepository for DatabaseClient {
    fn add_prices(&mut self, values: Vec<PriceData>) -> Result<usize, DatabaseError> {
        use crate::schema::prices::dsl::*;
        if values.is_empty() {
            return Ok(0);
        }
        let values = values.into_iter().map(NewPriceRow::from_price_data).collect::<Vec<_>>();
        Ok(diesel::insert_into(prices).values(&values).on_conflict_do_nothing().execute(&mut self.connection)?)
    }

    fn set_prices_assets(&mut self, values: Vec<PriceAsset>) -> Result<usize, DatabaseError> {
        use crate::schema::prices_assets::dsl::*;
        if values.is_empty() {
            return Ok(0);
        }
        let values = values.into_iter().map(|value| PriceAssetRow::new(value.asset_id, value.price_id)).collect::<Vec<_>>();
        Ok(diesel::insert_into(prices_assets)
            .values(&values)
            .on_conflict((asset_id, provider))
            .do_update()
            .set(price_id.eq(excluded(price_id)))
            .execute(&mut self.connection)?)
    }

    fn get_prices_by_filter(&mut self, filters: Vec<PriceFilter>) -> Result<Vec<PriceData>, DatabaseError> {
        Ok(prices_by_filter(self, filters)?.iter().map(PriceRow::as_price_data).collect())
    }

    fn get_prices_asset_ids(&mut self) -> Result<Vec<AssetId>, DatabaseError> {
        use crate::schema::prices_assets::dsl::*;
        let ids: Vec<String> = prices_assets.select(asset_id).load(&mut self.connection)?;
        Ok(ids.into_iter().filter_map(|value| AssetId::new(&value)).collect())
    }

    fn get_prices_assets_by_provider(&mut self, price_provider: PriceProvider) -> Result<Vec<PriceAsset>, DatabaseError> {
        use crate::schema::prices_assets::dsl::*;
        let rows = prices_assets.filter(provider.eq(PriceProviderRow::from(price_provider))).select(PriceAssetRow::as_select()).load(&mut self.connection)?;
        Ok(rows.into_iter().map(price_asset).collect())
    }

    fn get_primary_price_key(&mut self, asset_id: &AssetId, max_age: Duration) -> Result<PriceId, DatabaseError> {
        let providers = price_provider_rows(self)?;
        let rows = prices_for_asset_ids(self, &[asset_id.to_string()])?.into_iter().map(|(_, row)| row).collect::<Vec<_>>();
        Ok(primary_price(&providers, &rows, max_age).ok_or_else(|| DatabaseError::not_found(PriceRow::RESOURCE_NAME, asset_id.to_string()))?.id.0.clone())
    }

    fn get_primary_prices(&mut self, asset_ids: &[AssetId], max_age: Duration) -> Result<Vec<(AssetId, PriceData)>, DatabaseError> {
        Ok(primary_price_rows(self, asset_ids, max_age)?.into_iter().map(|(asset_id, row)| (asset_id, row.as_price_data())).collect())
    }

    fn get_primary_price_infos(&mut self, asset_ids: &[AssetId], max_age: Duration) -> Result<Vec<AssetPriceInfo>, DatabaseError> {
        let primary_prices = primary_price_rows(self, asset_ids, max_age)?;
        if primary_prices.is_empty() {
            return Ok(vec![]);
        }
        Ok(primary_prices.into_iter().map(|(asset_id, price)| price.as_price_asset_info(asset_id)).collect())
    }

    fn get_price_by_id(&mut self, price_id: &str) -> Result<Price, DatabaseError> {
        Ok(price_row(self, price_id).or_not_found(price_id.to_string())?.as_primitive())
    }

    fn get_prices_for_asset(&mut self, asset_id: &AssetId) -> Result<Vec<PriceData>, DatabaseError> {
        Ok(prices_for_asset_ids(self, &[asset_id.to_string()])?.into_iter().map(|(_, row)| row.as_price_data()).collect())
    }

    fn get_price_at(&mut self, asset_id: &AssetId, at: NaiveDateTime) -> Result<Option<ChartResult>, DatabaseError> {
        let price_ids = prices_for_asset_ids(self, &[asset_id.to_string()])?.into_iter().map(|(_, row)| row.id.to_string()).collect::<Vec<_>>();
        Ok(chart_price_at(self, &price_ids, at)?)
    }

    fn get_prices_assets_for_price_ids(&mut self, ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError> {
        Ok(price_assets_for_price_ids(self, ids)?.into_iter().map(price_asset).collect())
    }

    fn delete_prices(&mut self, ids: Vec<String>) -> Result<usize, DatabaseError> {
        use crate::schema::prices::dsl::*;
        if ids.is_empty() {
            return Ok(0);
        }
        Ok(diesel::delete(prices.filter(id.eq_any(ids))).execute(&mut self.connection)?)
    }

    fn update_prices(&mut self, price_ids: Vec<String>, updates: Vec<PriceUpdate>) -> Result<usize, DatabaseError> {
        if updates.is_empty() {
            return Ok(0);
        }
        let changeset = PricesChangeset::from_updates(updates);
        use crate::schema::prices::dsl::*;
        if price_ids.is_empty() {
            return Ok(0);
        }
        Ok(diesel::update(prices.filter(id.eq_any(&price_ids))).set(&changeset).execute(&mut self.connection)?)
    }

    fn update_extremes_for_price(&mut self, price_id: &str) -> Result<usize, DatabaseError> {
        let row = price_row(self, price_id).or_not_found(price_id.to_string())?;
        let timeframes = [ChartTimeframe::Raw, ChartTimeframe::Hourly, ChartTimeframe::Daily];
        let extremes: Vec<MinMax<f64>> = timeframes.into_iter().map(|tf| chart_extremes(self, price_id, tf)).collect::<Result<_, _>>()?;
        let combined = MinMax {
            max: extremes.iter().filter_map(|e| e.max).max_by(|a, b| a.value.total_cmp(&b.value)),
            min: extremes.iter().filter_map(|e| e.min).min_by(|a, b| a.value.total_cmp(&b.value)),
        };
        let updates = row.merge_extremes_from_charts(combined);
        self.update_prices(vec![price_id.to_string()], updates)
    }

    fn set_prices(&mut self, prices: Vec<PriceData>) -> Result<Vec<AssetId>, DatabaseError> {
        if prices.is_empty() {
            return Ok(vec![]);
        }
        let prices: Vec<PriceRow> = prices.into_iter().map(PriceRow::from_price_data).collect();
        let price_ids: Vec<String> = prices.iter().map(|p| p.id.to_string()).collect();
        let mappings = price_assets_for_price_ids(self, price_ids)?;
        let mapped_ids: HashSet<String> = mappings.iter().map(|m| m.price_id.to_string()).collect();
        let to_store: Vec<PriceRow> = prices.into_iter().filter(|p| mapped_ids.contains(&p.id.to_string())).collect();
        if to_store.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<String> = to_store.iter().map(|p| p.id.to_string()).collect();
        let incoming_by_id: HashMap<String, PriceRow> = to_store.iter().cloned().map(|p| (p.id.to_string(), p)).collect();
        upsert_prices(self, to_store)?;

        let current_prices = prices_by_filter(self, vec![PriceFilter::Ids(ids)])?;
        let extreme_updates: Vec<(String, Vec<PriceUpdate>)> = current_prices
            .iter()
            .filter_map(|price| {
                let id = price.id.to_string();
                let updates = price.merge_extremes(incoming_by_id.get(&id));
                (!updates.is_empty()).then_some((id, updates))
            })
            .collect();
        for (id, updates) in extreme_updates {
            self.update_prices(vec![id], updates)?;
        }

        let charts: Vec<ChartRow> = current_prices.iter().cloned().map(ChartRow::from_price).collect();
        insert_chart_rows(self, ChartTimeframe::Raw, charts)?;

        Ok(mappings.into_iter().map(|m| m.asset_id.0).collect::<HashSet<_>>().into_iter().collect())
    }

    fn get_assets_markets(&mut self, filters: Vec<AssetsWithPricesFilter>, max_age: Duration) -> Result<Vec<AssetWithMarket>, DatabaseError> {
        let mut ids = None;
        let mut since = None;
        for filter in filters {
            match filter {
                AssetsWithPricesFilter::Ids(values) => ids = Some(values),
                AssetsWithPricesFilter::UpdatedSince(value) => since = Some(value),
            }
        }

        let asset_ids = match since {
            Some(value) => {
                let mut asset_ids = asset_ids_updated_since(self, value)?;
                asset_ids.extend(price_asset_ids_updated_since(self, value)?);
                asset_ids.sort();
                asset_ids.dedup();
                if let Some(ids) = ids {
                    let ids = ids.into_iter().collect::<HashSet<_>>();
                    asset_ids.retain(|id| ids.contains(id));
                }
                asset_ids
            }
            None => match ids {
                Some(ids) => ids,
                None => all_asset_ids(self)?,
            },
        };

        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let providers = price_provider_rows(self)?;
        let assets = asset_rows(self, asset_ids)?;
        let mut prices_by_asset: HashMap<String, Vec<PriceRow>> = prices_for_asset_ids(self, &assets.iter().map(|a| a.id.clone()).collect::<Vec<_>>())?
            .into_iter()
            .fold(HashMap::new(), |mut acc, (asset_id, row)| {
                acc.entry(asset_id).or_default().push(row);
                acc
            });

        Ok(assets
            .into_iter()
            .map(|asset| {
                let rows = prices_by_asset.remove(&asset.id).unwrap_or_default();
                let market = primary_price(&providers, &rows, max_age).map(PriceRow::as_market_primitive);
                AssetWithMarket { asset: asset.as_basic_primitive(), market }
            })
            .collect())
    }
}

fn primary_price<'a>(providers: &[PriceProviderConfigRow], rows: &'a [PriceRow], max_age: Duration) -> Option<&'a PriceRow> {
    let cutoff = (Utc::now() - chrono::Duration::from_std(max_age).ok()?).naive_utc();
    let mut candidates: Vec<(&PriceProviderConfigRow, &PriceRow)> = providers
        .iter()
        .filter(|p| p.enabled)
        .filter_map(|p| rows.iter().find(|row| row.provider.0 == p.id.0).map(|row| (p, row)))
        .filter(|(_, row)| row.last_updated_at >= cutoff)
        .collect();
    candidates.sort_by_key(|(p, _)| p.priority);
    candidates.first().map(|(_, row)| *row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::HOUR;

    #[test]
    fn test_primary_price() {
        let providers = vec![
            PriceProviderConfigRow::new(PriceProvider::Coingecko, true),
            PriceProviderConfigRow::new(PriceProvider::Jupiter, true),
            PriceProviderConfigRow::new(PriceProvider::Pyth, false),
        ];
        let max_age = HOUR;

        let fresh = vec![
            PriceRow {
                market_cap: Some(600.0),
                circulating_supply: Some(300.0),
                ..PriceRow::mock_with_age(PriceProvider::Coingecko, 60)
            },
            PriceRow {
                market_cap: Some(3_000.0),
                circulating_supply: Some(1_000.0),
                ..PriceRow::mock_with_age(PriceProvider::Jupiter, 60)
            },
        ];
        let primary = primary_price(&providers, &fresh, max_age).unwrap();
        assert_eq!(primary.provider.0, PriceProvider::Coingecko);
        assert_eq!(primary.as_market_primitive().market_cap, Some(600.0));
        assert_eq!(primary.as_market_primitive().circulating_supply, Some(300.0));

        let stale_primary = vec![PriceRow::mock_with_age(PriceProvider::Coingecko, 7200), PriceRow::mock_with_age(PriceProvider::Jupiter, 60)];
        assert_eq!(primary_price(&providers, &stale_primary, max_age).unwrap().provider.0, PriceProvider::Jupiter);

        let only_disabled = vec![PriceRow::mock_with_age(PriceProvider::Pyth, 60)];
        assert!(primary_price(&providers, &only_disabled, max_age).is_none());

        assert!(primary_price(&providers, &[], max_age).is_none());
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use std::time::Duration;

    use chrono::Utc;
    use primitives::{Asset, AssetId, AssetType, Chain, PriceData, PriceId, PriceProvider};

    use crate::{AssetsRepository, ChainsRepository, Database, DatabaseError, PriceAsset, PricesProvidersRepository, PricesRepository};

    #[tokio::test]
    async fn test_get_primary_price_infos() {
        let database = Database::mock();
        let asset_id = AssetId::from_token(Chain::Ethereum, "0xpricetest");
        let coingecko_id = PriceId::new(PriceProvider::Coingecko, "price-test".to_string());
        let coingecko = PriceData {
            id: coingecko_id.clone(),
            provider_price_id: coingecko_id.provider_price_id.clone(),
            price: 42.0,
            price_change_percentage_24h: 5.0,
            market_cap: Some(12_600.0),
            circulating_supply: Some(300.0),
            last_updated_at: Utc::now(),
            ..PriceData::mock()
        };
        let jupiter_id = PriceId::new(PriceProvider::Jupiter, "price-test".to_string());
        let jupiter = PriceData {
            id: jupiter_id.clone(),
            provider: PriceProvider::Jupiter,
            provider_price_id: jupiter_id.provider_price_id.clone(),
            price: 41.0,
            market_cap: Some(41_000.0),
            circulating_supply: Some(1_000.0),
            last_updated_at: Utc::now(),
            ..PriceData::mock()
        };
        let requested = asset_id.clone();
        let infos = database
            .run(move |client| -> Result<_, DatabaseError> {
                client.add_chains(vec![Chain::Ethereum])?;
                client.add_assets(vec![Asset::new(requested.clone(), "Price Test".to_string(), "PT".to_string(), 18, AssetType::ERC20).as_basic_primitive()])?;
                client.add_prices_providers(vec![PriceProvider::Coingecko, PriceProvider::Jupiter])?;
                client.add_prices(vec![coingecko, jupiter])?;
                client.set_prices_assets(vec![
                    PriceAsset {
                        asset_id: requested.clone(),
                        price_id: coingecko_id,
                    },
                    PriceAsset {
                        asset_id: requested.clone(),
                        price_id: jupiter_id,
                    },
                ])?;
                client.get_primary_price_infos(&[requested], Duration::from_secs(3600))
            })
            .await
            .unwrap();

        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].asset_id, asset_id);
        assert_eq!(infos[0].price.price, 42.0);
        assert_eq!(infos[0].price.price_change_percentage_24h, 5.0);
        assert_eq!(infos[0].price.provider, PriceProvider::Coingecko);
        assert_eq!(infos[0].market.market_cap, Some(12_600.0));
        assert_eq!(infos[0].market.circulating_supply, Some(300.0));
    }
}
