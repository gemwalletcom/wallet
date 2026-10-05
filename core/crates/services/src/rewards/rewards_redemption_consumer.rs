use async_trait::async_trait;
use gem_tracing::info_with_fields;
use primitives::rewards::RedemptionStatus;
use primitives::{NotificationRewardsRedeemMetadata, NotificationType, TransactionId};
use rewards::{RedemptionAsset, RedemptionRequest, RedemptionService};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use storage::RedemptionUpdate;
use streamer::consumer::MessageConsumer;
use streamer::{InAppNotificationPayload, RewardsRedemptionPayload, StreamProducerQueue};

use super::repository::{RedemptionStart, Repository};

pub struct RedemptionRetryConfig {
    pub max_retries: u32,
    pub delay: Duration,
    pub errors: Vec<String>,
}

pub struct RewardsRedemptionConsumer<S: RedemptionService> {
    repository: Arc<dyn Repository>,
    redemption_service: Arc<S>,
    retry_config: RedemptionRetryConfig,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl<S: RedemptionService> RewardsRedemptionConsumer<S> {
    pub(crate) fn new(repository: Arc<dyn Repository>, redemption_service: Arc<S>, retry_config: RedemptionRetryConfig, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self {
            repository,
            redemption_service,
            retry_config,
            stream_producer,
        }
    }

    async fn redeem_with_retry(&self, request: RedemptionRequest) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut attempt = 0;
        loop {
            match self.redemption_service.process_redemption(request.clone()).await {
                Ok(result) => return Ok(result.transaction_id),
                Err(error) => {
                    let is_retryable = self.retry_config.errors.iter().any(|p| error.to_string().contains(p));
                    if attempt < self.retry_config.max_retries && is_retryable {
                        attempt += 1;
                        tokio::time::sleep(self.retry_config.delay).await;
                        continue;
                    }
                    return Err(error);
                }
            }
        }
    }
}

#[async_trait]
impl<S: RedemptionService> MessageConsumer<RewardsRedemptionPayload, RedemptionStatus> for RewardsRedemptionConsumer<S> {
    async fn should_consume(&self, payload: &RewardsRedemptionPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let redemption_id = payload.redemption_id;
        let redemption = self.repository.redemption(redemption_id).await?;
        Ok(redemption.status == RedemptionStatus::Pending)
    }

    async fn consume(&self, payload: RewardsRedemptionPayload) -> Result<RedemptionStatus, Box<dyn Error + Send + Sync>> {
        let redemption_id = payload.redemption_id;
        let RedemptionStart { redemption, recipient_address, option } = self.repository.start_redemption(redemption_id).await?;

        let asset_id = option.asset.as_ref().map(|a| a.id.clone());
        let asset_id_str = asset_id.as_ref().map(ToString::to_string);
        let value = option.value.clone();
        let points = option.points;

        let asset = option.asset.map(|asset| RedemptionAsset { asset, value: option.value.clone() });

        let request = RedemptionRequest { recipient_address, asset };

        match self.redeem_with_retry(request).await {
            Ok(transaction_id) => {
                let updates = vec![RedemptionUpdate::TransactionId(transaction_id.clone()), RedemptionUpdate::Status(RedemptionStatus::Completed)];
                self.repository.update_redemption(redemption_id, updates).await?;

                if let Some(id) = &asset_id {
                    let pending_tx_id = TransactionId::new(id.chain, transaction_id.clone());
                    if let Err(error) = self.stream_producer.publish_pending_transaction(pending_tx_id.clone()).await {
                        info_with_fields!("failed to publish redemption transaction to pending", transaction_id = pending_tx_id.to_string(), error = error.to_string());
                    } else {
                        info_with_fields!("published redemption transaction to pending", transaction_id = pending_tx_id.to_string());
                    }

                    let metadata = NotificationRewardsRedeemMetadata {
                        transaction_id: transaction_id.clone(),
                        points,
                        value: value.clone(),
                    };
                    let notification = InAppNotificationPayload::new_with_asset(redemption.wallet_id, id.clone(), NotificationType::RewardsRedeemed, serde_json::to_value(metadata).ok());
                    self.stream_producer.publish_in_app_notifications(vec![notification]).await?;
                }

                info_with_fields!("redemption completed", id = payload.redemption_id, asset = asset_id_str.as_deref().unwrap_or("none"), value = value, tx_id = transaction_id);
                Ok(RedemptionStatus::Completed)
            }
            Err(error) => {
                let error_msg = error.to_string();
                let updates = vec![RedemptionUpdate::Status(RedemptionStatus::Failed), RedemptionUpdate::Error(error_msg.clone())];
                self.repository.update_redemption(redemption_id, updates).await?;
                info_with_fields!("redemption failed", id = payload.redemption_id, asset = asset_id_str.as_deref().unwrap_or("none"), value = value, error = error_msg);
                Ok(RedemptionStatus::Failed)
            }
        }
    }
}
