use std::error::Error;

use job_runner::{JobHandle, ShutdownReceiver};

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = ctx.services();
    let config = services.config();
    let stream_producer = services.stream_producer("rewards_worker", shutdown_rx.clone()).await?;
    let rewards = services.rewards_jobs(stream_producer);

    ctx.plan_builder(WorkerService::Rewards, &config, shutdown_rx)
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
