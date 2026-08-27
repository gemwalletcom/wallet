use std::collections::HashSet;
use std::sync::Arc;

use futures::lock::Mutex;

use primitives::{AssetId, StreamMessage, StreamMessagePrices, WalletId};

use super::connection::GemStreamConnection;
use super::rules;
use crate::services::error::GemServiceError;
use crate::services::price::GemPriceService;
use crate::services::price_alert::GemPriceAlertStore;

#[derive(Default)]
struct SubscriptionState {
    wallet_id: Option<WalletId>,
    subscribed: HashSet<AssetId>,
}

#[derive(uniffi::Object)]
pub struct GemStreamSubscriptionService {
    price: Arc<GemPriceService>,
    alerts: Arc<dyn GemPriceAlertStore>,
    connection: Arc<dyn GemStreamConnection>,
    state: Mutex<SubscriptionState>,
}

#[uniffi::export]
impl GemStreamSubscriptionService {
    #[uniffi::constructor]
    pub fn new(price: Arc<GemPriceService>, alerts: Arc<dyn GemPriceAlertStore>, connection: Arc<dyn GemStreamConnection>) -> Self {
        Self {
            price,
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
        let asset_ids = self.price.observable_asset_ids(wallet_id, alert_asset_ids).await?;
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
    use crate::alien::{AlienError, AlienProvider, AlienResponse, AlienTarget};
    use crate::api::GemApiClient;
    use crate::services::price::{GemPriceStore, GemPriceUpdate};
    use async_trait::async_trait;
    use primitives::currency::Currency;
    use primitives::{AssetMarket, Chain, FiatRate, PriceAlert};
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug)]
    struct NoopProvider;

    #[async_trait]
    impl AlienProvider for NoopProvider {
        async fn request(&self, _target: AlienTarget) -> Result<Arc<AlienResponse>, AlienError> {
            Err(AlienError::ResponseError { msg: "unused".into() })
        }

        fn get_endpoint(&self, _chain: Chain) -> Result<String, AlienError> {
            Ok("https://example.com".into())
        }
    }

    struct EnabledStore(Vec<AssetId>);

    #[async_trait]
    impl GemPriceStore for EnabledStore {
        async fn get_rate(&self, _currency: Currency) -> Result<Option<FiatRate>, GemServiceError> {
            Ok(None)
        }
        async fn get_enabled_price_asset_ids(&self, _wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
            Ok(self.0.clone())
        }
        async fn save_rates(&self, _rates: Vec<FiatRate>) -> Result<(), GemServiceError> {
            Ok(())
        }
        async fn save_prices(&self, _currency: Currency, _prices: Vec<GemPriceUpdate>) -> Result<(), GemServiceError> {
            Ok(())
        }
        async fn convert_prices(&self, _currency: Currency, _rate: f64) -> Result<(), GemServiceError> {
            Ok(())
        }
        async fn save_market(&self, _asset_id: AssetId, _market: AssetMarket) -> Result<(), GemServiceError> {
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
        async fn update(&self, _alerts: Vec<PriceAlert>, _delete_ids: Vec<String>) -> Result<(), GemServiceError> {
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

    fn service(connection: Arc<Connection>) -> GemStreamSubscriptionService {
        let price = GemPriceService::new(
            Arc::new(GemApiClient::new(Arc::new(NoopProvider), "https://example.com".into())),
            Arc::new(EnabledStore(vec![AssetId::from_chain(Chain::Bitcoin)])),
        );
        GemStreamSubscriptionService::new(Arc::new(price), Arc::new(AlertStore(vec![AssetId::from_chain(Chain::Bitcoin)])), connection)
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
            let service = service(connection.clone());
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
}
