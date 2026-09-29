use std::{
    error::Error,
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{ConsumerStatus, ConsumerStatusReporter, QueueName, ShutdownReceiver, StreamMessage, StreamReader};
use async_trait::async_trait;
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::unix_seconds;
use serde::Deserialize;
use tokio;

#[derive(Clone)]
pub struct ConsumerConfig {
    pub timeout_on_error: Duration,
    pub skip_on_error: bool,
    pub delay: Duration,
    pub retries: u32,
}

enum ConsumeResult<R> {
    Consumed(R),
    Skipped,
    Error(Box<dyn Error + Send + Sync>),
}

#[async_trait]
pub trait MessageConsumer<P, R> {
    async fn consume(&self, payload: P) -> Result<R, Box<dyn Error + Send + Sync>>;
    async fn should_consume(&self, payload: &P) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

pub async fn run_consumer<P, C, R>(
    name: &str,
    mut stream_reader: StreamReader,
    queue_name: QueueName,
    routing_key: Option<&str>,
    consumer: C,
    config: ConsumerConfig,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    P: Send + Display + 'static,
    C: MessageConsumer<P, R> + Send + 'static,
    R: std::fmt::Debug,
    for<'a> P: Deserialize<'a> + std::fmt::Debug,
{
    if routing_key.is_none() {
        info_with_fields!("running consumer", consumer = queue_name.to_string());
    }
    stream_reader
        .read::<P, _, _>(queue_name, routing_key, |message| consume_message(name, &consumer, &config, &reporter, message), shutdown_rx)
        .await
}

async fn consume_message<P, C, R>(name: &str, consumer: &C, config: &ConsumerConfig, reporter: &Arc<dyn ConsumerStatusReporter>, message: StreamMessage<P>) -> Result<(), Box<dyn Error + Send + Sync>>
where
    P: Send + Display + 'static,
    C: MessageConsumer<P, R> + Send + 'static,
    R: std::fmt::Debug,
    for<'a> P: Deserialize<'a> + std::fmt::Debug,
{
    let StreamMessage { payload, published_at } = message;
    let payload_display = payload.to_string();
    info_with_fields!("processing", consumer = name, payload = payload_display.as_str());
    let start = Instant::now();
    let queue_wait = match published_at {
        Some(timestamp) => Some(unix_seconds()?.saturating_sub(timestamp)),
        None => None,
    };
    reporter.report(name, ConsumerStatus::Started { queue_wait_seconds: queue_wait }).await;
    let result = match consumer.should_consume(&payload).await {
        Ok(true) => match consumer.consume(payload).await {
            Ok(r) => ConsumeResult::Consumed(r),
            Err(e) => ConsumeResult::Error(e),
        },
        Ok(false) => ConsumeResult::Skipped,
        Err(e) => ConsumeResult::Error(e),
    };

    match result {
        ConsumeResult::Consumed(value) => {
            let duration = start.elapsed().as_millis() as u64;
            let result_str = format!("{:?}", value);
            info_with_fields!("processed", consumer = name, payload = payload_display.as_str(), result = result_str, elapsed = DurationMs(start.elapsed()));
            reporter.report(name, ConsumerStatus::Success { duration_milliseconds: duration }).await;
            if !config.delay.is_zero() {
                tokio::time::sleep(config.delay).await;
            }
            Ok(())
        }
        ConsumeResult::Skipped => {
            info_with_fields!("skipped", consumer = name, payload = payload_display.as_str(), elapsed = DurationMs(start.elapsed()));
            reporter
                .report(
                    name,
                    ConsumerStatus::Skipped {
                        duration_milliseconds: start.elapsed().as_millis() as u64,
                    },
                )
                .await;
            Ok(())
        }
        ConsumeResult::Error(e) => {
            error_with_fields!("failed", &*e, consumer = name, payload = payload_display.as_str(), elapsed = DurationMs(start.elapsed()));
            reporter
                .report(
                    name,
                    ConsumerStatus::Error {
                        duration_milliseconds: start.elapsed().as_millis() as u64,
                    },
                )
                .await;
            if !config.timeout_on_error.is_zero() {
                tokio::time::sleep(config.timeout_on_error).await;
            }
            if config.skip_on_error { Ok(()) } else { Err(e) }
        }
    }
}
