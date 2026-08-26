pub mod error;
pub mod rules;

use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Chain, StreamEvent, SupportStreamEvent};

use crate::services::balance::GemBalanceService;
use crate::services::fiat::GemFiatService;
use crate::services::nft::GemNftService;
use crate::services::notification::GemNotificationStore;
use crate::services::perpetual::GemPerpetualService;
use crate::services::price::GemPriceService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::subscription::GemWalletStore;
use crate::services::support::GemSupportStore;
use crate::services::transactions::GemTransactionsService;

pub use error::GemStreamError;

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
    wallet_store: Arc<dyn GemWalletStore>,
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
        wallet_store: Arc<dyn GemWalletStore>,
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
            wallet_store,
        }
    }

    pub async fn handle(&self, event: StreamEvent, currency: Currency) -> Result<(), GemStreamError> {
        match event {
            StreamEvent::Prices(payload) => {
                self.price.update_rates(payload.rates, currency.clone()).await.map_err(GemStreamError::service)?;
                self.price.update_prices(payload.prices, currency).await.map_err(GemStreamError::service)
            }
            StreamEvent::Balances(update) => self.balance.update(update.wallet_id, update.asset_ids).await.map_err(GemStreamError::service),
            StreamEvent::Transactions(update) => {
                self.transactions.sync(update.wallet_id.clone(), None).await.map_err(GemStreamError::service)?;
                self.balance.update(update.wallet_id, update.asset_ids).await.map_err(GemStreamError::service)
            }
            StreamEvent::PriceAlerts(_) => self.price_alert.sync(None).await.map_err(GemStreamError::service),
            StreamEvent::Nft(update) => self.nft.sync(update.wallet_id).await.map(|_| ()).map_err(GemStreamError::service),
            StreamEvent::Perpetual(update) => {
                let Some(wallet) = self.wallet_store.get_wallet(update.wallet_id.clone()).await.map_err(GemStreamError::service)? else {
                    return Ok(());
                };
                let Some(account) = rules::hyperliquid_account(&wallet.accounts) else {
                    return Ok(());
                };
                self.perpetual
                    .sync_positions(update.wallet_id, Chain::HyperCore, account.address.clone())
                    .await
                    .map_err(GemStreamError::service)
            }
            StreamEvent::InAppNotification(update) => self.notifications.save(vec![update.notification]).await.map_err(GemStreamError::service),
            StreamEvent::FiatTransaction(update) => self.fiat.sync_transactions(update.wallet_id).await.map_err(GemStreamError::service),
            StreamEvent::Support(SupportStreamEvent::Message(message)) => self.support.save_messages(vec![message]).await.map_err(GemStreamError::service),
            StreamEvent::Support(SupportStreamEvent::Typing(_)) => Ok(()),
        }
    }
}
