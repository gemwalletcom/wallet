use std::error::Error;
use std::sync::Arc;

use config_keys::{ConfigKey, RateLimitKey, RateLimitWindow};
use primitives::Localize;
use primitives::rewards::{RedemptionResult, Rewards};
use primitives::{NaiveDateTimeExt, now};
use rewards::{RewardsError, RewardsRedemptionError};
use storage::{Database, RewardsRedemptionsRepository, RewardsRepository};
use streamer::{RewardsRedemptionPayload, StreamProducer, StreamProducerQueue};

use super::redemption::redeem_points;
use super::summary::rewards_by_wallet_id;
use super::username::username_rules;
use crate::ConfigCacher;

pub struct RewardsRedemptionClient {
    database: Database,
    config: Arc<ConfigCacher>,
    stream_producer: StreamProducer,
}

impl RewardsRedemptionClient {
    pub fn new(database: Database, config: Arc<ConfigCacher>, stream_producer: StreamProducer) -> Self {
        Self { database, config, stream_producer }
    }

    pub async fn redeem_by_wallet_id(&self, wallet_id: i32, id: &str, device_id: i32, locale: &str) -> Result<RedemptionResult, Box<dyn Error + Send + Sync>> {
        self.redeem(wallet_id, id, device_id).await.map_err(|error| localized_redemption_error(error, locale))
    }

    async fn redeem(&self, wallet_id: i32, id: &str, device_id: i32) -> Result<RedemptionResult, Box<dyn Error + Send + Sync>> {
        let rules = username_rules(&self.config).await?;
        let rewards = self.database.run(move |client| rewards_by_wallet_id(client, wallet_id, &rules)).await?;

        if !rewards.status.is_verified() {
            return Err(RewardsRedemptionError::NotEligible.into());
        }

        let username = rewards.code.clone().ok_or(RewardsRedemptionError::NoUsername)?;

        self.check_redemption_limits(&username, &rewards).await?;

        let option_id = id.to_string();
        let points = rewards.points;
        let response = self.database.run(move |client| redeem_points(client, &username, points, &option_id, device_id, wallet_id)).await?;
        self.stream_producer.publish_rewards_redemption(RewardsRedemptionPayload::new(response.redemption_id)).await?;

        Ok(response.result)
    }

    async fn check_redemption_limits(&self, username: &str, rewards: &Rewards) -> Result<(), Box<dyn Error + Send + Sync>> {
        let current = now();

        if rewards.created_at > current.ago(self.config.get_duration(ConfigKey::RedemptionMinAccountAge).await?) {
            return Err(RewardsRedemptionError::AccountTooNew.into());
        }

        let cooldown_since = current.ago(self.config.get_duration(ConfigKey::RedemptionCooldownAfterReferral).await?);
        let limits = self.config.get_rate_limit(RateLimitKey::RedemptionPerUserLimit).await?;
        let username = username.to_string();
        self.database
            .run(move |client| -> Result<(), Box<dyn Error + Send + Sync>> {
                if client.count_referrals_since(&username, cooldown_since)? > 0 {
                    return Err(RewardsRedemptionError::CooldownNotElapsed.into());
                }

                for window in RateLimitWindow::ALL {
                    let count = client.count_redemptions_since(&username, current.ago(window.duration()))?;
                    if count >= limits.get(window) {
                        return Err(RewardsRedemptionError::LimitReached.into());
                    }
                }

                Ok(())
            })
            .await
    }
}

fn localized_redemption_error(error: Box<dyn Error + Send + Sync>, locale: &str) -> Box<dyn Error + Send + Sync> {
    match error.downcast::<RewardsRedemptionError>() {
        Ok(error) => RewardsError::Redemption(error.localize(locale)).into(),
        Err(error) => error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_rejected_redemption_reads_in_the_device_language_and_other_errors_pass_through() {
        let rejected = localized_redemption_error(RewardsRedemptionError::NotEnoughPoints.into(), "es");
        assert!(rejected.downcast_ref::<RewardsError>().is_some());
        assert_eq!(rejected.to_string(), "No tienes suficientes puntos para esta recompensa.");

        assert_eq!(localized_redemption_error("connection refused".into(), "es").to_string(), "connection refused");
    }
}
