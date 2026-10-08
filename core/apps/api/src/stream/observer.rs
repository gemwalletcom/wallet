use std::error::Error;

use axum::extract::ws::{Message, WebSocket};
use gem_tracing::info_with_fields;
use primitives::response::ErrorDetail;
use primitives::{AssetPrice, StreamEvent, StreamMessage, Version, device_stream_channel};
use redis::PushInfo;
use redis::aio::MultiplexedConnection;
use services::prices::PriceClient;
use std::sync::Arc;

use super::price_handler::PriceHandler;
use super::redis::decode_push_message;

pub struct StreamObserver {
    device_id: String,
    device_channel: String,
    price_handler: PriceHandler,
}

impl StreamObserver {
    pub fn new(device_id: String, version: Version, price_client: Arc<PriceClient>) -> Self {
        let device_channel = device_stream_channel(&device_id);
        Self {
            device_id,
            device_channel,
            price_handler: PriceHandler::new(price_client, version),
        }
    }

    pub async fn next_price_interval(&mut self) {
        self.price_handler.next_interval().await;
    }

    pub fn take_prices(&mut self) -> Vec<AssetPrice> {
        self.price_handler.take_prices()
    }

    pub async fn subscribe_device_channel(&self, redis_connection: &mut MultiplexedConnection) -> Result<(), Box<dyn Error + Send + Sync>> {
        redis_connection.subscribe(&self.device_channel).await?;
        Ok(())
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub async fn respond_to_ws_message(&mut self, message: Message, redis_connection: &mut MultiplexedConnection, socket: &mut WebSocket) -> Result<(), Box<dyn Error + Send + Sync>> {
        match message {
            Message::Binary(data) => self.respond_to_message_payload(&data, redis_connection, socket).await,
            Message::Text(text) => self.respond_to_message_payload(text.as_bytes(), redis_connection, socket).await,
            Message::Ping(data) => Ok(socket.send(Message::Pong(data)).await?),
            Message::Close(_) => {
                info_with_fields!("websocket client closed connection gracefully", status = "ok");
                Ok(())
            }
            Message::Pong(_) => Ok(()),
        }
    }

    async fn respond_to_message_payload(&mut self, data: &[u8], redis_connection: &mut MultiplexedConnection, socket: &mut WebSocket) -> Result<(), Box<dyn Error + Send + Sync>> {
        let message = match serde_json::from_slice::<StreamMessage>(data) {
            Ok(message) => message,
            Err(error) => {
                info_with_fields!("websocket rejected unreadable message", device_id = self.device_id.as_str(), error = error.to_string());
                let detail = ErrorDetail { message: error.to_string(), data: None };
                return self.send_event(socket, StreamEvent::Error(detail)).await;
            }
        };
        if let Some(event) = self.price_handler.respond_to_stream_message(&message, redis_connection).await? {
            self.send_event(socket, event).await?;
        }
        Ok(())
    }

    pub fn receive_redis_message(&mut self, message: &PushInfo) -> Result<Option<StreamEvent>, Box<dyn Error + Send + Sync>> {
        let Some((channel, value)) = decode_push_message(message) else {
            return Ok(None);
        };

        if channel == self.device_channel {
            Ok(Some(serde_json::from_slice::<StreamEvent>(value)?))
        } else {
            self.price_handler.record_price_message(value)?;
            Ok(None)
        }
    }

    pub async fn send_event(&self, socket: &mut WebSocket, event: StreamEvent) -> Result<(), Box<dyn Error + Send + Sync>> {
        let text = serde_json::to_string(&event)?;
        Ok(socket.send(Message::Text(text.into())).await?)
    }
}
