use cacher::AssetCatalogCacher;
use chrono::{Duration, Utc};
use fiat::{FiatProvider, model::FiatProviderAsset};
use gem_tracing::{info_with_fields, warn_with_fields};
use primitives::{AssetId, AssetTag, FiatProviderName, currency::Currency};
use std::sync::Arc;

use storage::{AssetFilter, AssetUpdate, FiatAssetFilter};

use super::repository::Repository;

#[derive(Clone, Copy)]
enum FiatAssetDirection {
    Buy,
    Sell,
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
        let asset_filters = vec![AssetFilter::IsEnabled(true), AssetFilter::IsBuyable(true)];
        let updated = self.repository.sync_asset_flag(Self::fiat_asset_filters(FiatAssetDirection::Buy), asset_filters, AssetUpdate::IsBuyable).await?;
        self.invalidate_catalog(updated).await;
        Ok(updated)
    }

    pub async fn update_sellable_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let asset_filters = vec![AssetFilter::IsEnabled(true), AssetFilter::IsSellable(true)];
        let updated = self.repository.sync_asset_flag(Self::fiat_asset_filters(FiatAssetDirection::Sell), asset_filters, AssetUpdate::IsSellable).await?;
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

        let disabled = self.repository.sync_fiat_assets(provider_name, assets, Self::map_fiat_asset).await?;

        info_with_fields!("fiat update assets", provider = provider_name.id(), assets = asset_count, disabled = disabled);

        Ok(asset_count)
    }

    pub async fn update_fiat_countries_for(&self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;
        let countries = provider.get_countries().await?;
        let country_count = countries.len();
        let disabled = self.repository.sync_countries(provider_name, countries).await?;
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
