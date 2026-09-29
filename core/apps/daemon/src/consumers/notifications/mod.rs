use std::error::Error;
use std::sync::Arc;

use services::Services;
use settings::Settings;
use streamer::{ConsumerStatusReporter, InAppNotificationPayload, NotificationsFailedPayload, NotificationsPayload, QueueName, ShutdownReceiver, StreamReader, run_consumer};

use crate::consumers::{consumer_config, reader_config};

pub async fn run(settings: Settings, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let services = Services::new(Arc::new(settings))?;

    futures::future::try_join_all(vec![
        tokio::spawn(run_notification_consumer(services.clone(), QueueName::NotificationsPriceAlerts, shutdown.clone(), reporter.clone())),
        tokio::spawn(run_notification_consumer(services.clone(), QueueName::NotificationsTransactions, shutdown.clone(), reporter.clone())),
        tokio::spawn(run_notification_consumer(services.clone(), QueueName::NotificationsObservers, shutdown.clone(), reporter.clone())),
        tokio::spawn(run_notification_consumer(services.clone(), QueueName::NotificationsSupport, shutdown.clone(), reporter.clone())),
        tokio::spawn(run_notification_consumer(services.clone(), QueueName::NotificationsRewards, shutdown.clone(), reporter.clone())),
        tokio::spawn(run_notification_consumer(services.clone(), QueueName::NotificationsFiatPurchase, shutdown.clone(), reporter.clone())),
        tokio::spawn(run_notifications_failed_consumer(services.clone(), QueueName::NotificationsFailed, shutdown.clone(), reporter.clone())),
        tokio::spawn(run_in_app_notifications_consumer(services.clone(), shutdown.clone(), reporter.clone())),
    ])
    .await?;

    Ok(())
}

async fn queue_reader(services: &Services, name: &str, shutdown: &ShutdownReceiver) -> Result<StreamReader, Box<dyn Error + Send + Sync>> {
    Ok(StreamReader::new(reader_config(&services.settings().rabbitmq, name.to_string()), shutdown).await?.ok_or("shutdown during connect")?)
}

async fn run_notification_consumer(services: Services, queue: QueueName, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let stream_reader = queue_reader(&services, &name, &shutdown).await?;
    let consumer = services.notifications_consumer(&name, shutdown.clone()).await?;
    run_consumer::<NotificationsPayload, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&services.settings().consumer), shutdown, reporter).await
}

async fn run_notifications_failed_consumer(services: Services, queue: QueueName, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let stream_reader = queue_reader(&services, &name, &shutdown).await?;
    let consumer = services.notifications_failed_consumer();
    run_consumer::<NotificationsFailedPayload, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&services.settings().consumer), shutdown, reporter).await
}

async fn run_in_app_notifications_consumer(services: Services, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::NotificationsInApp;
    let name = queue.to_string();
    let stream_reader = queue_reader(&services, &name, &shutdown).await?;
    let consumer = services.in_app_notifications_consumer(&name, shutdown.clone()).await?;
    run_consumer::<InAppNotificationPayload, _, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&services.settings().consumer), shutdown, reporter).await
}
