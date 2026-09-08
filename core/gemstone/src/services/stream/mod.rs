pub mod connection;
pub mod rules;
pub mod subscription;
#[cfg(test)]
pub(crate) mod testkit;

pub use connection::GemStreamConnection;
pub use subscription::GemStreamSubscriptionService;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use tracing::error;

use primitives::{Chain, StreamEvent, SupportMessageSender, SupportStreamEvent};

use crate::services::balance::GemBalanceService;
use crate::services::device::GemDeviceService;
use crate::services::fiat::GemFiatService;
use crate::services::nft::GemNftService;
use crate::services::notification::GemNotificationStore;
use crate::services::perpetual::GemPerpetualService;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::support::GemSupportStore;
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemStreamService {
    price: Arc<GemPriceService>,
    price_alert: Arc<GemPriceAlertService>,
    balance: Arc<GemBalanceService>,
    transactions: Arc<GemTransactionsService>,
    nft: Arc<GemNftService>,
    perpetual: Arc<GemPerpetualService>,
    fiat: Arc<GemFiatService>,
    notifications: Arc<dyn GemNotificationStore>,
    support: Arc<dyn GemSupportStore>,
    subscriptions: Arc<GemStreamSubscriptionService>,
    preferences: Arc<GemPreferencesService>,
    session: Arc<GemWalletSessionService>,
    device: Arc<GemDeviceService>,
}

#[uniffi::export]
impl GemStreamService {
    #[uniffi::constructor]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        price: Arc<GemPriceService>,
        price_alert: Arc<GemPriceAlertService>,
        balance: Arc<GemBalanceService>,
        transactions: Arc<GemTransactionsService>,
        nft: Arc<GemNftService>,
        perpetual: Arc<GemPerpetualService>,
        fiat: Arc<GemFiatService>,
        notifications: Arc<dyn GemNotificationStore>,
        support: Arc<dyn GemSupportStore>,
        subscriptions: Arc<GemStreamSubscriptionService>,
        preferences: Arc<GemPreferencesService>,
        session: Arc<GemWalletSessionService>,
        device: Arc<GemDeviceService>,
    ) -> Self {
        Self {
            price,
            price_alert,
            balance,
            transactions,
            nft,
            perpetual,
            fiat,
            notifications,
            support,
            subscriptions,
            preferences,
            session,
            device,
        }
    }

    pub async fn prepare_connection(&self) -> Result<bool, GemServiceError> {
        if !self.subscriptions.prepare_session(self.session.get_current_wallet_id()?).await? {
            return Ok(false);
        }
        if let Err(error) = self.device.synchronize_if_needed().await {
            error!(%error, "stream device synchronization failed");
        }
        Ok(true)
    }

    pub async fn connected(&self) -> Result<(), GemServiceError> {
        self.subscriptions.reconnect().await
    }

    pub async fn disconnected(&self) {
        self.subscriptions.reset().await;
    }

    pub async fn handle(&self, event: StreamEvent) -> Result<(), GemServiceError> {
        match event {
            StreamEvent::Prices(payload) => {
                let currency = self.preferences.get_currency();
                self.price.update_rates(payload.rates, currency.clone()).await?;
                self.price.update_prices(payload.prices, currency).await
            }
            StreamEvent::Balances(update) => self.balance.update(update.wallet_id, update.asset_ids).await,
            StreamEvent::Transactions(update) => {
                self.transactions.sync_wallet(update.wallet_id.clone(), None).await?;
                self.balance.update(update.wallet_id, update.asset_ids).await
            }
            StreamEvent::PriceAlerts(_) => self.price_alert.sync(None).await,
            StreamEvent::Nft(update) => self.nft.sync_wallet(update.wallet_id).await.map(|_| ()),
            StreamEvent::Perpetual(update) => {
                let Some(wallet) = self.session.get_wallet(update.wallet_id.clone()).await? else {
                    return Ok(());
                };
                let Some(account) = rules::hyperliquid_account(&wallet.accounts) else {
                    return Ok(());
                };
                self.perpetual.sync_positions(update.wallet_id, Chain::HyperCore, account.address.clone()).await.map(|_| ())
            }
            StreamEvent::InAppNotification(update) => self.notifications.save_notifications(vec![update.notification]).await,
            StreamEvent::FiatTransaction(update) => self.fiat.sync_transactions(update.wallet_id).await,
            StreamEvent::Support(SupportStreamEvent::Message(message)) => {
                let from_agent = matches!(message.sender, SupportMessageSender::Agent(_));
                self.support.save_messages(vec![message]).await?;
                if from_agent {
                    self.support.clear_typing()?;
                }
                Ok(())
            }
            StreamEvent::Support(SupportStreamEvent::Typing(typing)) => self.support.update_typing(typing),
        }
    }
}
