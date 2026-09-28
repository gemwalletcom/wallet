use std::error::Error;
use std::sync::Arc;

use crate::ConfigCacher;
use config_keys::ConfigKey;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{NaiveDateTimeExt, RewardStatus, now};
use storage::{Database, DatabaseError, RewardsEligibilityConfig, RewardsFilter, RewardsRepository};
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

pub struct RewardsEligibilityChecker {
    database: Database,
    config: Arc<ConfigCacher>,
    stream_producer: StreamProducer,
}

impl RewardsEligibilityChecker {
    pub fn new(database: Database, config: Arc<ConfigCacher>, stream_producer: StreamProducer) -> Self {
        Self { database, config, stream_producer }
    }

    pub async fn check(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let promotion_limit = self.config.get_i64(ConfigKey::RewardsEligibilityPromotionLimit).await?;
        let active_duration = self.config.get_duration(ConfigKey::RewardsEligibilityActiveDuration).await?;
        let eligibility = RewardsEligibilityConfig {
            activity_cutoff: now().ago(active_duration),
            transactions_required: self.config.get_i64(ConfigKey::RewardsEligibilityTransactionsCount).await?,
        };

        let usernames = self.database.run(|client| client.get_usernames_by_filter(vec![RewardsFilter::Statuses(vec![RewardStatus::Unverified])])).await?;
        let mut promoted = 0;

        for username in usernames {
            if promoted >= promotion_limit as usize {
                break;
            }

            let result = match self.evaluate_and_promote(&username, eligibility).await {
                Ok(result) => result,
                Err(error) => {
                    error_with_fields!("rewards eligibility check failed", &*error, username = username);
                    continue;
                }
            };

            if result {
                promoted += 1;
            }
        }

        Ok(promoted)
    }

    async fn evaluate_and_promote(&self, username: &str, eligibility: RewardsEligibilityConfig) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let candidate = username.to_string();
        let promotion = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let Some(wallet_id) = client.check_eligibility(&candidate, eligibility)? else {
                    return Ok(None);
                };
                Ok(Some((wallet_id, client.promote_to_verified(&candidate)?)))
            })
            .await?;
        let Some((wallet_id, reward_event_ids)) = promotion else {
            return Ok(false);
        };

        info_with_fields!("rewards eligibility promoted user", username = username, wallet_id = wallet_id, events = reward_event_ids.len());

        self.publish_promotion(reward_event_ids).await?;
        Ok(true)
    }

    async fn publish_promotion(&self, reward_event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer.publish_rewards_events(reward_event_ids.into_iter().map(RewardsNotificationPayload::new).collect()).await?;

        Ok(())
    }
}
