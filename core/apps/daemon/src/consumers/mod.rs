pub mod fiat;
pub mod indexer;
pub mod notifications;
pub mod rewards;
pub mod runner;
pub mod store;
pub mod support;

use std::error::Error;

use settings::Settings;
use streamer::{ConsumerConfig, QueueName, ShutdownReceiver, StreamReader, StreamReaderConfig};

pub use fiat::run_consumer_fiat;
pub use indexer::run_consumer_indexer;
pub use rewards::run_consumer_rewards;
pub use store::run_consumer_store;
pub use support::run_consumer_support;

pub fn consumer_user_agent(name: &str) -> String {
    settings::service_user_agent("consumer", Some(name))
}

pub(crate) fn consumer_config(consumer: &settings::Consumer) -> ConsumerConfig {
    ConsumerConfig {
        timeout_on_error: consumer.error.timeout,
        skip_on_error: consumer.error.skip,
        delay: consumer.delay,
        retries: consumer.error.retries,
    }
}

pub(crate) fn reader_config(rabbitmq: &settings::RabbitMQ, name: String) -> StreamReaderConfig {
    let retry = streamer::Retry::new(rabbitmq.retry.delay, rabbitmq.retry.timeout);
    StreamReaderConfig::new(rabbitmq.url.clone(), name, rabbitmq.prefetch, retry)
}

pub(crate) async fn reader_for_queue(settings: &Settings, queue: &QueueName, shutdown_rx: &ShutdownReceiver) -> Result<(String, StreamReader), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let config = reader_config(&settings.rabbitmq, name.clone());
    let reader = StreamReader::new(config, shutdown_rx).await?.ok_or("shutdown during connect")?;
    Ok((name, reader))
}
