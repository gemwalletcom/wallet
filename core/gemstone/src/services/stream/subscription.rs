use std::collections::HashSet;
use std::sync::Arc;

use futures::lock::Mutex;

use primitives::{AssetId, StreamMessage, StreamMessagePrices, WalletId};

use super::connection::GemStreamConnection;
use super::rules;
use crate::models::asset::asset_ids_enabled_by_default;
use crate::services::balance::GemBalanceStore;
use crate::services::error::GemServiceError;
use crate::services::price::rules as price_rules;
use crate::services::price_alert::GemPriceAlertStore;

#[derive(Default)]
struct SubscriptionState {
    wallet_id: Option<WalletId>,
    subscribed: HashSet<AssetId>,
}

#[derive(uniffi::Object)]
pub struct GemStreamSubscriptionService {
    balances: Arc<dyn GemBalanceStore>,
    alerts: Arc<dyn GemPriceAlertStore>,
    connection: Arc<dyn GemStreamConnection>,
    state: Mutex<SubscriptionState>,
}

#[uniffi::export]
impl GemStreamSubscriptionService {
    #[uniffi::constructor]
    pub fn new(balances: Arc<dyn GemBalanceStore>, alerts: Arc<dyn GemPriceAlertStore>, connection: Arc<dyn GemStreamConnection>) -> Self {
        Self {
            balances,
            alerts,
            connection,
            state: Mutex::new(SubscriptionState::default()),
        }
    }

    pub async fn setup_assets(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.state.lock().await.wallet_id = Some(wallet_id);
        self.resubscribe().await
    }

    pub async fn resubscribe(&self) -> Result<(), GemServiceError> {
        let (wallet_id, subscribed) = {
            let state = self.state.lock().await;
            (state.wallet_id.clone(), state.subscribed.clone())
        };
        let Some(wallet_id) = wallet_id else {
            return Ok(());
        };
        if !self.connection.is_connected().await {
            return Ok(());
        }
        let alert_asset_ids = self.alerts.get_price_alerts(None).await?.into_iter().map(|alert| alert.asset_id).collect();
        let enabled_asset_ids = self.balances.get_enabled_asset_ids(wallet_id).await?;
        let asset_ids = price_rules::observable_asset_ids(enabled_asset_ids, alert_asset_ids, asset_ids_enabled_by_default());
        let target: HashSet<AssetId> = asset_ids.iter().cloned().collect();
        if subscribed == target {
            return Ok(());
        }
        self.connection.send(StreamMessage::SubscribePrices(StreamMessagePrices { assets: asset_ids })).await?;
        self.state.lock().await.subscribed = target;
        Ok(())
    }

    pub async fn add_prices(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let new_asset_ids = rules::new_asset_ids(&self.state.lock().await.subscribed, asset_ids);
        if new_asset_ids.is_empty() || !self.connection.is_connected().await {
            return Ok(());
        }
        self.connection
            .send(StreamMessage::AddPrices(StreamMessagePrices { assets: new_asset_ids.clone() }))
            .await?;
        self.state.lock().await.subscribed.extend(new_asset_ids);
        Ok(())
    }

    pub async fn reset(&self) {
        self.state.lock().await.subscribed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::balance::{GemAssetBalance, GemBalanceUpdate};
    use async_trait::async_trait;
    use primitives::currency::Currency;
    use primitives::{Chain, PriceAlert};
    use std::sync::atomic::{AtomicBool, Ordering};

    struct EnabledStore(Vec<AssetId>);

    #[async_trait]
    impl GemBalanceStore for EnabledStore {
        async fn get_available_balances(&self, _wallet_id: WalletId, _asset_ids: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError> {
            Ok(vec![])
        }
        async fn update_balances(&self, _wallet_id: WalletId, _updates: Vec<GemBalanceUpdate>) -> Result<(), GemServiceError> {
            Ok(())
        }
        async fn get_enabled_asset_ids(&self, _wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
            Ok(self.0.clone())
        }
        async fn set_assets_enabled(&self, _wallet_id: WalletId, _asset_ids: Vec<AssetId>, _enabled: bool) -> Result<(), GemServiceError> {
            Ok(())
        }
        async fn set_asset_pinned(&self, _wallet_id: WalletId, _asset_id: AssetId, _pinned: bool) -> Result<(), GemServiceError> {
            Ok(())
        }
    }

    struct AlertStore(Vec<AssetId>);

    #[async_trait]
    impl GemPriceAlertStore for AlertStore {
        async fn get_price_alerts(&self, _asset_id: Option<AssetId>) -> Result<Vec<PriceAlert>, GemServiceError> {
            Ok(self
                .0
                .iter()
                .map(|asset_id| PriceAlert {
                    asset_id: asset_id.clone(),
                    currency: Currency::USD,
                    price: None,
                    price_percent_change: None,
                    price_direction: None,
                    identifier: String::new(),
                    last_notified_at: None,
                })
                .collect())
        }
        async fn update_price_alerts(&self, _alerts: Vec<PriceAlert>, _delete_ids: Vec<String>) -> Result<(), GemServiceError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct Connection {
        connected: AtomicBool,
        sent: std::sync::Mutex<Vec<StreamMessage>>,
    }

    #[async_trait]
    impl GemStreamConnection for Connection {
        async fn is_connected(&self) -> bool {
            self.connected.load(Ordering::SeqCst)
        }
        async fn send(&self, message: StreamMessage) -> Result<(), GemServiceError> {
            self.sent.lock().unwrap().push(message);
            Ok(())
        }
    }

    fn service(connection: Arc<Connection>, enabled: Vec<AssetId>, alerts: Vec<AssetId>) -> GemStreamSubscriptionService {
        GemStreamSubscriptionService::new(Arc::new(EnabledStore(enabled)), Arc::new(AlertStore(alerts)), connection)
    }

    fn subscribed(connection: &Connection) -> Vec<Vec<AssetId>> {
        connection
            .sent
            .lock()
            .unwrap()
            .iter()
            .filter_map(|message| match message {
                StreamMessage::SubscribePrices(prices) => Some(prices.assets.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_subscribes_once_when_connected_and_after_reset() {
        futures::executor::block_on(async {
            let connection = Arc::new(Connection::default());
            let service = service(connection.clone(), vec![AssetId::from_chain(Chain::Bitcoin)], vec![AssetId::from_chain(Chain::Bitcoin)]);
            let wallet_id = WalletId::Multicoin("0x1".into());

            service.setup_assets(wallet_id.clone()).await.unwrap();
            assert!(subscribed(&connection).is_empty());

            connection.connected.store(true, Ordering::SeqCst);
            service.resubscribe().await.unwrap();
            service.setup_assets(wallet_id).await.unwrap();
            assert_eq!(subscribed(&connection).len(), 1);
            assert_eq!(subscribed(&connection)[0], vec![AssetId::from_chain(Chain::Bitcoin)]);

            service
                .add_prices(vec![AssetId::from_chain(Chain::Bitcoin), AssetId::from_chain(Chain::Ethereum)])
                .await
                .unwrap();
            assert!(matches!(connection.sent.lock().unwrap().last(), Some(StreamMessage::AddPrices(prices)) if prices.assets == vec![AssetId::from_chain(Chain::Ethereum)]));

            service.reset().await;
            service.resubscribe().await.unwrap();
            assert_eq!(subscribed(&connection).len(), 2);
        });
    }

    #[test]
    fn test_subscribes_to_alerted_assets_the_wallet_has_not_enabled() {
        futures::executor::block_on(async {
            let connection = Arc::new(Connection::default());
            let service = service(connection.clone(), vec![AssetId::from_chain(Chain::Bitcoin)], vec![AssetId::from_chain(Chain::Ethereum)]);
            connection.connected.store(true, Ordering::SeqCst);

            service.setup_assets(WalletId::Multicoin("0x1".into())).await.unwrap();

            assert_eq!(subscribed(&connection)[0], vec![AssetId::from_chain(Chain::Bitcoin), AssetId::from_chain(Chain::Ethereum)]);
        });
    }
}
