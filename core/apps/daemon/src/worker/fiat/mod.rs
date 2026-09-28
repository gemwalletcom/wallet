use std::error::Error;

use job_runner::{JobHandle, ShutdownReceiver};
use prices::{FiatRatesProviderConfig, build_fiat_rates_providers};
use primitives::FiatProviderName;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = ctx.services();
    let settings = services.settings();
    let config = services.config();
    let fiat = services.fiat_jobs().await?;
    let providers = build_fiat_rates_providers(&FiatRatesProviderConfig {
        coingecko: settings.coingecko.remote_provider_config(),
        coinmarketcap: settings.coinmarketcap.remote_provider_config(),
    });

    ctx.plan_builder(WorkerService::Fiat, &config, shutdown_rx)
        .jobs(WorkerJob::UpdateFiatRates, providers.keys().copied(), |provider, _| {
            let provider = providers[&provider].clone();
            let fiat = fiat.clone();
            move |_| {
                let updater = fiat.rates_updater(provider.clone());
                async move { updater.update().await }
            }
        })
        .jobs(WorkerJob::UpdateFiatAssets, FiatProviderName::all(), |provider, _| {
            let fiat = fiat.clone();
            move |_| {
                let updater = fiat.assets_updater();
                async move { updater.update_fiat_assets_for(provider).await }
            }
        })
        .jobs(WorkerJob::UpdateFiatProviderCountries, FiatProviderName::all(), |provider, _| {
            let fiat = fiat.clone();
            move |_| {
                let updater = fiat.assets_updater();
                async move { updater.update_fiat_countries_for(provider).await }
            }
        })
        .job(WorkerJob::UpdateFiatBuyableAssets, {
            let fiat = fiat.clone();
            move |_| {
                let updater = fiat.assets_updater();
                async move { updater.update_buyable_assets().await }
            }
        })
        .job(WorkerJob::UpdateFiatSellableAssets, {
            let fiat = fiat.clone();
            move |_| {
                let updater = fiat.assets_updater();
                async move { updater.update_sellable_assets().await }
            }
        })
        .job(WorkerJob::UpdateTrendingFiatAssets, {
            let fiat = fiat.clone();
            move |_| {
                let updater = fiat.assets_updater();
                async move { updater.update_trending_fiat_assets().await }
            }
        })
        .finish()
        .await
}
