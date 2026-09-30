use std::error::Error;

use job_runner::{JobHandle, ShutdownReceiver};
use primitives::Chain;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;

pub async fn jobs(context: WorkerContext, shutdown: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let services = context.services();
    let config = services.config();
    let stream_producer = services.stream_producer("send_price_alerts", shutdown.clone()).await?;
    let alerter = services.alerter_jobs(stream_producer).await?;

    context
        .plan_builder(WorkerService::Alerter, &config, shutdown)
        .job(WorkerJob::AlertPriceAlerts, {
            let alerter = alerter.clone();
            move |_| {
                let sender = alerter.price_alert_sender();
                async move { sender.run_observer().await }
            }
        })
        .jobs(WorkerJob::AlertStakeRewards, Chain::stakeable(), |chain, _| {
            let alerter = alerter.clone();
            move |_| {
                let notifier = alerter.staking_rewards_notifier();
                async move { notifier.check_chain(chain).await }
            }
        })
        .finish()
        .await
}
