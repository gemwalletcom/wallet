use std::error::Error;
use std::sync::Arc;

use job_runner::{JobHandle, ShutdownReceiver};
use primitives::Chain;
use services::assets::PerpetualUpdater;
use settings::service_user_agent;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = ctx.services();
    let settings = services.settings();
    let config = services.config();
    let assets = services.assets_jobs().await?;

    ctx.plan_builder(WorkerService::Assets, &config, shutdown_rx)
        .job(WorkerJob::UpdateSuspiciousAssetRanks, {
            let assets = assets.clone();
            move |_| {
                let updater = assets.asset_rank_updater();
                async move { updater.update_suspicious_assets().await }
            }
        })
        .jobs(WorkerJob::UpdatePerpetuals, PerpetualUpdater::chains(), |chain, _| {
            let chain = *chain;
            let assets = assets.clone();
            move |_| {
                let updater = assets.perpetual_updater();
                async move { updater.update_chain(chain).await }
            }
        })
        .job(WorkerJob::UpdateUsageRanks, {
            let assets = assets.clone();
            move |_| {
                let updater = assets.usage_rank_updater();
                async move { updater.update_usage_ranks().await }
            }
        })
        .jobs(WorkerJob::UpdateAssetsImages, Chain::all(), |chain, _| {
            let assets = assets.clone();
            move |_| {
                let updater = assets.assets_images_updater();
                async move { updater.update_chain(chain).await }
            }
        })
        .job(WorkerJob::UpdateAssetsHasPrice, {
            let assets = assets.clone();
            move |_| {
                let updater = assets.assets_has_price_updater();
                async move { updater.update().await }
            }
        })
        .jobs(WorkerJob::UpdateStakeApy, Chain::stakeable(), {
            let services = services.clone();
            let assets = assets.clone();
            move |chain, _| {
                let providers = Arc::new(services.chain_providers_for(chain, &service_user_agent("daemon", Some("staking_apy"))));
                let assets = assets.clone();
                move |_| {
                    let updater = assets.stake_apy_updater(providers.clone());
                    async move { updater.update_chain(chain).await }
                }
            }
        })
        .jobs(WorkerJob::UpdateChainValidators, Chain::stakeable(), {
            let services = services.clone();
            let assets = assets.clone();
            move |chain, _| {
                let providers = Arc::new(services.chain_providers_for(chain, &service_user_agent("daemon", Some("scan_validators"))));
                let assets = assets.clone();
                move |_| {
                    let scanner = assets.validator_scanner(providers.clone());
                    async move { scanner.update_validators_for_chain(chain).await }
                }
            }
        })
        .jobs(WorkerJob::UpdateValidatorsFromStaticAssets, [Chain::Tron, Chain::SmartChain], {
            let services = services.clone();
            let assets = assets.clone();
            move |chain, _| {
                let providers = Arc::new(services.chain_providers_for(chain, &service_user_agent("daemon", Some("scan_static_assets"))));
                let assets_url = settings.assets.url.clone();
                let assets = assets.clone();
                move |_| {
                    let scanner = assets.validator_scanner(providers.clone());
                    let assets_url = assets_url.clone();
                    async move { scanner.update_validators_from_static_assets_for_chain(chain, &assets_url).await }
                }
            }
        })
        .finish()
        .await
}
