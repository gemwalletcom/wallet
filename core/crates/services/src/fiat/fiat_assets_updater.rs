use chrono::{Duration, Utc};
use fiat::{FiatProvider, model::FiatProviderAsset};
use gem_tracing::info_with_fields;
use primitives::{AssetId, AssetTag, Diff, FiatProviderName, currency::Currency};
use storage::{AssetFilter, AssetUpdate, FiatAssetFilter};
use storage::{AssetsRepository, Database, DatabaseError, FiatRepository, TagRepository};

#[derive(Clone, Copy)]
enum FiatAssetDirection {
    Buy,
    Sell,
}

pub struct FiatAssetsUpdater {
    database: Database,
    providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
}

impl FiatAssetsUpdater {
    pub fn new(database: Database, providers: Vec<Box<dyn FiatProvider + Send + Sync>>) -> Self {
        Self { database, providers }
    }

    pub async fn update_buyable_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .database
            .run(|client| -> Result<usize, DatabaseError> {
                let enabled_asset_ids = client.get_fiat_asset_ids_by_filter(Self::fiat_asset_filters(FiatAssetDirection::Buy))?;
                let buyable_assets_ids = client
                    .get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::IsBuyable(true)])?
                    .into_iter()
                    .map(|x| x.asset.id)
                    .collect::<Vec<AssetId>>();
                let result = Diff::compare(buyable_assets_ids, enabled_asset_ids);

                client.update_assets(result.missing.clone(), vec![AssetUpdate::IsBuyable(true)])?;
                client.update_assets(result.different.clone(), vec![AssetUpdate::IsBuyable(false)])?;

                Ok(result.missing.len() + result.different.len())
            })
            .await?)
    }

    pub async fn update_sellable_assets(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .database
            .run(|client| -> Result<usize, DatabaseError> {
                let enabled_asset_ids = client.get_fiat_asset_ids_by_filter(Self::fiat_asset_filters(FiatAssetDirection::Sell))?;
                let sellable_assets_ids = client
                    .get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::IsSellable(true)])?
                    .into_iter()
                    .map(|x| x.asset.id)
                    .collect::<Vec<AssetId>>();

                let result = Diff::compare(sellable_assets_ids, enabled_asset_ids);
                client.update_assets(result.missing.clone(), vec![AssetUpdate::IsSellable(true)])?;
                client.update_assets(result.different.clone(), vec![AssetUpdate::IsSellable(false)])?;

                Ok(result.missing.len() + result.different.len())
            })
            .await?)
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
        Ok(self
            .database
            .run(move |client| -> Result<usize, DatabaseError> {
                let asset_ids = client.get_fiat_assets_popular(from.naive_utc(), 30)?;
                client.set_assets_tags_for_tag(AssetTag::TrendingFiatPurchase.as_ref(), asset_ids)
            })
            .await?)
    }

    fn get_provider(&self, provider_name: FiatProviderName) -> Result<&(dyn FiatProvider + Send + Sync), Box<dyn std::error::Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|p| p.name() == provider_name)
            .map(|p| p.as_ref())
            .ok_or_else(|| format!("Provider {} not found", provider_name.id()).into())
    }

    pub async fn update_fiat_assets_for(&self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;

        let payment_methods = provider.payment_methods().await;
        let payment_methods_json = serde_json::to_value(&payment_methods)?;
        self.database.run(move |client| client.update_fiat_provider_payment_methods(provider_name, payment_methods_json)).await?;

        let assets = provider.get_assets().await?;
        let asset_count = assets.len();

        let disabled = self
            .database
            .run(move |client| -> Result<usize, DatabaseError> {
                let validated_assets: Vec<(FiatProviderAsset, Option<AssetId>)> = assets
                    .into_iter()
                    .map(|fiat_asset| {
                        let asset_id = fiat_asset.asset_id().filter(|id| client.get_asset(id).is_ok());
                        (fiat_asset, asset_id)
                    })
                    .collect();

                let assets = validated_assets.into_iter().map(|(fiat_asset, asset)| Self::map_fiat_asset(fiat_asset, asset)).collect::<Vec<primitives::FiatAsset>>();
                client.sync_fiat_assets(provider_name, assets)
            })
            .await?;

        info_with_fields!("fiat update assets", provider = provider_name.id(), assets = asset_count, disabled = disabled);

        Ok(asset_count)
    }

    pub async fn update_fiat_countries_for(&self, provider_name: FiatProviderName) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(provider_name)?;
        let countries = provider.get_countries().await?;
        let country_count = countries.len();
        let disabled = self.database.run(move |client| client.sync_fiat_providers_countries(provider_name, countries)).await?;
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
