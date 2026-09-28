use std::error::Error;

use chrono::{DateTime, Utc};
use primitives::asset_score::AssetRank;
use std::sync::Arc;

use crate::ConfigCacher;
use config_keys::ConfigKey;
use primitives::{Asset, AssetBasic, AssetFull, AssetId};
use storage::{AssetFilter, AssetsAddressesRepository, AssetsRepository, Database, DatabaseError, WalletsRepository};

#[derive(Clone)]
pub struct AssetsClient {
    database: Database,
    config: Arc<ConfigCacher>,
}

impl AssetsClient {
    pub fn new(database: Database, config: Arc<ConfigCacher>) -> Self {
        Self { database, config }
    }

    pub async fn get_asset(&self, asset_id: &AssetId) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let asset_id = asset_id.clone();
        Ok(self.database.run(move |client| client.get_asset(&asset_id)).await?)
    }

    pub async fn get_assets(&self, asset_ids: Vec<AssetId>, rate: f64) -> Result<Vec<AssetBasic>, Box<dyn Error + Send + Sync>> {
        let max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let filters = vec![AssetFilter::Ids(asset_ids.iter().map(ToString::to_string).collect())];
        let assets = self.database.run(move |client| client.get_assets_with_prices(filters, max_age)).await?;
        Ok(assets.into_iter().map(|asset| asset.asset_basic_with_rate(rate)).collect())
    }

    pub async fn get_asset_full(&self, asset_id: &AssetId) -> Result<AssetFull, Box<dyn Error + Send + Sync>> {
        let asset_id = asset_id.clone();
        let max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        Ok(self.database.run(move |client| client.get_asset_full(&asset_id, max_age)).await?)
    }

    pub async fn get_assets_by_wallet_id(&self, device_id: i32, wallet_id: i32, from_timestamp: Option<u64>) -> Result<Vec<AssetId>, Box<dyn Error + Send + Sync>> {
        let from_datetime = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));
        let max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let assets = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let chain_addresses = client.get_subscriptions_by_wallet_id(device_id, wallet_id)?;
                let asset_ids = client.get_assets_by_addresses(chain_addresses, from_datetime)?;
                if asset_ids.is_empty() {
                    return Ok(vec![]);
                }
                client.get_assets_with_prices(
                    vec![
                        AssetFilter::IsEnabled(true),
                        AssetFilter::HasPrice(true),
                        AssetFilter::RankGt(AssetRank::Trivial.threshold()),
                        AssetFilter::Ids(asset_ids.iter().map(ToString::to_string).collect()),
                    ],
                    max_age,
                )
            })
            .await?;
        Ok(assets.into_iter().map(|asset| asset.asset.asset.id).collect())
    }
}
