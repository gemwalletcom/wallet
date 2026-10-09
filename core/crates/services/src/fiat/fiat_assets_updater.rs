use cacher::AssetCatalogCacher;
use chrono::{Duration, Utc};
use fiat::{FiatProvider, model::FiatProviderAsset};
use gem_tracing::{info_with_fields, warn_with_fields};
use primitives::{AssetId, AssetIdVecExt, AssetTag, FiatProviderName, currency::Currency};
use std::collections::HashSet;
use std::sync::Arc;

use storage::{AssetFilter, AssetUpdate, FiatAssetFilter};

use super::repository::Repository;

#[derive(Clone, Copy)]
enum FiatAssetDirection {
    Buy,
    Sell,
}

impl FiatAssetDirection {
    fn asset_filter(&self, value: bool) -> AssetFilter {
        match self {
            Self::Buy => AssetFilter::IsBuyable(value),
            Self::Sell => AssetFilter::IsSellable(value),
        }
    }

    fn asset_update(&self, value: bool) -> AssetUpdate {
        match self {
            Self::Buy => AssetUpdate::IsBuyable(value),
            Self::Sell => AssetUpdate::IsSellable(value),
        }
    }
}

pub struct FiatAssetsUpdater {
    repository: Arc<dyn Repository>,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
    asset_catalog: Arc<dyn AssetCatalogCacher>,
}

impl FiatAssetsUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>, providers: Vec<Box<dyn FiatProvider + Send + Sync>>, asset_catalog: Arc<dyn AssetCatalogCacher>) -> Self {
        Self { repository, providers, asset_catalog }
    }

    pub async fn update_buyable_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        self.update_asset_flags(FiatAssetDirection::Buy).await
    }

    pub async fn update_sellable_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        self.update_asset_flags(FiatAssetDirection::Sell).await
    }

    async fn update_asset_flags(&self, direction: FiatAssetDirection) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let ids = self.repository.get_fiat_asset_ids(Self::fiat_asset_filters(direction)).await?.ids();
        let added = self.repository.update_assets(vec![AssetFilter::Ids(ids.clone()), direction.asset_filter(false)], vec![direction.asset_update(true)]).await?;
        let removed = self
            .repository
            .update_assets(vec![AssetFilter::IsEnabled(true), direction.asset_filter(true), AssetFilter::ExcludeIds(ids)], vec![direction.asset_update(false)])
            .await?;
        let updated = added + removed;
        self.invalidate_catalog(updated).await;
        Ok(updated)
    }

    async fn invalidate_catalog(&self, updated: usize) {
        if updated > 0
            && let Err(error) = self.asset_catalog.delete_asset_catalog().await
        {
            warn_with_fields!("asset catalog cache invalidation failed", error = error.as_ref());
        }
    }

    fn fiat_asset_filters(direction: FiatAssetDirection) -> Vec<FiatAssetFilter> {
        [
            FiatAssetFilter::HasAssetId,
            FiatAssetFilter::IsEnabled(true),
            FiatAssetFilter::IsEnabledByProvider(true),
            FiatAssetFilter::ProviderEnabled(true),
        ]
        .into_iter()
        .chain(match direction {
            FiatAssetDirection::Buy => [FiatAssetFilter::IsBuyEnabled(true), FiatAssetFilter::ProviderBuyEnabled(true)],
            FiatAssetDirection::Sell => [FiatAssetFilter::IsSellEnabled(true), FiatAssetFilter::ProviderSellEnabled(true)],
        })
        .collect()
    }

    pub async fn update_trending_fiat_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let from = Utc::now() - Duration::days(30);
        Ok(self.repository.set_popular_assets_tag(AssetTag::TrendingFiatPurchase.as_ref().to_string(), from.naive_utc(), 30).await?)
    }

    fn get_provider(&self, provider_name: FiatProviderName) -> Result<&(dyn FiatProvider + Send + Sync), Box<dyn std::error::Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|p| p.name() == provider_name)
            .map(AsRef::as_ref)
            .ok_or_else(|| format!("Provider {} not found", provider_name.id()).into())
    }

    pub async fn update_fiat_assets_for(&self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;

        let payment_methods = provider.payment_methods().await;
        let payment_methods_json = serde_json::to_value(&payment_methods)?;
        self.repository.update_payment_methods(provider_name, payment_methods_json).await?;

        let assets = provider.get_assets().await?;
        let asset_count = assets.len();

        let asset_ids: Vec<AssetId> = assets.iter().filter_map(FiatProviderAsset::asset_id).collect();
        let known: HashSet<AssetId> = self.repository.get_asset_ids(vec![AssetFilter::Ids(asset_ids.ids())]).await?.into_iter().collect();
        let fiat_assets = assets
            .into_iter()
            .map(|asset| {
                let asset_id = asset.asset_id().filter(|asset_id| known.contains(asset_id));
                Self::map_fiat_asset(asset, asset_id)
            })
            .collect();
        let disabled = self.repository.set_fiat_assets(provider_name, fiat_assets).await?;

        info_with_fields!("fiat update assets", provider = provider_name.id(), assets = asset_count, disabled = disabled);

        Ok(asset_count)
    }

    pub async fn update_fiat_countries_for(&self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;
        let countries = provider.get_countries().await?;
        let country_count = countries.len();
        let disabled = self.repository.set_countries(provider_name, countries).await?;
        info_with_fields!("fiat update countries", provider = provider_name.id(), countries = country_count, disabled = disabled);
        Ok(country_count)
    }

    fn map_fiat_asset(fiat_asset: FiatProviderAsset, asset_id: Option<AssetId>) -> primitives::FiatAsset {
        primitives::FiatAsset {
            id: fiat_asset.id,
            asset_id,
            provider: fiat_asset.provider,
            symbol: fiat_asset.symbol,
            network: fiat_asset.network,
            token_id: fiat_asset.token_id,
            enabled: fiat_asset.enabled,
            is_buy_enabled: fiat_asset.is_buy_enabled,
            is_sell_enabled: fiat_asset.is_sell_enabled,
            unsupported_countries: fiat_asset.unsupported_countries.unwrap_or_default(),
            buy_limits: fiat_asset.buy_limits.into_iter().filter(|x| x.currency == Currency::USD).collect::<Vec<_>>(),
            sell_limits: fiat_asset.sell_limits.into_iter().filter(|x| x.currency == Currency::USD).collect::<Vec<_>>(),
        }
    }
}
