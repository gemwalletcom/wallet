use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigParamKey;
use job_runner::{JobHandle, ShutdownReceiver};
use primitives::ListProviderName;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = ctx.services();
    let config = services.config();
    let lists_client = Arc::new(services.lists());

    ctx.plan_builder(WorkerService::Lists, &config, shutdown_rx)
        .jobs_with_config(WorkerJob::UpdateLists, ListProviderName::all(), ConfigParamKey::ListProviderUpdateDuration, |provider, _| {
            let lists_client = lists_client.clone();
            move |_| {
                let lists_client = lists_client.clone();
                async move { lists_client.update_lists(provider).await }
            }
        })
        .finish()
        .await
}
