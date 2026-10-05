use std::sync::Arc;

use config_keys::RateLimitWindow;
use primitives::rewards::{RedemptionResult, Rewards};
use primitives::{NaiveDateTimeExt, now};
use rewards::RewardsRedemptionError;
use storage::DatabaseError;
use streamer::{RewardsRedemptionPayload, StreamProducerQueue};

use super::config::{RedemptionConfig, username_rules};
use super::error::RewardsServiceError;
use super::repository::Repository;
use crate::ConfigCacher;

pub struct RewardsRedemptionClient {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl RewardsRedemptionClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: Arc<ConfigCacher>, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { repository, config, stream_producer }
    }

    pub async fn redeem_by_wallet_id(&self, wallet_id: i32, id: &str, device_id: i32, locale: &str) -> Result<RedemptionResult, RewardsServiceError> {
        let rules = username_rules(&self.config).await?;
        let rewards = self.repository.rewards(wallet_id, rules).await?;

        if !rewards.status.is_verified() {
            return Err(RewardsServiceError::redemption(RewardsRedemptionError::NotEligible, locale));
        }

        let username = rewards.code.clone().ok_or_else(|| RewardsServiceError::redemption(RewardsRedemptionError::NoUsername, locale))?;

        self.check_redemption_limits(&username, &rewards).await?.map_err(|error| RewardsServiceError::redemption(error, locale))?;

        let response = self
            .repository
            .redeem_points(username, rewards.points, id.to_string(), device_id, wallet_id)
            .await?
            .map_err(|error| RewardsServiceError::redemption(error, locale))?;
        self.stream_producer.publish_rewards_redemption(RewardsRedemptionPayload::new(response.redemption_id)).await?;

        Ok(response.result)
    }

    async fn check_redemption_limits(&self, username: &str, rewards: &Rewards) -> Result<Result<(), RewardsRedemptionError>, DatabaseError> {
        let current = now();

        let config = RedemptionConfig::from_config(&self.config).await?;

        if rewards.created_at > current.ago(config.min_account_age) {
            return Ok(Err(RewardsRedemptionError::AccountTooNew));
        }

        let window_limits = RateLimitWindow::ALL.iter().map(|window| (current.ago(window.duration()), config.limits.get(*window))).collect();
        self.repository.check_redemption_limits(username.to_string(), current.ago(config.cooldown_after_referral), window_limits).await
    }
}

#[cfg(test)]
mod tests {
    use rewards::RewardsError;

    use super::*;

    #[test]
    fn test_a_rejected_redemption_reads_in_the_device_language() {
        let rejected = RewardsServiceError::redemption(RewardsRedemptionError::NotEnoughPoints, "es");

        assert!(matches!(&rejected, RewardsServiceError::Rejected(RewardsError::Redemption(_))));
        assert_eq!(rejected.to_string(), "No tienes suficientes puntos para esta recompensa.");
    }
}
