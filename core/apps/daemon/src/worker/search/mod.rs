use std::error::Error;

use job_runner::{JobHandle, ShutdownReceiver};

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = ctx.services();
    let config = services.config();
    let search = services.search_jobs().await?;

    ctx.plan_builder(WorkerService::Search, &config, shutdown_rx)
        .job(WorkerJob::UpdateAssetsIndex, {
            let search = search.clone();
            move |_| {
                let updater = search.assets_index_updater();
                async move { updater.update().await }
            }
        })
        .job(WorkerJob::UpdateAssetListsIndex, {
            let search = search.clone();
            move |_| {
                let updater = search.asset_lists_index_updater();
                async move { updater.update().await }
            }
        })
        .job(WorkerJob::UpdatePerpetualsIndex, {
            let search = search.clone();
            move |_| {
                let updater = search.perpetuals_index_updater();
                async move { updater.update().await }
            }
        })
        .job(WorkerJob::UpdateNftsIndex, {
            let search = search.clone();
            move |_| {
                let updater = search.nfts_index_updater();
                async move { updater.update().await }
            }
        })
        .finish()
        .await
}
