use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use cacher::{AccessTokenCacherClient, CacherClient};
use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use fiat_rates_updater::FiatRatesUpdater;
use job_runner::{JobHandle, ShutdownReceiver};
use pricer::PriceClient;
use prices::{FiatRatesProviderConfig, build_fiat_rates_providers};
use primitives::FiatProviderName;
use std::{error::Error, sync::Arc};
use storage::ConfigCacher;

mod fiat_assets_updater;
mod fiat_rates_updater;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());

    let cacher_client = CacherClient::new(&settings.redis.url).await?;
    let access_token_cacher = Arc::new(AccessTokenCacherClient::new(cacher_client.clone(), FiatProviderName::Transak.id()));
    let providers = build_fiat_rates_providers(&FiatRatesProviderConfig {
        coingecko: settings.coingecko.remote_provider_config(),
        coinmarketcap: settings.coinmarketcap.remote_provider_config(),
    });

    ctx.plan_builder(WorkerService::Fiat, &config, shutdown_rx)
        .jobs(WorkerJob::UpdateFiatRates, providers.keys().copied(), |provider, _| {
            let provider = providers[&provider].clone();
            let price_client = PriceClient::new(database.clone(), cacher_client.clone());
            move |_| {
                let updater = FiatRatesUpdater::new(provider.clone(), price_client.clone());
                async move { updater.update().await }
            }
        })
        .jobs(WorkerJob::UpdateFiatAssets, FiatProviderName::all(), |provider, _| {
            let access_token_cacher = access_token_cacher.clone();
            let settings = settings.clone();
            let database = database.clone();
            move |_| {
                let settings = settings.clone();
                let database = database.clone();
                let access_token_cacher = access_token_cacher.clone();
                let provider = provider;
                async move {
                    let providers = FiatProviderFactory::new_providers((*settings).clone(), access_token_cacher);
                    let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                    fiat_assets_updater.update_fiat_assets_for(provider).await
                }
            }
        })
        .jobs(WorkerJob::UpdateFiatProviderCountries, FiatProviderName::all(), |provider, _| {
            let access_token_cacher = access_token_cacher.clone();
            let settings = settings.clone();
            let database = database.clone();
            move |_| {
                let settings = settings.clone();
                let database = database.clone();
                let access_token_cacher = access_token_cacher.clone();
                let provider = provider;
                async move {
                    let providers = FiatProviderFactory::new_providers((*settings).clone(), access_token_cacher);
                    let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                    fiat_assets_updater.update_fiat_countries_for(provider).await
                }
            }
        })
        .job(WorkerJob::UpdateFiatBuyableAssets, {
            let settings = settings.clone();
            let database = database.clone();
            let access_token_cacher = access_token_cacher.clone();
            move |_| {
                let providers = FiatProviderFactory::new_providers((*settings).clone(), access_token_cacher.clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_buyable_assets().await }
            }
        })
        .job(WorkerJob::UpdateFiatSellableAssets, {
            let settings = settings.clone();
            let database = database.clone();
            let access_token_cacher = access_token_cacher.clone();
            move |_| {
                let providers = FiatProviderFactory::new_providers((*settings).clone(), access_token_cacher.clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_sellable_assets().await }
            }
        })
        .job(WorkerJob::UpdateTrendingFiatAssets, {
            let settings = settings.clone();
            let database = database.clone();
            let access_token_cacher = access_token_cacher.clone();
            move |_| {
                let providers = FiatProviderFactory::new_providers((*settings).clone(), access_token_cacher.clone());
                let fiat_assets_updater = FiatAssetsUpdater::new(database.clone(), providers);
                async move { fiat_assets_updater.update_trending_fiat_assets().await }
            }
        })
        .finish()
}
