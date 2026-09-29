use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigParamKey;
use job_runner::{JobHandle, ShutdownReceiver};
use primitives::ListProviderName;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(context: WorkerContext, shutdown: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = context.services();
    let config = services.config();
    let lists_client = Arc::new(services.lists());

    context
        .plan_builder(WorkerService::Lists, &config, shutdown)
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
