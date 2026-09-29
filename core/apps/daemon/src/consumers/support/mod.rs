use std::error::Error;
use std::sync::Arc;

use services::Services;
use settings::Settings;
use streamer::{ConsumerStatusReporter, QueueName, ShutdownReceiver, SupportWebhookPayload, run_consumer};

use crate::consumers::{consumer_config, reader_for_queue};

pub async fn run_consumer_support(settings: Settings, shutdown: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let services = Services::new(Arc::new(settings.clone()))?;
    let consumer = services.support_webhook_consumer(shutdown.clone()).await?;
    let queue = QueueName::SupportWebhooks;
    let (name, stream_reader) = reader_for_queue(&settings, &queue, &shutdown).await?;
    run_consumer::<SupportWebhookPayload, _, bool>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown, reporter).await
}
