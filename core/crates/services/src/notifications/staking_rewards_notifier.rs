use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use super::repository::Repository;
use cacher::{ThrottleCacher, ThrottledTask};
use chain_providers::ChainProviders;
use gem_tracing::info_with_fields;
use localizer::LanguageLocalizer;
use num_bigint::BigUint;
use number_formatter::{BigNumberFormatter, ValueFormatter, ValueStyle};
use primitives::{Asset, Chain, DelegationBase, DeviceSubscription, TransactionType};
use push_notification::{GorushNotification, PushNotification};
use streamer::{NotificationsPayload, StreamProducerQueue};

use crate::subscriptions::SubscriptionLookup;

#[derive(Clone, Copy)]
pub struct StakeRewardsConfig {
    pub threshold: f64,
    pub lookback: Duration,
}

pub struct StakingRewardsNotifier {
    chain_providers: Arc<ChainProviders>,
    repository: Arc<dyn Repository>,
    config: StakeRewardsConfig,
    throttle: Arc<dyn ThrottleCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
    subscription_lookup: Arc<SubscriptionLookup>,
}

impl StakingRewardsNotifier {
    pub(crate) fn new(
        chain_providers: Arc<ChainProviders>,
        repository: Arc<dyn Repository>,
        config: StakeRewardsConfig,
        throttle: Arc<dyn ThrottleCacher>,
        stream_producer: Arc<dyn StreamProducerQueue>,
        subscription_lookup: Arc<SubscriptionLookup>,
    ) -> Self {
        Self {
            chain_providers,
            repository,
            config,
            throttle,
            stream_producer,
            subscription_lookup,
        }
    }

    pub async fn check_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let since = chrono::Utc::now().naive_utc() - chrono::Duration::from_std(self.config.lookback)?;
        let kinds = TransactionType::staking_types();
        let addresses = self.repository.addresses_with_transactions(chain, kinds, since).await?;

        let mut notified = 0;
        for address in &addresses {
            match self.notify_address(chain, address).await {
                Ok(true) => notified += 1,
                Ok(false) => {}
                Err(error) => {
                    gem_tracing::error("staking rewards notifier", error.as_ref());
                }
            }
        }

        info_with_fields!("staking rewards notifier", chain = chain.as_ref(), addresses = addresses.len(), notified = notified);
        Ok(notified)
    }

    async fn notify_address(&self, chain: Chain, address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.subscription_lookup.get(chain, vec![address.to_string()]).await?;
        if subscriptions.is_empty() {
            return Ok(false);
        }

        if !self.throttle.try_start(ThrottledTask::StakeRewardsAlert { chain: chain.as_ref(), address }).await? {
            return Ok(false);
        }

        let delegations = self.chain_providers.get_staking_delegations(chain, address.to_string()).await?;

        let total_staked = DelegationBase::total_active_balance(&delegations);
        let total_rewards = DelegationBase::total_active_rewards(&delegations);
        if total_staked == BigUint::from(0u32) || total_rewards == BigUint::from(0u32) {
            return Ok(false);
        }

        if BigNumberFormatter::ratio(&total_rewards, &total_staked) < self.config.threshold {
            return Ok(false);
        }

        let asset = Asset::from_chain(chain);
        let rewards_value = ValueFormatter::format(ValueStyle::Auto, &total_rewards.to_string(), asset.decimals)?;

        let notifications: Vec<_> = subscriptions.into_iter().filter_map(|sub| Self::create_notification(sub, &rewards_value, &asset)).collect();

        self.stream_producer.publish_notifications_observers(NotificationsPayload::new(notifications)).await?;

        Ok(true)
    }

    fn create_notification(sub: DeviceSubscription, rewards_value: &str, asset: &Asset) -> Option<GorushNotification> {
        let localizer = LanguageLocalizer::new_with_language(sub.device.locale.as_ref());
        let notification = localizer.notification_stake_rewards(rewards_value, &asset.name);
        let push = PushNotification::new_stake(sub.wallet_id, asset.id.clone());
        GorushNotification::from_device(sub.device, notification.title, notification.description, push)
    }
}
