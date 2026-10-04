use std::sync::Arc;

use config_keys::RateLimitWindow;
use primitives::rewards::{RedemptionResult, Rewards};
use primitives::{NaiveDateTimeExt, now};
use rewards::RewardsRedemptionError;
use storage::{Database, DatabaseError, RewardsRedemptionsRepository, RewardsRepository};
use streamer::{RewardsRedemptionPayload, StreamProducerQueue};

use super::config::{RedemptionConfig, username_rules};
use super::error::RewardsServiceError;
use super::redemption::redeem_points;
use super::summary::rewards_by_wallet_id;
use crate::ConfigCacher;

pub struct RewardsRedemptionClient {
    database: Database,
    config: Arc<ConfigCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl RewardsRedemptionClient {
    pub fn new(database: Database, config: Arc<ConfigCacher>, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { database, config, stream_producer }
    }

    pub async fn redeem_by_wallet_id(&self, wallet_id: i32, id: &str, device_id: i32, locale: &str) -> Result<RedemptionResult, RewardsServiceError> {
        let rules = username_rules(&self.config).await?;
        let rewards = self.database.run(move |client| rewards_by_wallet_id(client, wallet_id, &rules)).await?;

        if !rewards.status.is_verified() {
            return Err(RewardsServiceError::redemption(RewardsRedemptionError::NotEligible, locale));
        }

        let username = rewards.code.clone().ok_or_else(|| RewardsServiceError::redemption(RewardsRedemptionError::NoUsername, locale))?;

        self.check_redemption_limits(&username, &rewards).await?.map_err(|error| RewardsServiceError::redemption(error, locale))?;

        let option_id = id.to_string();
        let points = rewards.points;
        let response = self
            .database
            .run(move |client| redeem_points(client, &username, points, &option_id, device_id, wallet_id))
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

        let cooldown_since = current.ago(config.cooldown_after_referral);
        let limits = config.limits;
        let username = username.to_string();
        self.database
            .run(move |client| {
                if client.count_referrals_since(&username, cooldown_since)? > 0 {
                    return Ok(Err(RewardsRedemptionError::CooldownNotElapsed));
                }

                for window in RateLimitWindow::ALL {
                    let count = client.count_redemptions_since(&username, current.ago(window.duration()))?;
                    if count >= limits.get(window) {
                        return Ok(Err(RewardsRedemptionError::LimitReached));
                    }
                }

                Ok(Ok(()))
            })
            .await
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
