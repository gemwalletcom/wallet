use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use fiat::FiatProvider;
use gem_tracing::{error_with_fields, info_with_fields};
use localizer::LanguageLocalizer;
use primitives::{FiatTransactionStatus, FiatWebhook, TransactionId};
use push_notification::{GorushNotification, PushNotification};
use storage::FiatTransactionRecord;
use streamer::consumer::MessageConsumer;
use streamer::{FiatWebhookPayload, NotificationsPayload, StreamProducerQueue, WalletStreamEvent, WalletStreamPayload};

use super::repository::{NotificationContext, Repository};
use crate::notifications::Pusher;

pub struct FiatWebhookConsumer {
    pub(crate) repository: Arc<dyn Repository>,
    pub providers: Vec<Box<dyn FiatProvider + Send + Sync>>,
    pub stream_producer: Arc<dyn StreamProducerQueue>,
}

impl FiatWebhookConsumer {
    pub(crate) fn new(repository: Arc<dyn Repository>, providers: Vec<Box<dyn FiatProvider + Send + Sync>>, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { repository, providers, stream_producer }
    }

    async fn send_fiat_notification(&self, updated: &FiatTransactionRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
        let NotificationContext { asset, wallet_id, devices } = self.repository.notification_context(updated.asset_id.clone(), updated.wallet_id).await?;

        let Some(crypto_value) = updated.value.as_deref() else {
            return Ok(());
        };
        let provider = updated.provider;
        let quote_type = updated.transaction_type;
        let notifications: Vec<GorushNotification> = devices
            .iter()
            .filter_map(|device| {
                let localizer = LanguageLocalizer::new_with_language(device.locale.as_ref());
                let message = Pusher::fiat_transaction_message(&localizer, &quote_type, provider.name(), &asset, crypto_value).ok()?;
                let data = PushNotification::new_fiat_transaction(wallet_id.clone(), asset.id.clone());
                GorushNotification::from_device(device.clone(), message.title, message.message.unwrap_or_default(), data)
            })
            .collect();

        self.stream_producer.publish_notifications_fiat_purchase(NotificationsPayload::new(notifications)).await?;
        Ok(())
    }
}

#[async_trait]
impl MessageConsumer<FiatWebhookPayload, bool> for FiatWebhookConsumer {
    async fn should_consume(&self, _payload: &FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        info_with_fields!("received webhook", provider = payload.provider.id());

        let provider = match self.providers.iter().find(|provider| provider.name() == payload.provider) {
            Some(provider) => provider,
            None => {
                info_with_fields!("ignoring webhook for unsupported provider", provider = payload.provider.id());
                return Ok(false);
            }
        };
        let provider_name = provider.name();
        let provider_id = provider_name.id();

        let transaction_update = match &payload.payload {
            FiatWebhook::OrderId(order_id) => {
                info_with_fields!("fetching order status", provider = provider_id, provider_transaction_id = order_id);
                match provider.get_order_status(order_id).await {
                    Ok(transaction) => transaction,
                    Err(error) => {
                        error_with_fields!("get_order_status", &*error, provider = provider_id, provider_transaction_id = order_id);
                        return Err(error);
                    }
                }
            }
            FiatWebhook::Transaction(transaction) => transaction.clone(),
            FiatWebhook::None => {
                info_with_fields!("ignoring webhook", provider = provider_id);
                return Ok(true);
            }
        };

        let (existing, updated) = self.repository.update_fiat_transaction(provider_name, transaction_update).await?;

        info_with_fields!(
            "processed webhook",
            provider = provider_id,
            provider_transaction_id = updated.provider_transaction_id.as_deref().unwrap_or(""),
            status = format!("{:?}", updated.status),
            quote_id = updated.quote_id.as_str(),
            transaction_hash = updated.transaction_hash.as_deref().unwrap_or("")
        );

        if updated.status == FiatTransactionStatus::Complete && !existing.is_some_and(|record| record.status == FiatTransactionStatus::Complete) {
            if let Some(hash) = &updated.transaction_hash {
                let transaction_id = TransactionId::new(updated.asset_id.chain, hash.clone());
                let _ = self.stream_producer.publish_pending_transaction(transaction_id.clone()).await;
                info_with_fields!("published fiat transaction to pending", provider = provider_id, transaction_id = transaction_id.to_string());
            }

            if let Err(error) = self.send_fiat_notification(&updated).await {
                error_with_fields!("send_fiat_notification", &*error, provider = provider_id);
            }
        }

        let _ = self
            .stream_producer
            .publish_wallet_stream_events(vec![WalletStreamPayload {
                wallet_id: updated.wallet_id,
                event: WalletStreamEvent::FiatTransaction,
            }])
            .await;

        Ok(true)
    }
}
