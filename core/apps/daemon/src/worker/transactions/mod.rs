use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigParamKey;
use job_runner::{JobHandle, ShutdownReceiver};
use primitives::SwapProvider;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = ctx.services();
    let config = services.config();
    let stream_producer = services.stream_producer("transactions_worker", shutdown_rx.clone()).await?;
    let transactions = services.transaction_jobs(stream_producer).await?;

    ctx.plan_builder(WorkerService::Transactions, &config, shutdown_rx)
        .job(WorkerJob::UpdateInTransitTransactions, {
            let updater = transactions.in_transit_updater();
            move |_| {
                let updater = updater.clone();
                async move { updater.update().await }
            }
        })
        .job(WorkerJob::UpdatePendingTransactions, {
            let updater = transactions.pending_updater();
            move |_| {
                let updater = updater.clone();
                async move { updater.update().await }
            }
        })
        .jobs_with_config(WorkerJob::UpdateSwapVaultAddresses, SwapProvider::cross_chain_providers(), ConfigParamKey::SwapperVaultAddresses, |provider, _| {
            let updater = Arc::new(transactions.vault_addresses_updater());
            move |ctx| {
                let updater = updater.clone();
                async move { updater.update(provider, ctx.last_success_at).await }
            }
        })
        .finish()
        .await
}
