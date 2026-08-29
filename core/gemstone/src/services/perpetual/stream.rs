use std::sync::Arc;

use async_trait::async_trait;
use futures::lock::Mutex;
use gem_hypercore::{
    models::websocket::HyperliquidRequest,
    provider::{websocket_mapper::account_subscriptions, websocket_subscriptions::WebSocketSubscriptions},
};
use primitives::chart::ChartCandleUpdate;
use primitives::{PerpetualAccountMode, WalletId};

use super::GemPerpetualService;
use super::model::GemPerpetualSocketUpdate;
use crate::models::perpetual::GemPerpetualSubscription;
use crate::services::error::GemServiceError;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemPerpetualStreamConnection: Send + Sync {
    async fn send(&self, message: String) -> Result<(), GemServiceError>;
}

#[derive(uniffi::Object)]
pub struct GemPerpetualStreamService {
    perpetual: Arc<GemPerpetualService>,
    connection: Arc<dyn GemPerpetualStreamConnection>,
    subscriptions: Mutex<WebSocketSubscriptions>,
}

#[uniffi::export]
impl GemPerpetualStreamService {
    #[uniffi::constructor]
    pub fn new(perpetual: Arc<GemPerpetualService>, connection: Arc<dyn GemPerpetualStreamConnection>) -> Self {
        Self {
            perpetual,
            connection,
            subscriptions: Mutex::new(WebSocketSubscriptions::new()),
        }
    }

    pub async fn connected(&self, address: String, mode: PerpetualAccountMode) -> Result<(), GemServiceError> {
        let requests = self.subscriptions.lock().await.connected(account_subscriptions(address, mode));
        self.send(requests).await
    }

    pub async fn disconnected(&self) {
        self.subscriptions.lock().await.disconnected();
    }

    pub async fn subscribe(&self, subscription: GemPerpetualSubscription) -> Result<(), GemServiceError> {
        let requests = self.subscriptions.lock().await.subscribe(subscription.map());
        self.send(requests).await
    }

    pub async fn unsubscribe(&self, subscription: GemPerpetualSubscription) -> Result<(), GemServiceError> {
        let requests = self.subscriptions.lock().await.unsubscribe(&subscription.map());
        self.send(requests).await
    }

    pub async fn handle(&self, wallet_id: WalletId, mode: PerpetualAccountMode, data: Vec<u8>) -> Result<Option<ChartCandleUpdate>, GemServiceError> {
        match self.perpetual.apply_socket_message(wallet_id, mode, data).await? {
            GemPerpetualSocketUpdate::Candle { candle } => Ok(Some(candle)),
            GemPerpetualSocketUpdate::Applied | GemPerpetualSocketUpdate::SubscriptionResponse { .. } | GemPerpetualSocketUpdate::Unknown => Ok(None),
            GemPerpetualSocketUpdate::Error { message } => Err(GemServiceError::Core { msg: message }),
        }
    }
}

impl GemPerpetualStreamService {
    async fn send(&self, requests: Vec<HyperliquidRequest>) -> Result<(), GemServiceError> {
        for request in requests {
            let message = serde_json::to_string(&request).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
            self.connection.send(message).await?;
        }
        Ok(())
    }
}
