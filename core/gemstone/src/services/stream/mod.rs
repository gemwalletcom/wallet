pub mod connection;
pub mod model;
pub mod rules;
pub mod subscription;
#[cfg(test)]
pub(crate) mod testkit;

pub use connection::GemStreamConnection;
pub use model::GemStreamEvent;
pub use subscription::GemStreamSubscriptionService;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use tracing::error;

use primitives::{Chain, StreamEvent, SupportMessageSender, SupportStreamEvent, SupportTypingStatus};

use crate::services::balance::GemBalanceService;
use crate::services::device::GemDeviceService;
use crate::services::fiat::GemFiatService;
use crate::services::nft::GemNftService;
use crate::services::notification::GemNotificationService;
use crate::services::perpetual::GemPerpetualService;
use crate::services::price::GemPriceService;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::support::GemSupportService;
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
    notifications: Arc<GemNotificationService>,
    support: Arc<GemSupportService>,
    subscriptions: Arc<GemStreamSubscriptionService>,
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
        notifications: Arc<GemNotificationService>,
        support: Arc<GemSupportService>,
        subscriptions: Arc<GemStreamSubscriptionService>,
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

    pub async fn update_session(&self) -> Result<(), GemServiceError> {
        self.subscriptions.prepare_session(self.session.get_current_wallet_id()?).await.map(|_| ())
    }

    pub async fn connected(&self) -> Result<(), GemServiceError> {
        self.subscriptions.reconnect().await
    }

    pub async fn disconnected(&self) {
        self.subscriptions.reset().await;
        if let Err(error) = self.support.clear_typing() {
            error!(%error, "stream support typing reset failed");
        }
    }

    pub async fn decode_event(&self, event: String) -> Result<GemStreamEvent, GemServiceError> {
        let event = serde_json::from_str(&event).map_err(|error| GemServiceError::InvalidInput { msg: error.to_string() })?;
        self.stream_event(event).await
    }

    pub async fn sync(&self, event: GemStreamEvent) -> Result<(), GemServiceError> {
        match event {
            GemStreamEvent::Prices { .. } | GemStreamEvent::Notification { .. } | GemStreamEvent::SupportMessage { .. } | GemStreamEvent::SupportTyping { .. } => Ok(()),
            GemStreamEvent::Balances { wallet_id, asset_ids } => self.balance.update(wallet_id, asset_ids).await,
            GemStreamEvent::Transactions { wallet_id, asset_ids, .. } => {
                let (transactions, balances) = futures::join!(self.transactions.sync_wallet(wallet_id.clone(), None), self.balance.sync_assets_and_update(wallet_id, asset_ids));
                transactions.and(balances)
            }
            GemStreamEvent::PriceAlerts { .. } => self.price_alert.sync(None).await,
            GemStreamEvent::Nft { wallet_id } => self.nft.sync_wallet(wallet_id).await.map(|_| ()),
            GemStreamEvent::Perpetual { wallet_id } => {
                let Some(wallet) = self.session.get_wallet(wallet_id.clone()).await? else {
                    return Ok(());
                };
                let Some(account) = rules::hyperliquid_account(&wallet.accounts) else {
                    return Ok(());
                };
                self.perpetual.sync_positions(wallet_id, Chain::HyperCore, account.address.clone()).await.map(|_| ())
            }
            GemStreamEvent::FiatTransaction { wallet_id } => self.fiat.sync_transactions(wallet_id).await,
        }
    }
}

impl GemStreamService {
    async fn stream_event(&self, event: StreamEvent) -> Result<GemStreamEvent, GemServiceError> {
        match event {
            StreamEvent::Prices(payload) => {
                let handled = GemStreamEvent::Prices {
                    prices: payload.prices.len() as u32,
                    rates: payload.rates.len() as u32,
                };
                self.price.update_rates_and_prices(payload.rates, payload.prices).await?;
                Ok(handled)
            }
            StreamEvent::Balances(update) => Ok(GemStreamEvent::Balances {
                wallet_id: update.wallet_id,
                asset_ids: update.asset_ids,
            }),
            StreamEvent::Transactions(update) => Ok(GemStreamEvent::Transactions {
                wallet_id: update.wallet_id,
                transaction_ids: update.transactions,
                asset_ids: update.asset_ids,
            }),
            StreamEvent::PriceAlerts(update) => Ok(GemStreamEvent::PriceAlerts { asset_ids: update.assets }),
            StreamEvent::Nft(update) => Ok(GemStreamEvent::Nft { wallet_id: update.wallet_id }),
            StreamEvent::Perpetual(update) => Ok(GemStreamEvent::Perpetual { wallet_id: update.wallet_id }),
            StreamEvent::InAppNotification(update) => {
                let wallet_id = update.wallet_id;
                self.notifications.save_notifications(vec![update.notification]).await?;
                Ok(GemStreamEvent::Notification { wallet_id })
            }
            StreamEvent::FiatTransaction(update) => Ok(GemStreamEvent::FiatTransaction { wallet_id: update.wallet_id }),
            StreamEvent::Support(SupportStreamEvent::Message(message)) => {
                let handled = GemStreamEvent::SupportMessage {
                    message_id: message.id.clone(),
                    images: message.images.len() as u32,
                };
                let from_agent = matches!(message.sender, SupportMessageSender::Agent(_));
                self.support.save_messages(vec![message]).await?;
                if from_agent {
                    self.support.clear_typing()?;
                }
                Ok(handled)
            }
            StreamEvent::Support(SupportStreamEvent::Typing(typing)) => {
                let is_typing = typing.status == SupportTypingStatus::On;
                self.support.update_typing(typing)?;
                Ok(GemStreamEvent::SupportTyping { is_typing })
            }
            StreamEvent::Error(detail) => Err(GemServiceError::Api { msg: detail.message }),
        }
    }
}
