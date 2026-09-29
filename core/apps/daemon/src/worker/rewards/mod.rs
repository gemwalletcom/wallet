use std::error::Error;

use job_runner::{JobHandle, ShutdownReceiver};

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(context: WorkerContext, shutdown: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = context.services();
    let config = services.config();
    let stream_producer = services.stream_producer("rewards_worker", shutdown.clone()).await?;
    let rewards = services.rewards_jobs(stream_producer);

    context
        .plan_builder(WorkerService::Rewards, &config, shutdown)
        .job(WorkerJob::CheckRewardsAbuse, {
            let rewards = rewards.clone();
            move |_| {
                let checker = rewards.abuse_checker();
                async move { checker.check().await }
            }
        })
        .job(WorkerJob::CheckRewardsEligibility, {
            let rewards = rewards.clone();
            move |_| {
                let checker = rewards.eligibility_checker();
                async move { checker.check().await }
            }
        })
        .finish()
        .await
}
