use gem_rewards::{RewardsRedemptionError, redeem_points};
use primitives::rewards::{RedemptionResult, Rewards};
use primitives::{ConfigKey, NaiveDateTimeExt, RateLimitKey, RateLimitWindow, now};
use storage::{ConfigCacher, Database, RewardsRedemptionsRepository, RewardsRepository};
use streamer::{StreamProducer, StreamProducerQueue};

pub struct RewardsRedemptionClient {
    database: Database,
    config: ConfigCacher,
    stream_producer: StreamProducer,
}

impl RewardsRedemptionClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        let config = ConfigCacher::new(database.clone());
        Self {
            database,
            config,
            stream_producer,
        }
    }

    pub async fn redeem_by_wallet_id(&self, wallet_id: i32, id: &str, device_id: i32) -> Result<RedemptionResult, Box<dyn std::error::Error + Send + Sync>> {
        let rewards = self.database.rewards()?.get_reward_by_wallet_id(wallet_id)?;

        if !rewards.status.is_verified() {
            return Err(RewardsRedemptionError::NotEligible("Not eligible for rewards".to_string()).into());
        }

        let username = rewards.code.clone().ok_or(RewardsRedemptionError::NoUsername)?;

        self.check_redemption_limits(&username, &rewards)?;

        let response = redeem_points(&mut self.database.client()?, &username, id, device_id, wallet_id)?;
        self.stream_producer
            .publish_rewards_redemption(streamer::RewardsRedemptionPayload::new(response.redemption_id))
            .await?;

        Ok(response.result)
    }

    fn check_redemption_limits(&self, username: &str, rewards: &Rewards) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let current = now();

        if rewards.created_at > current.ago(self.config.get_duration(ConfigKey::RedemptionMinAccountAge)?) {
            return Err(RewardsRedemptionError::AccountTooNew.into());
        }

        let cooldown_since = current.ago(self.config.get_duration(ConfigKey::RedemptionCooldownAfterReferral)?);
        if self.database.rewards()?.count_referrals_since(username, cooldown_since)? > 0 {
            return Err(RewardsRedemptionError::CooldownNotElapsed.into());
        }

        let limits = self.config.get_rate_limit(RateLimitKey::RedemptionPerUserLimit)?;
        for window in RateLimitWindow::ALL {
            let count = self.database.rewards_redemptions()?.count_redemptions_since(username, current.ago(window.duration()))?;
            if count >= limits.get(window) {
                return Err(RewardsRedemptionError::LimitReached.into());
            }
        }

        Ok(())
    }
}
