use std::error::Error;
use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{AssetId, Chain, NFTAssetId, PriceId, TransactionId, TransactionIdRequest};
use serde::Serialize;
use serde_json::Value;
use streamer::{
    ChainAddressPayload, ExchangeName, FetchAssetAssociationsPayload, FetchListPayload, FetchPricesPayload, FiatWebhookPayload, InAppNotificationPayload, NotificationsFailedPayload, NotificationsPayload, PricesPayload, QueueName,
    RewardsNotificationPayload, RewardsRedemptionPayload, StreamProducerQueue, SupportWebhookPayload, TransactionsPayload, WalletStreamPayload,
};

pub(crate) struct RecordingStreamProducer {
    published: Mutex<Vec<(QueueName, Value)>>,
    failure: Option<String>,
}

impl RecordingStreamProducer {
    pub(crate) fn new() -> Self {
        Self {
            published: Mutex::new(Vec::new()),
            failure: None,
        }
    }

    pub(crate) fn failing(message: &str) -> Self {
        Self {
            published: Mutex::new(Vec::new()),
            failure: Some(message.to_string()),
        }
    }

    pub(crate) fn published(&self) -> Vec<(QueueName, Value)> {
        self.published.lock().unwrap().clone()
    }

    fn record<T: Serialize>(&self, queue: QueueName, payload: &T) -> Result<bool, Box<dyn Error + Send + Sync>> {
        if let Some(message) = &self.failure {
            return Err(message.clone().into());
        }
        self.published.lock().unwrap().push((queue, serde_json::to_value(payload)?));
        Ok(true)
    }
}

#[async_trait]
impl StreamProducerQueue for RecordingStreamProducer {
    async fn publish_fetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchAssets, &asset_ids)
    }

    async fn publish_fetch_asset_status(&self, asset_id: AssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchAssetStatus, &asset_id)
    }

    async fn publish_fetch_asset_associations(&self, payload: FetchAssetAssociationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchAssetAssociations, &payload)
    }

    async fn publish_fetch_nft_asset(&self, asset_id: NFTAssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchNFTCollectionAssets, &asset_id)
    }

    async fn publish_fetch_nft_assets(&self, asset_ids: Vec<NFTAssetId>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchNFTCollectionAssets, &asset_ids)
    }

    async fn publish_fetch_prices(&self, payload: FetchPricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchPrices, &payload)
    }

    async fn publish_fetch_list(&self, payload: FetchListPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchLists, &payload)
    }

    async fn publish_fetch_prices_assets(&self, asset_ids: Vec<AssetId>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchPrices, &asset_ids)
    }

    async fn publish_fetch_transactions(&self, transactions: Vec<TransactionIdRequest>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchTransactions, &transactions)?;
        Ok(transactions.len())
    }

    async fn publish_transactions(&self, payload: TransactionsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::StoreTransactions, &payload)?;
        Ok(payload.transactions.len())
    }

    async fn publish_notifications_transactions(&self, payload: Vec<NotificationsPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::NotificationsTransactions, &payload)
    }

    async fn publish_notifications_price_alerts(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::NotificationsPriceAlerts, &payload)
    }

    async fn publish_notifications_observers(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::NotificationsObservers, &payload)
    }

    async fn publish_notifications_support(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::NotificationsSupport, &payload)
    }

    async fn publish_notifications_fiat_purchase(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::NotificationsFiatPurchase, &payload)
    }

    async fn publish_notifications_rewards(&self, payload: NotificationsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::NotificationsRewards, &payload)
    }

    async fn publish_rewards_events(&self, payload: Vec<RewardsNotificationPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::RewardsEvents, &payload)
    }

    async fn publish_rewards_redemption(&self, payload: RewardsRedemptionPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::RewardsRedemptions, &payload)
    }

    async fn publish_notifications_failed(&self, payload: NotificationsFailedPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::NotificationsFailed, &payload)
    }

    async fn publish_prices(&self, payload: PricesPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::StorePrices, &payload)
    }

    async fn publish_blocks(&self, chain: Chain, blocks: &[u64]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchBlocks, &(chain, blocks))?;
        Ok(())
    }

    async fn publish_new_addresses(&self, payload: Vec<ChainAddressPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        for queue in ExchangeName::NewAddresses.queues() {
            self.record(queue, &payload)?;
        }
        Ok(true)
    }

    async fn publish_in_app_notifications(&self, payload: Vec<InAppNotificationPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::NotificationsInApp, &payload)
    }

    async fn publish_wallet_stream_events(&self, payload: Vec<WalletStreamPayload>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::WalletStreamEvents, &payload)
    }

    async fn publish_support_webhook(&self, payload: SupportWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::SupportWebhooks, &payload)
    }

    async fn publish_pending_transaction(&self, transaction_id: TransactionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::StorePendingTransactions, &transaction_id)
    }

    async fn publish_fiat_webhook(&self, payload: FiatWebhookPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FiatOrderWebhooks, &payload)
    }

    async fn publish_referral_transaction(&self, queue: QueueName, transaction_id: TransactionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(queue, &transaction_id)
    }

    async fn publish_fetch_prices_metadata(&self, price_id: PriceId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.record(QueueName::FetchPricesMetadata, &price_id)
    }
}
