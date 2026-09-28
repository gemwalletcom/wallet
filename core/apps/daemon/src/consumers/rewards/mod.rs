use std::error::Error;
use std::sync::Arc;

use primitives::rewards::RedemptionStatus;
use services::Services;
use settings::Settings;
use streamer::{ConsumerStatusReporter, QueueName, RewardsNotificationPayload, RewardsRedemptionPayload, ShutdownReceiver, run_consumer};

use crate::consumers::{consumer_config, reader_for_queue};

pub async fn run_consumer_rewards(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let services = Services::new(Arc::new(settings))?;

    futures::future::try_join_all(vec![
        tokio::spawn(run_rewards_events(services.clone(), shutdown_rx.clone(), reporter.clone())),
        tokio::spawn(run_rewards_redemptions(services.clone(), shutdown_rx.clone(), reporter.clone())),
    ])
    .await?;

    Ok(())
}

async fn run_rewards_events(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::RewardsEvents;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown_rx).await?;
    let consumer = services.rewards_consumer(&name, shutdown_rx.clone()).await?;
    run_consumer::<RewardsNotificationPayload, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

async fn run_rewards_redemptions(services: Services, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = services.settings();
    let queue = QueueName::RewardsRedemptions;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown_rx).await?;
    let consumer = services.rewards_redemption_consumer(&name, shutdown_rx.clone()).await?;
    run_consumer::<RewardsRedemptionPayload, _, RedemptionStatus>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}
