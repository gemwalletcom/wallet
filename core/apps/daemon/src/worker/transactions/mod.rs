use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigParamKey;
use job_runner::{JobHandle, ShutdownReceiver};
use primitives::Chain;
use settings::service_user_agent;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(context: WorkerContext, shutdown: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = context.services();
    let config = services.config();
    let stream_producer = services.stream_producer("transactions_worker", shutdown.clone()).await?;
    let transactions = services.transaction_jobs(stream_producer, context.transaction_metrics()).await?;
    let cacher = services.cacher().await?;
    let fee_estimates = Arc::new(services.fee_estimates(services.assets(), services.prices(cacher.clone()), cacher, &service_user_agent("daemon", Some("fee_estimates"))));

    context
        .plan_builder(WorkerService::Transactions, &config, shutdown)
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
        .jobs_with_config(WorkerJob::UpdateSwapVaultAddresses, transactions.vault_address_providers(), ConfigParamKey::SwapperVaultAddresses, |provider, _| {
            let updater = Arc::new(transactions.vault_addresses_updater());
            move |_| {
                let updater = updater.clone();
                async move { updater.update(provider).await }
            }
        })
        .jobs_with_config(WorkerJob::UpdateFeeEstimates, Chain::all(), ConfigParamKey::TransactionsFeeEstimatesUpdateDuration, |chain, _| {
            let fee_estimates = fee_estimates.clone();
            move |_| {
                let fee_estimates = fee_estimates.clone();
                async move { fee_estimates.update_chain_fee_estimates(chain).await }
            }
        })
        .finish()
        .await
}
