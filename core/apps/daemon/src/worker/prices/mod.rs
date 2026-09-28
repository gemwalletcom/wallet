use std::error::Error;
use std::future::Future;

use config_keys::ConfigParamKey;
use futures::future::BoxFuture;
use job_runner::{JobContext, JobHandle, ShutdownReceiver};
use prices::PriceProvider;
use primitives::ChartTimeframe;
use services::PriceJobs;
use services::prices::{ChartsHistoryConfig, PricesUpdater};

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::{JobVariant, WorkerJob};
use crate::worker::plan::JobPlanBuilder;

type JobFuture = BoxFuture<'static, Result<usize, Box<dyn Error + Send + Sync>>>;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = ctx.services();
    let config = services.config();
    let assets_producer = services.stream_producer("prices_provider_assets", streamer::no_shutdown()).await?;
    let prices_producer = services.stream_producer("prices_provider_prices", streamer::no_shutdown()).await?;
    let prices = services.price_jobs(assets_producer, prices_producer).await?;

    let mut builder = add_platform_jobs(ctx.plan_builder(WorkerService::Prices, config.as_ref(), shutdown_rx), &prices);
    for kind in prices.enabled_providers().to_vec() {
        builder = add_provider_jobs(builder, &prices, kind).await?;
    }
    builder.finish().await
}

fn add_platform_jobs<'a>(builder: JobPlanBuilder<'a>, prices: &PriceJobs) -> JobPlanBuilder<'a> {
    builder
        .job(WorkerJob::AggregateHourlyCharts, charts_job(prices, ChartsAction::Aggregate(ChartTimeframe::Hourly)))
        .job(WorkerJob::AggregateDailyCharts, charts_job(prices, ChartsAction::Aggregate(ChartTimeframe::Daily)))
        .job(WorkerJob::CleanupChartsRaw, charts_job(prices, ChartsAction::Delete(ChartTimeframe::Raw)))
        .job(WorkerJob::CleanupChartsHourly, charts_job(prices, ChartsAction::Delete(ChartTimeframe::Hourly)))
        .job(WorkerJob::UpdateObservedPrices, {
            let prices = prices.clone();
            move |_| {
                let prices = prices.clone();
                async move { prices.update_observed_prices().await }
            }
        })
        .job(WorkerJob::PublishMissingPrices, {
            let prices = prices.clone();
            move |_| {
                let publisher = prices.missing_prices_publisher();
                async move { publisher.update().await }
            }
        })
}

async fn add_provider_jobs<'a>(builder: JobPlanBuilder<'a>, prices: &PriceJobs, kind: PriceProvider) -> Result<JobPlanBuilder<'a>, Box<dyn Error + Send + Sync>> {
    let config = prices.config();
    let assets_limit = config.get_param_usize(&ConfigParamKey::PriceProviderAssetsLimit(kind)).await?;
    let mut builder = add_updater_job(
        builder,
        prices,
        kind,
        UpdaterProducer::Assets,
        WorkerJob::UpdatePricesAssets,
        ConfigParamKey::PriceProviderAssetsDuration(kind),
        move |updater| async move { updater.update_assets(assets_limit).await },
    )
    .await?;
    builder = add_updater_job(
        builder,
        prices,
        kind,
        UpdaterProducer::Assets,
        WorkerJob::UpdatePricesAssetsNew,
        ConfigParamKey::PriceProviderAssetsNewDuration(kind),
        |updater| async move { updater.update_assets_new().await },
    )
    .await?;
    builder = builder.job(JobVariant::labeled(WorkerJob::PublishPricesAssetsMetadata, kind), {
        let prices = prices.clone();
        move |_| {
            let prices = prices.clone();
            async move { prices.publish_assets_metadata(kind).await }
        }
    });

    let cleanup_variant = JobVariant::labeled(WorkerJob::CleanupOutdatedAssets, kind)
        .with_param_duration(&config, &ConfigParamKey::PriceProviderCleanOutdatedDuration(kind))
        .await?;
    builder = builder.job(cleanup_variant, {
        let prices = prices.clone();
        move |_| {
            let updater = prices.cleanup_updater(kind);
            async move { updater.update().await }
        }
    });

    let metrics_variant = JobVariant::labeled(WorkerJob::UpdatePricesMetrics, kind)
        .with_param_duration(&config, &ConfigParamKey::PriceProviderMetricsDuration(kind))
        .await?;
    builder = builder.job(metrics_variant, {
        let prices = prices.clone();
        move |_| {
            let updater = prices.metrics_updater(kind);
            async move { updater.update().await }
        }
    });

    let history_config = ChartsHistoryConfig {
        hourly_duration: config.get_param_duration(&ConfigParamKey::PriceProviderChartsHourlyDuration(kind)).await?,
    };
    builder = builder.job(JobVariant::labeled(WorkerJob::UpdateChartsHistory, kind), {
        let prices = prices.clone();
        move |_| {
            let updater = prices.charts_history_updater(kind, history_config);
            async move { updater.update().await }
        }
    });

    builder = match kind {
        PriceProvider::Coingecko => builder
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesTop, kind),
                updater_job(prices, kind, UpdaterProducer::Prices, |updater| async move { updater.update_prices_window(0, 500).await }),
            )
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesHigh, kind),
                updater_job(prices, kind, UpdaterProducer::Prices, |updater| async move { updater.update_prices_window(500, 2500).await }),
            )
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesLow, kind),
                updater_job(prices, kind, UpdaterProducer::Prices, |updater| async move { updater.update_prices_window(3000, usize::MAX).await }),
            )
            .job(WorkerJob::UpdateMarkets, {
                let prices = prices.clone();
                move |_| {
                    let updater = prices.markets_updater();
                    Box::pin(async move { updater.update_markets().await })
                }
            }),
        PriceProvider::Pyth | PriceProvider::Jupiter | PriceProvider::DefiLlama | PriceProvider::TonApi => {
            add_updater_job(
                builder,
                prices,
                kind,
                UpdaterProducer::Prices,
                WorkerJob::UpdatePrices,
                ConfigParamKey::PriceProviderPricesDuration(kind),
                |updater| async move { updater.update_prices_all().await },
            )
            .await?
        }
    };
    Ok(builder)
}

#[derive(Clone, Copy)]
enum UpdaterProducer {
    Assets,
    Prices,
}

async fn add_updater_job<'a, F, Fut>(
    builder: JobPlanBuilder<'a>,
    prices: &PriceJobs,
    kind: PriceProvider,
    producer: UpdaterProducer,
    job: WorkerJob,
    interval: ConfigParamKey,
    run: F,
) -> Result<JobPlanBuilder<'a>, Box<dyn Error + Send + Sync>>
where
    F: Fn(PricesUpdater) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<usize, Box<dyn Error + Send + Sync>>> + Send + 'static,
{
    let variant = JobVariant::labeled(job, kind).with_param_duration(&prices.config(), &interval).await?;
    Ok(builder.job(variant, updater_job(prices, kind, producer, run)))
}

fn updater_job<F, Fut>(prices: &PriceJobs, kind: PriceProvider, producer: UpdaterProducer, run: F) -> impl Fn(JobContext) -> JobFuture + Clone + Send + Sync + 'static
where
    F: Fn(PricesUpdater) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<usize, Box<dyn Error + Send + Sync>>> + Send + 'static,
{
    let prices = prices.clone();
    move |_| {
        let updater = match producer {
            UpdaterProducer::Assets => prices.assets_updater(kind),
            UpdaterProducer::Prices => prices.prices_updater(kind),
        };
        let run = run.clone();
        Box::pin(async move { run(updater).await })
    }
}

#[derive(Clone, Copy)]
enum ChartsAction {
    Aggregate(ChartTimeframe),
    Delete(ChartTimeframe),
}

fn charts_job(prices: &PriceJobs, action: ChartsAction) -> impl Fn(JobContext) -> JobFuture + Clone + Send + Sync + 'static {
    let prices = prices.clone();
    move |_| {
        let prices = prices.clone();
        Box::pin(async move {
            match action {
                ChartsAction::Aggregate(timeframe) => prices.aggregate_charts(timeframe).await,
                ChartsAction::Delete(timeframe) => prices.delete_expired_charts(timeframe).await,
            }
        })
    }
}
