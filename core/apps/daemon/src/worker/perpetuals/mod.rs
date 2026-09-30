use std::error::Error;
use std::sync::Arc;

use job_runner::{JobHandle, ShutdownReceiver};
use primitives::Chain;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(context: WorkerContext, shutdown: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = context.services();
    let config = services.config();
    let stream_producer = services.stream_producer("perpetuals_worker", shutdown.clone()).await?;
    let perpetuals = services.perpetual_jobs(stream_producer).await?;
    let refresher = Arc::new(perpetuals.address_refresher());

    context
        .plan_builder(WorkerService::Perpetuals, &config, shutdown)
        .jobs(WorkerJob::ClassifyPerpetualAddresses, Chain::perpetual_chains(), |chain, _| {
            let classifier = Arc::new(perpetuals.classifier(chain));
            move |_| {
                let classifier = classifier.clone();
                async move { classifier.classify().await }
            }
        })
        .jobs(WorkerJob::ObservePerpetualActiveAddresses, Chain::perpetual_chains(), |chain, _| {
            let observer = Arc::new(perpetuals.observer(chain));
            move |_| {
                let observer = observer.clone();
                async move { observer.observe_active().await }
            }
        })
        .jobs(WorkerJob::ObservePerpetualPriorityAddresses, Chain::perpetual_chains(), |chain, _| {
            let observer = Arc::new(perpetuals.observer(chain));
            move |_| {
                let observer = observer.clone();
                async move { observer.observe_priority().await }
            }
        })
        .jobs(WorkerJob::RefreshPerpetualTrackedAddresses, Chain::perpetual_chains(), |chain, _| {
            let refresher = refresher.clone();
            move |_| {
                let refresher = refresher.clone();
                async move { refresher.update(chain).await }
            }
        })
        .finish()
        .await
}
