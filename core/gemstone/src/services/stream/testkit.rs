use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use futures::channel::oneshot;
use primitives::{AssetId, Chain, PriceAlert, StreamMessage, WalletId};

use super::{GemStreamConnection, GemStreamSubscriptionService};
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::error::GemServiceError;
use crate::services::price_alert::testkit::MemoryPriceAlertStore;

pub struct SubscriptionTestkit {
    pub service: GemStreamSubscriptionService,
    pub balances: Arc<MemoryBalanceStore>,
    pub connection: Arc<MemoryStreamConnection>,
    pub wallet_id: WalletId,
}

impl SubscriptionTestkit {
    pub fn new(enabled: &[Chain], alerted: &[Chain]) -> Self {
        let wallet_id = WalletId::Multicoin("0x1".into());
        let balances = Arc::new(MemoryBalanceStore::with_enabled_asset_ids(wallet_id.clone(), asset_ids(enabled)));
        let connection = Arc::new(MemoryStreamConnection::default());
        connection.connected.store(true, Ordering::SeqCst);
        let service = GemStreamSubscriptionService::new(
            balances.clone(),
            Arc::new(MemoryPriceAlertStore::with_alerts(alerted.iter().map(|chain| PriceAlert::mock(*chain, None)).collect())),
            connection.clone(),
        );
        Self { service, balances, connection, wallet_id }
    }
}

pub fn asset_ids(chains: &[Chain]) -> Vec<AssetId> {
    chains.iter().copied().map(AssetId::from_chain).collect()
}

#[derive(Default)]
pub struct MemoryStreamConnection {
    pub connected: AtomicBool,
    pub latency: Mutex<Option<Duration>>,
    pub fail_next_send: AtomicBool,
    pause: Mutex<Option<oneshot::Receiver<()>>>,
    sent: Mutex<Vec<String>>,
}

impl MemoryStreamConnection {
    pub fn pause_next_send(&self) -> oneshot::Sender<()> {
        let (sender, receiver) = oneshot::channel();
        *self.pause.lock().unwrap() = Some(receiver);
        sender
    }

    pub fn messages(&self) -> Vec<(&'static str, Vec<AssetId>)> {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .filter_map(|message| serde_json::from_str::<StreamMessage>(message).ok())
            .map(|message| match message {
                StreamMessage::SubscribePrices(prices) => ("subscribe", prices.assets),
                StreamMessage::AddPrices(prices) => ("add", prices.assets),
                StreamMessage::GetPrices(prices) => ("get", prices.assets),
                StreamMessage::UnsubscribePrices(prices) => ("unsubscribe", prices.assets),
                StreamMessage::SubscribeRealtimePrices(prices) => ("subscribeRealtime", prices.assets),
                StreamMessage::UnsubscribeRealtimePrices(prices) => ("unsubscribeRealtime", prices.assets),
            })
            .collect()
    }
}

#[async_trait]
impl GemStreamConnection for MemoryStreamConnection {
    async fn latency(&self) -> Option<Duration> {
        *self.latency.lock().unwrap()
    }

    async fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    async fn send(&self, message: String) -> Result<(), GemServiceError> {
        let pause = self.pause.lock().unwrap().take();
        if let Some(pause) = pause {
            pause.await.unwrap();
        }
        if self.fail_next_send.swap(false, Ordering::SeqCst) {
            return Err(GemServiceError::Platform { msg: "send failed".into() });
        }
        self.sent.lock().unwrap().push(message);
        Ok(())
    }
}
