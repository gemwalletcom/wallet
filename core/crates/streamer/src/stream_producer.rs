use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use gem_tracing::info_with_fields;
use lapin::{BasicProperties, Channel, Confirmation, Connection, ConnectionProperties, ErrorKind, ExchangeKind, options::*, types::FieldTable};
use primitives::unix_seconds;
use tokio::sync::Mutex;

use crate::{ExchangeName, QueueName, Retry, ShutdownReceiver, StreamConnection, with_retry};

const ROUTING_KEY_EXCHANGE_SUFFIX: &str = "_exchange";

#[derive(Clone)]
pub struct StreamProducerConfig {
    url: String,
    retry: Retry,
    maxbytes: i64,
}

impl StreamProducerConfig {
    pub fn new(url: String, retry: Retry, maxbytes: i64) -> Self {
        Self { url, retry, maxbytes }
    }
}

fn queue_args(max_queue_bytes: i64) -> FieldTable {
    let mut args = FieldTable::default();
    args.insert("x-max-length-bytes".into(), max_queue_bytes.into());
    args
}

#[derive(Clone)]
pub struct StreamProducer {
    url: String,
    connection_name: String,
    retry: Retry,
    max_queue_bytes: i64,
    shutdown: ShutdownReceiver,
    channel: Arc<Mutex<Channel>>,
}

impl StreamProducer {
    pub async fn new(config: &StreamProducerConfig, connection_name: &str, shutdown: ShutdownReceiver) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let channel = with_retry(&config.retry, connection_name, &shutdown, || Self::try_connect(&config.url, connection_name))
            .await?
            .ok_or("shutdown during connect")?;
        Ok(Self {
            url: config.url.clone(),
            connection_name: connection_name.to_string(),
            retry: config.retry.clone(),
            max_queue_bytes: config.maxbytes,
            shutdown,
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    pub async fn from_connection(connection: &StreamConnection, max_queue_bytes: i64, shutdown: ShutdownReceiver) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let channel = Self::configure_channel(connection.create_channel().await?).await?;
        let retry = Retry::new(Duration::from_secs(1), Duration::from_secs(30));
        Ok(Self {
            url: connection.url().to_string(),
            connection_name: connection.name().to_string(),
            retry,
            max_queue_bytes,
            shutdown,
            channel: Arc::new(Mutex::new(channel)),
        })
    }

    async fn try_connect(url: &str, name: &str) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let options = ConnectionProperties::default().with_connection_name(name.to_string().into());
        let connection = Connection::connect(url, options).await?;
        Self::configure_channel(connection.create_channel().await?).await
    }

    async fn configure_channel(channel: Channel) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        channel.confirm_select(ConfirmSelectOptions::default()).await?;
        Ok(channel)
    }

    async fn reconnect(&self) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let mut channel = self.channel.lock().await;
        if channel.status().connected() {
            return Ok(channel.clone());
        }

        *channel = with_retry(&self.retry, &self.connection_name, &self.shutdown, || Self::try_connect(&self.url, &self.connection_name))
            .await?
            .ok_or("shutdown during reconnect")?;
        Ok(channel.clone())
    }

    async fn channel(&self) -> Result<Channel, Box<dyn Error + Send + Sync>> {
        let channel = self.channel.lock().await.clone();
        if channel.status().connected() {
            return Ok(channel);
        }
        self.reconnect().await
    }

    async fn run<T, F, Fut>(&self, mut operation: F) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        F: FnMut(Channel) -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error + Send + Sync>>>,
    {
        let mut delay = self.retry.delay;
        let mut attempt = 0;

        loop {
            if *self.shutdown.borrow() {
                return Err("shutdown during operation".into());
            }

            let channel = self.channel().await?;
            match operation(channel).await {
                Ok(value) => return Ok(value),
                Err(error) => {
                    attempt += 1;
                    info_with_fields!(
                        "rabbitmq producer retry",
                        connection = self.connection_name.as_str(),
                        attempt = attempt,
                        delay_secs = delay.as_secs(),
                        error = error.to_string()
                    );
                    let _ = self.reconnect().await;
                    let mut shutdown = self.shutdown.clone();
                    tokio::select! {
                        _ = tokio::time::sleep(delay) => {}
                        _ = shutdown.changed() => return Err("shutdown during operation".into()),
                    }
                    delay = next_delay(delay, &self.retry);
                }
            }
        }
    }

    pub async fn declare_queue(&self, name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.queue_exists(name).await? {
            return Ok(());
        }
        self.reconnect().await?;
        let max_queue_bytes = self.max_queue_bytes;
        self.run(|channel| async move {
            channel.queue_declare(name.into(), QueueDeclareOptions { durable: true, ..Default::default() }, queue_args(max_queue_bytes)).await?;
            Ok(())
        })
        .await
    }

    async fn queue_exists(&self, name: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let channel = self.channel().await?;
        match channel.queue_declare(name.into(), QueueDeclareOptions { passive: true, ..Default::default() }, FieldTable::default()).await {
            Ok(_) => Ok(true),
            Err(error) => {
                if let ErrorKind::ProtocolError(protocol_error) = error.kind()
                    && protocol_error.get_id() == 404
                {
                    return Ok(false);
                }
                Err(Box::new(error))
            }
        }
    }

    pub async fn declare_queues(&self, queues: Vec<QueueName>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for queue in queues {
            self.declare_queue(&queue.to_string()).await?;
        }
        Ok(())
    }

    pub async fn declare_exchange(&self, name: &str, kind: ExchangeKind) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.run(|channel| {
            let kind = kind.clone();
            async move {
                channel.exchange_declare(name.into(), kind, ExchangeDeclareOptions::default(), FieldTable::default()).await?;
                Ok(())
            }
        })
        .await
    }

    pub async fn declare_exchanges(&self, exchanges: Vec<ExchangeName>) -> Result<(), Box<dyn Error + Send + Sync>> {
        for exchange in exchanges {
            self.declare_exchange(&exchange.to_string(), exchange.kind()).await?;
        }
        Ok(())
    }

    pub async fn bind_queue(&self, queue: &str, exchange: &str, routing_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.run(|channel| async move {
            channel.queue_bind(queue.into(), exchange.into(), routing_key.into(), QueueBindOptions::default(), FieldTable::default()).await?;
            Ok(())
        })
        .await
    }

    pub async fn bind_queue_routing_key(&self, queue: QueueName, routing_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let exchange_name = format!("{}{}", queue, ROUTING_KEY_EXCHANGE_SUFFIX);
        let queue_name = format!("{}.{}", queue, routing_key);
        self.declare_queue(&queue_name).await?;
        self.bind_queue(&queue_name, &exchange_name, routing_key).await
    }

    async fn publish_message<T>(&self, exchange: &str, routing_key: &str, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let data = Arc::new(serde_json::to_vec(message)?);
        let published_at = unix_seconds()?;
        let confirmation = self
            .run(|channel| {
                let data = data.clone();
                async move {
                    let confirm = channel
                        .basic_publish(
                            exchange.into(),
                            routing_key.into(),
                            BasicPublishOptions { mandatory: true, ..Default::default() },
                            data.as_ref(),
                            BasicProperties::default().with_delivery_mode(2).with_content_type("application/json".into()).with_timestamp(published_at),
                        )
                        .await?;

                    Ok(confirm.await?)
                }
            })
            .await?;
        confirmed_publish(confirmation)
    }

    pub async fn publish<T>(&self, queue: QueueName, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        self.publish_message("", &queue.to_string(), message).await
    }

    pub async fn publish_batch<T>(&self, queue: QueueName, messages: &[T]) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let queue_name = queue.to_string();
        for message in messages {
            if !self.publish_message("", &queue_name, message).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn publish_to_exchange_with_routing_key<T>(&self, exchange: ExchangeName, routing_key: &str, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        self.publish_message(&exchange.to_string(), routing_key, message).await
    }

    pub async fn publish_with_routing_key<T>(&self, queue: QueueName, routing_key: &str, message: &T) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        T: serde::Serialize,
    {
        let exchange_name = format!("{}{}", queue, ROUTING_KEY_EXCHANGE_SUFFIX);
        self.publish_message(&exchange_name, routing_key, message).await
    }
}

fn next_delay(delay: Duration, retry: &Retry) -> Duration {
    (delay * 2).min(retry.timeout)
}

fn confirmed_publish(confirmation: Confirmation) -> Result<bool, Box<dyn Error + Send + Sync>> {
    match confirmation {
        Confirmation::Ack(None) => Ok(true),
        Confirmation::Ack(Some(message)) | Confirmation::Nack(Some(message)) => Err(format!(
            "rabbitmq rejected publish: {} {} (exchange={}, routing_key={})",
            message.reply_code, message.reply_text, message.delivery.exchange, message.delivery.routing_key
        )
        .into()),
        Confirmation::Nack(None) => Err("rabbitmq negatively acknowledged publish".into()),
        Confirmation::NotRequested => Err("rabbitmq publisher confirms are not enabled".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmed_publish_requires_broker_ack() {
        assert!(confirmed_publish(Confirmation::Ack(None)).is_ok());
        assert!(confirmed_publish(Confirmation::Nack(None)).is_err());
        assert!(confirmed_publish(Confirmation::NotRequested).is_err());
    }
}
