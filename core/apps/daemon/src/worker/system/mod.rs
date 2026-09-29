use std::error::Error;

use job_runner::{JobHandle, ShutdownReceiver};
use services::system::VersionUpdater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(context: WorkerContext, shutdown: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = context.services();
    let config = services.config();
    let stream_producer = services.stream_producer("observe_inactive_devices", shutdown.clone()).await?;
    let system = services.system_jobs(stream_producer).await?;

    context
        .plan_builder(WorkerService::System, &config, shutdown)
        .job(WorkerJob::CleanupProcessedTransactions, {
            let transaction_cleanup = system.transaction_cleanup();
            move |_| {
                let transaction_cleanup = transaction_cleanup.clone();
                async move { transaction_cleanup.cleanup().await }
            }
        })
        .job(WorkerJob::CleanupStaleDeviceSubscriptions, {
            let system = system.clone();
            move |_| {
                let device_updater = system.device_updater();
                async move { device_updater.update().await }
            }
        })
        .job(WorkerJob::ObserveInactiveDevices, {
            let system = system.clone();
            move |_| {
                let observer = system.inactive_devices_observer();
                async move { observer.observe().await }
            }
        })
        .jobs(WorkerJob::UpdateStoreVersion, VersionUpdater::stores(), |store, _| {
            let store = *store;
            let system = system.clone();
            move |_| {
                let updater = system.version_updater();
                async move { updater.update_store(store).await }
            }
        })
        .finish()
        .await
}
