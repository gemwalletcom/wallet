use std::collections::HashSet;
use std::sync::Arc;

use futures::lock::Mutex;

use primitives::{AssetId, StreamMessage, StreamMessagePrices, WalletId};

use super::connection::GemStreamConnection;
use super::rules;
use crate::models::asset::asset_ids_enabled_by_default;
use crate::services::balance::GemBalanceStore;
use crate::services::collections::unique;
use crate::services::error::GemServiceError;
use crate::services::price::rules as price_rules;
use crate::services::price_alert::GemPriceAlertStore;

#[derive(Default)]
struct SubscriptionState {
    wallet_id: Option<WalletId>,
    requested: Vec<AssetId>,
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
}

impl GemStreamSubscriptionService {
    pub(super) async fn prepare_session(&self, wallet_id: Option<WalletId>) -> Result<bool, GemServiceError> {
        match wallet_id {
            Some(wallet_id) => {
                self.setup_assets(wallet_id).await?;
                Ok(true)
            }
            None => {
                *self.state.lock().await = SubscriptionState::default();
                Ok(false)
            }
        }
    }

    async fn setup_assets(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let mut state = self.state.lock().await;
        if state.wallet_id.as_ref().is_some_and(|current| current != &wallet_id) {
            state.requested.clear();
            state.subscribed.clear();
        }
        state.wallet_id = Some(wallet_id);
        self.subscribe(&mut state).await
    }

    pub(super) async fn reconnect(&self) -> Result<(), GemServiceError> {
        let mut state = self.state.lock().await;
        state.subscribed.clear();
        self.subscribe(&mut state).await
    }

    pub(crate) async fn resubscribe(&self) -> Result<(), GemServiceError> {
        let mut state = self.state.lock().await;
        self.subscribe(&mut state).await
    }

    pub(crate) async fn add_prices(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        if asset_ids.is_empty() {
            return Ok(());
        }
        let mut state = self.state.lock().await;
        state.requested = unique(state.requested.iter().cloned().chain(asset_ids));
        if state.subscribed.is_empty() {
            return self.subscribe(&mut state).await;
        }
        let new_asset_ids = rules::new_asset_ids(&state.subscribed, state.requested.clone());
        if new_asset_ids.is_empty() || !self.connection.is_connected().await {
            return Ok(());
        }
        self.send(StreamMessage::AddPrices(StreamMessagePrices { assets: new_asset_ids.clone() })).await?;
        state.subscribed.extend(new_asset_ids);
        Ok(())
    }

    async fn send(&self, message: StreamMessage) -> Result<(), GemServiceError> {
        let message = serde_json::to_string(&message).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
        self.connection.send(message).await
    }

    pub(super) async fn reset(&self) {
        self.state.lock().await.subscribed.clear();
    }

    async fn subscribe(&self, state: &mut SubscriptionState) -> Result<(), GemServiceError> {
        if state.wallet_id.is_none() && state.requested.is_empty() {
            return Ok(());
        }
        if !self.connection.is_connected().await {
            return Ok(());
        }
        let alert_asset_ids = self
            .alerts
            .get_price_alerts(None)
            .await?
            .into_iter()
            .map(|alert| alert.asset_id)
            .chain(state.requested.iter().cloned())
            .collect();
        let enabled_asset_ids = match &state.wallet_id {
            Some(wallet_id) => self.balances.get_enabled_asset_ids(wallet_id.clone()).await?,
            None => vec![],
        };
        let asset_ids = price_rules::observable_asset_ids(enabled_asset_ids, alert_asset_ids, asset_ids_enabled_by_default());
        let target: HashSet<AssetId> = asset_ids.iter().cloned().collect();
        if state.subscribed == target {
            return Ok(());
        }
        self.send(StreamMessage::SubscribePrices(StreamMessagePrices { assets: asset_ids })).await?;
        state.subscribed = target;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use std::sync::atomic::Ordering;

    use futures::{executor::block_on, poll};
    use primitives::{Chain, WalletId};

    use crate::models::asset::asset_ids_enabled_by_default;
    use crate::services::balance::GemBalanceStore;
    use crate::services::stream::testkit::{SubscriptionTestkit, asset_ids};

    #[test]
    fn test_subscribes_to_enabled_and_alerted_assets_once() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[Chain::Bitcoin, Chain::Ethereum]);
            kit.service.setup_assets(kit.wallet_id.clone()).await.unwrap();
            kit.service.resubscribe().await.unwrap();
            kit.service.setup_assets(kit.wallet_id).await.unwrap();
            assert_eq!(kit.connection.messages(), vec![("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Ethereum]))]);
        });
    }

    #[test]
    fn test_each_reconnect_sends_a_fresh_subscription() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[Chain::Ethereum]);
            kit.connection.connected.store(false, Ordering::SeqCst);
            assert!(kit.service.prepare_session(Some(kit.wallet_id)).await.unwrap());

            kit.connection.connected.store(true, Ordering::SeqCst);
            kit.service.reconnect().await.unwrap();
            kit.service.reconnect().await.unwrap();

            assert_eq!(kit.connection.messages(), vec![("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Ethereum])); 2]);
        });
    }

    #[test]
    fn test_missing_session_clears_wallet_and_requested_subscriptions() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[]);
            assert!(kit.service.prepare_session(Some(kit.wallet_id.clone())).await.unwrap());
            kit.service.add_prices(asset_ids(&[Chain::Solana])).await.unwrap();

            assert!(!kit.service.prepare_session(None).await.unwrap());
            kit.service.reconnect().await.unwrap();
            assert!(kit.service.prepare_session(Some(kit.wallet_id.clone())).await.unwrap());
            assert!(!kit.service.prepare_session(None).await.unwrap());
            assert!(kit.service.prepare_session(Some(kit.wallet_id)).await.unwrap());

            assert_eq!(
                kit.connection.messages(),
                vec![
                    ("subscribe", asset_ids(&[Chain::Bitcoin])),
                    ("add", asset_ids(&[Chain::Solana])),
                    ("subscribe", asset_ids(&[Chain::Bitcoin])),
                    ("subscribe", asset_ids(&[Chain::Bitcoin])),
                ]
            );
        });
    }

    #[test]
    fn test_failed_reconnect_preserves_requested_assets_for_retry() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[]);
            kit.service.setup_assets(kit.wallet_id).await.unwrap();
            kit.service.add_prices(asset_ids(&[Chain::Solana])).await.unwrap();

            kit.connection.fail_next_send.store(true, Ordering::SeqCst);
            assert!(kit.service.reconnect().await.is_err());
            kit.service.resubscribe().await.unwrap();
            kit.service.resubscribe().await.unwrap();

            assert_eq!(
                kit.connection.messages(),
                vec![
                    ("subscribe", asset_ids(&[Chain::Bitcoin])),
                    ("add", asset_ids(&[Chain::Solana])),
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Solana])),
                ]
            );
        });
    }

    #[test]
    fn test_empty_wallet_subscribes_to_default_assets() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[], &[]);
            kit.service.setup_assets(kit.wallet_id).await.unwrap();
            assert_eq!(kit.connection.messages(), vec![("subscribe", asset_ids_enabled_by_default())]);
        });
    }

    #[test]
    fn test_initial_add_subscribes_before_setup_and_deduplicates_later_adds() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[]);
            kit.service.add_prices(asset_ids(&[Chain::Ethereum])).await.unwrap();
            kit.service.setup_assets(kit.wallet_id.clone()).await.unwrap();
            kit.service.add_prices(asset_ids(&[Chain::Ethereum, Chain::Solana, Chain::Solana])).await.unwrap();
            kit.service.add_prices(asset_ids(&[Chain::Solana])).await.unwrap();
            kit.service.resubscribe().await.unwrap();
            assert_eq!(
                kit.connection.messages(),
                vec![
                    ("subscribe", asset_ids(&[Chain::Ethereum])),
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Ethereum])),
                    ("add", asset_ids(&[Chain::Solana])),
                ]
            );
        });
    }

    #[test]
    fn test_offline_additions_survive_setup_and_reconnect() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[]);
            kit.connection.connected.store(false, Ordering::SeqCst);
            kit.service.add_prices(asset_ids(&[Chain::Solana])).await.unwrap();
            kit.service.setup_assets(kit.wallet_id.clone()).await.unwrap();
            kit.balances.set_assets_enabled(kit.wallet_id, asset_ids(&[Chain::Ethereum]), true).await.unwrap();
            kit.service.add_prices(asset_ids(&[Chain::Ethereum])).await.unwrap();
            assert_eq!(kit.connection.messages(), vec![]);

            kit.service.reset().await;
            kit.connection.connected.store(true, Ordering::SeqCst);
            kit.service.resubscribe().await.unwrap();
            kit.service.reset().await;
            kit.service.add_prices(asset_ids(&[Chain::Solana])).await.unwrap();
            assert_eq!(
                kit.connection.messages(),
                vec![("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Ethereum, Chain::Solana])); 2]
            );
        });
    }

    #[test]
    fn test_failed_sends_retain_requested_assets_for_retry() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[]);
            kit.connection.fail_next_send.store(true, Ordering::SeqCst);
            assert!(kit.service.add_prices(asset_ids(&[Chain::Ethereum])).await.is_err());
            kit.service.setup_assets(kit.wallet_id).await.unwrap();

            kit.connection.fail_next_send.store(true, Ordering::SeqCst);
            assert!(kit.service.add_prices(asset_ids(&[Chain::Solana])).await.is_err());
            kit.service.add_prices(asset_ids(&[Chain::Tron])).await.unwrap();
            kit.service.resubscribe().await.unwrap();
            assert_eq!(
                kit.connection.messages(),
                vec![
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Ethereum])),
                    ("add", asset_ids(&[Chain::Solana, Chain::Tron])),
                ]
            );
        });
    }

    #[test]
    fn test_wallet_change_removes_previous_wallet_additions() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[Chain::Tron]);
            let second_wallet = WalletId::Multicoin("0x2".into());
            kit.balances.set_assets_enabled(second_wallet.clone(), asset_ids(&[Chain::Ethereum]), true).await.unwrap();
            kit.service.setup_assets(kit.wallet_id.clone()).await.unwrap();
            kit.service.add_prices(asset_ids(&[Chain::Solana])).await.unwrap();
            kit.service.setup_assets(kit.wallet_id.clone()).await.unwrap();
            kit.service.setup_assets(second_wallet).await.unwrap();
            kit.service.setup_assets(kit.wallet_id).await.unwrap();
            assert_eq!(
                kit.connection.messages(),
                vec![
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Tron])),
                    ("add", asset_ids(&[Chain::Solana])),
                    ("subscribe", asset_ids(&[Chain::Ethereum, Chain::Tron])),
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Tron])),
                ]
            );
        });
    }

    #[test]
    fn test_enabled_asset_changes_preserve_requested_assets_and_alerts() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[Chain::Tron]);
            kit.service.setup_assets(kit.wallet_id.clone()).await.unwrap();
            kit.service.add_prices(asset_ids(&[Chain::Solana])).await.unwrap();

            kit.balances.set_assets_enabled(kit.wallet_id.clone(), asset_ids(&[Chain::Ethereum]), true).await.unwrap();
            kit.service.resubscribe().await.unwrap();
            kit.balances.set_assets_enabled(kit.wallet_id, asset_ids(&[Chain::Ethereum]), false).await.unwrap();
            kit.service.resubscribe().await.unwrap();

            assert_eq!(
                kit.connection.messages(),
                vec![
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Tron])),
                    ("add", asset_ids(&[Chain::Solana])),
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Ethereum, Chain::Tron, Chain::Solana])),
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Tron, Chain::Solana])),
                ]
            );
        });
    }

    #[test]
    fn test_add_waits_for_in_flight_subscription() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[]);
            let release = kit.connection.pause_next_send();
            let mut setup = pin!(kit.service.setup_assets(kit.wallet_id));
            assert!(poll!(setup.as_mut()).is_pending());
            let mut add = pin!(kit.service.add_prices(asset_ids(&[Chain::Ethereum])));
            assert!(poll!(add.as_mut()).is_pending());
            release.send(()).unwrap();
            setup.await.unwrap();
            add.await.unwrap();
            kit.service.resubscribe().await.unwrap();
            assert_eq!(
                kit.connection.messages(),
                vec![("subscribe", asset_ids(&[Chain::Bitcoin])), ("add", asset_ids(&[Chain::Ethereum]))]
            );
        });
    }

    #[test]
    fn test_reset_waits_for_in_flight_add_and_preserves_reconnect_assets() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[]);
            kit.service.setup_assets(kit.wallet_id).await.unwrap();
            let release = kit.connection.pause_next_send();
            let mut add = pin!(kit.service.add_prices(asset_ids(&[Chain::Ethereum])));
            assert!(poll!(add.as_mut()).is_pending());
            let mut reset = pin!(kit.service.reset());
            assert!(poll!(reset.as_mut()).is_pending());
            release.send(()).unwrap();
            add.await.unwrap();
            reset.await;
            kit.service.resubscribe().await.unwrap();
            assert_eq!(
                kit.connection.messages(),
                vec![
                    ("subscribe", asset_ids(&[Chain::Bitcoin])),
                    ("add", asset_ids(&[Chain::Ethereum])),
                    ("subscribe", asset_ids(&[Chain::Bitcoin, Chain::Ethereum])),
                ]
            );
        });
    }

    #[test]
    fn test_wallet_switch_waits_for_in_flight_setup() {
        block_on(async {
            let kit = SubscriptionTestkit::new(&[Chain::Bitcoin], &[]);
            let second_wallet = WalletId::Multicoin("0x2".into());
            kit.balances.set_assets_enabled(second_wallet.clone(), asset_ids(&[Chain::Ethereum]), true).await.unwrap();
            let release = kit.connection.pause_next_send();
            let mut first = pin!(kit.service.setup_assets(kit.wallet_id));
            assert!(poll!(first.as_mut()).is_pending());
            let mut second = pin!(kit.service.setup_assets(second_wallet));
            assert!(poll!(second.as_mut()).is_pending());
            release.send(()).unwrap();
            first.await.unwrap();
            second.await.unwrap();
            kit.service.resubscribe().await.unwrap();
            assert_eq!(
                kit.connection.messages(),
                vec![("subscribe", asset_ids(&[Chain::Bitcoin])), ("subscribe", asset_ids(&[Chain::Ethereum]))]
            );
        });
    }
}
