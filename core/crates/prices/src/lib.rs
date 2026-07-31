mod config;
pub mod model;
pub mod providers;

use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::{AssetId, ChartValue};

pub use config::{PriceProviderConfig, PriceProviders, build_price_providers};
pub use model::{AssetPriceFull, AssetPriceMapping, PriceProviderAsset, PriceProviderAssetMetadata};
pub use primitives::PriceProvider;
pub use providers::coingecko::provider::CoinGeckoPricesProvider;
pub use providers::defillama::provider::DefiLlamaProvider;
pub use providers::jupiter::provider::JupiterProvider;
pub use providers::pyth::provider::PythProvider;
pub use providers::tonapi::provider::TonApiProvider;

#[async_trait]
pub trait PriceAssetsProvider: Send + Sync {
    fn provider(&self) -> PriceProvider;
    async fn get_assets(&self, limit: usize) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>>;
    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>>;
    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>>;
    async fn get_assets_new(&self) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
    async fn get_assets_metadata(&self, _mappings: Vec<AssetPriceMapping>) -> Result<Vec<PriceProviderAssetMetadata>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>>;
    async fn get_charts_daily(&self, _provider_price_id: &str) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
    async fn get_charts_hourly(&self, _provider_price_id: &str, _duration: Duration) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
