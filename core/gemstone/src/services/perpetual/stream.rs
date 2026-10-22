use std::sync::Arc;

use async_trait::async_trait;
use futures::lock::Mutex;
use gem_hypercore::{
    models::websocket::HyperliquidRequest,
    provider::{websocket_mapper::account_subscriptions, websocket_subscriptions::WebSocketSubscriptions},
};
use primitives::WalletId;
use primitives::chart::ChartCandleUpdate;

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

    pub async fn connected(&self, address: String) -> Result<(), GemServiceError> {
        let requests = self.subscriptions.lock().await.connected(account_subscriptions(address, &self.perpetual.perp_dexes));
        self.send(requests).await
    }

    pub async fn disconnected(&self) {
        self.subscriptions.lock().await.disconnected();
    }

    pub async fn subscribe(&self, subscription: GemPerpetualSubscription) -> Result<(), GemServiceError> {
        let requests = {
            let mut subscriptions = self.subscriptions.lock().await;
            subscription.map(&self.perpetual.perp_dexes).into_iter().flat_map(|subscription| subscriptions.subscribe(subscription)).collect()
        };
        self.send(requests).await
    }

    pub async fn unsubscribe(&self, subscription: GemPerpetualSubscription) -> Result<(), GemServiceError> {
        let requests = {
            let mut subscriptions = self.subscriptions.lock().await;
            subscription.map(&self.perpetual.perp_dexes).iter().flat_map(|subscription| subscriptions.unsubscribe(subscription)).collect()
        };
        self.send(requests).await
    }

    pub async fn candle_update(&self, wallet_id: WalletId, data: Vec<u8>) -> Result<Option<ChartCandleUpdate>, GemServiceError> {
        match self.perpetual.on_socket_message(wallet_id, data).await? {
            GemPerpetualSocketUpdate::Candle { candle } => Ok(Some(candle)),
            GemPerpetualSocketUpdate::Applied | GemPerpetualSocketUpdate::SubscriptionResponse { .. } | GemPerpetualSocketUpdate::Unknown => Ok(None),
            GemPerpetualSocketUpdate::Error { message } => Err(GemServiceError::Core { msg: message }),
        }
    }
}

impl GemPerpetualStreamService {
    async fn send(&self, requests: Vec<HyperliquidRequest>) -> Result<(), GemServiceError> {
        for request in requests {
            let message = serde_json::to_string(&request).map_err(GemServiceError::core)?;
            self.connection.send(message).await?;
        }
        Ok(())
    }
}
