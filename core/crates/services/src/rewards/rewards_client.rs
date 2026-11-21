use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::{GLOBAL_RATE_LIMIT_SCOPE, RateLimitCacher};
use config_keys::{ConfigKey, RateLimitKey, RateLimitWindow};
use gem_tracing::error_with_fields;
use localizer::LanguageLocalizer;
use primitives::rewards::{RewardRedemptionOption, RewardStatus};
use primitives::{DateTime, Localize, Platform, ReferralLeaderboard, RewardEvent, Rewards};
use pusher::PushProvider;
use rewards::{ReferralError, ReferralValidationError, RewardsError, RiskScoringInput, UsernameError, UsernameRules};
use storage::{DatabaseError, DeviceRecord, WalletRecord};
use streamer::{RewardsNotificationPayload, StreamProducerQueue};

use super::config::{ReferralSecurityConfig, ReferralVerificationConfig, referrer_multiplier, risk_score_config, username_rules};
use super::error::RewardsServiceError;
use super::ip_security_client::IpSecurityClient;
use super::repository::{ReferralCodeRequest, ReferralCodeUse, ReferralUseCheck, Repository};
use super::risk::RiskAssessment;
use crate::ConfigCacher;

enum ReferralProcessResult {
    Success { risk_signal_id: i32, referrer_status: RewardStatus },
    Failed(ReferralError),
    RiskScoreExceeded(i32, ReferralError),
}

pub struct RewardsClient {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
    rate_limiter: Arc<dyn RateLimitCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
    ip_security_client: IpSecurityClient,
    pusher: Arc<dyn PushProvider>,
}

impl RewardsClient {
    pub(crate) fn new(
        repository: Arc<dyn Repository>,
        config: Arc<ConfigCacher>,
        rate_limiter: Arc<dyn RateLimitCacher>,
        stream_producer: Arc<dyn StreamProducerQueue>,
        ip_security_client: IpSecurityClient,
        pusher: Arc<dyn PushProvider>,
    ) -> Self {
        Self {
            repository,
            config,
            rate_limiter,
            stream_producer,
            ip_security_client,
            pusher,
        }
    }

    fn map_username_error(&self, error: UsernameError, locale: &str) -> RewardsError {
        if matches!(error, UsernameError::Internal(_)) {
            error_with_fields!("username creation failed", &error);
        }
        RewardsError::Username(error.localize(locale))
    }

    pub async fn get_rewards_by_wallet_id(&self, device: &DeviceRecord, wallet_id: i32, locale: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let rules = username_rules(&self.config).await?;
        let eligibility = self.config.get_duration(ConfigKey::ReferralEligibility).await?;
        match self.wallet_rewards(device, wallet_id, rules, eligibility).await {
            Ok(rewards) => Ok(Rewards {
                disable_reason: rewards.disable_reason.map(|_| LanguageLocalizer::new_with_language(locale).notification_rewards_disabled_description()),
                ..rewards
            }),
            Err(error) if error.is_not_found() => Ok(Rewards::default()),
            Err(error) => Err(error.into()),
        }
    }

    async fn wallet_rewards(&self, device: &DeviceRecord, wallet_id: i32, rules: UsernameRules, eligibility: Duration) -> Result<Rewards, DatabaseError> {
        let rewards = self.repository.get_rewards(wallet_id, rules).await?;
        let facts = self.repository.get_referral_use_facts(wallet_id, device.id).await?;
        Ok(Rewards {
            use_referral_code_until: Some(facts.eligibility_ends_at(device.created_at, eligibility)),
            ..rewards
        })
    }

    pub async fn get_rewards_events_by_wallet_id(&self, wallet_id: i32) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.get_reward_events(wallet_id).await?)
    }

    pub async fn get_rewards_leaderboard(&self) -> Result<ReferralLeaderboard, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.get_leaderboard().await?)
    }

    pub async fn get_rewards_redemption_option(&self, code: &str) -> Result<RewardRedemptionOption, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.get_redemption_option(code.to_string()).await?)
    }

    pub async fn create_username(&self, address: &str, code: &str, device_id: i32, ip_address: &str, locale: &str) -> Result<Rewards, RewardsServiceError> {
        let wallet = self.multicoin_wallet(address).await?;

        self.consume_username_creation_limits(username_creation_limits(&device_id.to_string(), ip_address))
            .await
            .map_err(|error| self.map_username_error(error, locale))?;

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;

        self.consume_username_creation_limits([(RateLimitKey::UsernameCreationPerCountryLimit, ip_result.country_code.as_str())])
            .await
            .map_err(|error| self.map_username_error(error, locale))?;

        let rules = username_rules(&self.config).await?;
        let event_id = self
            .repository
            .add_username(wallet.id, code.to_string(), rules)
            .await
            .map_err(|error| self.map_username_error(UsernameError::internal(error), locale))?
            .map_err(|error| RewardsError::Username(UsernameError::Validation(error).localize(locale)))?;
        let rewards = self.repository.get_rewards(wallet.id, rules).await.map_err(|error| self.map_username_error(UsernameError::internal(error), locale))?;
        self.publish_events(vec![event_id]).await?;
        Ok(rewards)
    }

    async fn multicoin_wallet(&self, address: &str) -> Result<WalletRecord, DatabaseError> {
        self.repository.get_or_add_multicoin_wallet(address.to_string()).await
    }

    async fn consume_username_creation_limits<'a>(&self, limits: impl IntoIterator<Item = (RateLimitKey, &'a str)>) -> Result<(), UsernameError> {
        match first_limit_reached(&self.config, self.rate_limiter.as_ref(), limits).await {
            Ok(None) => Ok(()),
            Ok(Some(key)) => Err(UsernameError::LimitReached(key)),
            Err(error) => Err(UsernameError::internal(error)),
        }
    }

    pub async fn use_referral_code(&self, device: &DeviceRecord, address: &str, code: &str, ip_address: &str, user_agent: &str) -> Result<Vec<RewardEvent>, RewardsServiceError> {
        let locale = device.device.locale.as_ref();
        let wallet = self.multicoin_wallet(address).await?;

        let wallet_id = wallet.id;
        let device_id = device.id;
        let verification_config = ReferralVerificationConfig::from_config(&self.config).await?;
        let request = ReferralCodeRequest {
            code: code.to_string(),
            wallet_id,
            device_id,
            device_created_at: device.created_at,
        };
        let referral = self.repository.get_referral_code_use(request).await?.map_err(|error| RewardsServiceError::referral(error, locale))?;

        let referrer_username = match referral {
            ReferralCodeUse::Apply { referrer_username, referrer_status } => {
                return self
                    .repository
                    .set_referral(referrer_username, referrer_status, wallet_id, device_id, None, verification_config)
                    .await?
                    .map_err(|error| RewardsServiceError::referral(error, locale));
            }
            ReferralCodeUse::NeedsScoring(referrer_username) => referrer_username,
        };

        match self.validate_and_score_referral(device, wallet_id, &referrer_username, ip_address, user_agent).await {
            ReferralProcessResult::Success { risk_signal_id, referrer_status } => {
                let events = self
                    .repository
                    .set_referral(referrer_username, referrer_status, wallet_id, device_id, Some(risk_signal_id), verification_config)
                    .await?
                    .map_err(|error| RewardsServiceError::referral(error, locale))?;
                Ok(events)
            }
            ReferralProcessResult::Failed(error) => {
                let reason = error.to_string();
                let _ = self.repository.add_referral_attempt(referrer_username, wallet_id, device_id, None, reason).await;
                Err(RewardsServiceError::referral(error, locale))
            }
            ReferralProcessResult::RiskScoreExceeded(risk_signal_id, error) => {
                let reason = error.to_string();
                let _ = self.repository.add_referral_attempt(referrer_username, wallet_id, device_id, Some(risk_signal_id), reason).await;
                Err(RewardsServiceError::referral(error, locale))
            }
        }
    }

    async fn validate_and_score_referral(&self, device: &DeviceRecord, wallet_id: i32, referrer_username: &str, ip_address: &str, user_agent: &str) -> ReferralProcessResult {
        match self.validate_and_score_referral_inner(device, wallet_id, referrer_username, ip_address, user_agent).await {
            Ok(result) => result,
            Err(error) => ReferralProcessResult::Failed(error),
        }
    }

    async fn validate_and_score_referral_inner(&self, device: &DeviceRecord, wallet_id: i32, referrer_username: &str, ip_address: &str, user_agent: &str) -> Result<ReferralProcessResult, ReferralError> {
        self.consume_referral_limits(referral_limits(&device.id.to_string(), ip_address)).await?;

        let referrer_info = self.repository.get_referrer_info(referrer_username.to_string()).await.map_err(ReferralError::internal)?;
        if !referrer_info.status.is_verified() {
            return Err(ReferralValidationError::RewardsNotEnabled(referrer_username.to_string()).into());
        }

        let multiplier = referrer_multiplier(&self.config, &referrer_info.status).await.map_err(ReferralError::internal)?;
        let current = DateTime::now();
        let cooldown = self.config.get_duration(ConfigKey::ReferralCooldown).await.map_err(ReferralError::internal)?;
        let limits = self.config.get_rate_limit(RateLimitKey::ReferralPerUserLimit).await.map_err(ReferralError::internal)?;
        let eligibility = self.config.get_duration(ConfigKey::ReferralEligibility).await.map_err(ReferralError::internal)?;

        let check = ReferralUseCheck {
            referrer_username: referrer_username.to_string(),
            referrer_wallet_id: referrer_info.wallet_id,
            wallet_id,
            device_id: device.id,
            device_created_at: device.created_at,
            window_limits: RateLimitWindow::ALL.iter().map(|window| (current - window.duration(), limits.get(*window) * multiplier)).collect(),
            cooldown_since: current - cooldown,
            eligibility,
        };
        self.repository.get_referral_use_check(check).await.map_err(ReferralError::internal)??;

        if device.device.platform == Platform::Android {
            match self.pusher.is_device_token_valid(&device.device.token, device.device.platform.as_i32()).await {
                Ok(true) => {}
                Ok(false) => return Err(ReferralError::InvalidDeviceToken("token_not_registered".to_string())),
                Err(error) => return Err(ReferralError::InvalidDeviceToken(error.to_string())),
            }
        }

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;
        let security_config = ReferralSecurityConfig::from_config(&self.config).await.map_err(ReferralError::internal)?;
        if !security_config.tor_allowed && ip_result.is_tor {
            return Err(ReferralError::IpTorNotAllowed);
        }
        if security_config.ineligible_countries.contains(&ip_result.country_code) {
            return Err(ReferralError::IpCountryIneligible(ip_result.country_code));
        }
        self.consume_referral_limits([(RateLimitKey::ReferralPerCountryLimit, ip_result.country_code.as_str())]).await?;

        let risk_score_config = risk_score_config(&self.config).await.map_err(ReferralError::internal)?;
        let since = DateTime::now() - risk_score_config.lookback;

        let scoring_input = RiskScoringInput {
            username: referrer_username.to_string(),
            device_id: device.id,
            device_platform: device.device.platform,
            device_platform_store: device.device.platform_store,
            device_os: device.device.os.clone(),
            device_model: device.device.model.clone(),
            device_locale: device.device.locale.as_ref().to_string(),
            device_currency: device.device.currency.to_string(),
            ip_result,
            referrer_status: referrer_info.status,
            referrer_referral_count: referrer_info.referral_count as i64,
            user_agent: user_agent.to_string(),
        };
        let referrer_status = referrer_info.status;
        let assessment = self.repository.add_referral_risk_signal(scoring_input, risk_score_config, since).await.map_err(ReferralError::internal)??;

        Ok(match assessment {
            RiskAssessment::Allowed { risk_signal_id } => ReferralProcessResult::Success { risk_signal_id, referrer_status },
            RiskAssessment::Exceeded { risk_signal_id, error } => ReferralProcessResult::RiskScoreExceeded(risk_signal_id, error),
        })
    }

    async fn consume_referral_limits<'a>(&self, limits: impl IntoIterator<Item = (RateLimitKey, &'a str)>) -> Result<(), ReferralError> {
        match first_limit_reached(&self.config, self.rate_limiter.as_ref(), limits).await? {
            Some(_) => Err(ReferralError::LimitReached),
            None => Ok(()),
        }
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer.publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect()).await?;
        Ok(())
    }
}

fn username_creation_limits<'a>(device_id: &'a str, ip_address: &'a str) -> [(RateLimitKey, &'a str); 3] {
    [
        (RateLimitKey::UsernameCreationPerDeviceLimit, device_id),
        (RateLimitKey::UsernameCreationPerIpLimit, ip_address),
        (RateLimitKey::UsernameCreationGlobalLimit, GLOBAL_RATE_LIMIT_SCOPE),
    ]
}

fn referral_limits<'a>(device_id: &'a str, ip_address: &'a str) -> [(RateLimitKey, &'a str); 3] {
    [
        (RateLimitKey::ReferralPerDeviceLimit, device_id),
        (RateLimitKey::ReferralPerIpLimit, ip_address),
        (RateLimitKey::ReferralGlobalLimit, GLOBAL_RATE_LIMIT_SCOPE),
    ]
}

async fn first_limit_reached<'a>(config: &ConfigCacher, rate_limiter: &dyn RateLimitCacher, limits: impl IntoIterator<Item = (RateLimitKey, &'a str)>) -> Result<Option<RateLimitKey>, Box<dyn Error + Send + Sync>> {
    for (key, scope) in limits {
        if !rate_limiter.consume(key, scope, config.get_rate_limit(key).await?).await? {
            return Ok(Some(key));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{MemoryConfigRepository, MemoryRateLimiter};

    type Limits = for<'a> fn(&'a str, &'a str) -> [(RateLimitKey, &'a str); 3];

    const IP: &str = "203.0.113.7";

    #[tokio::test]
    async fn test_a_refused_device_or_address_spends_no_shared_limit() {
        let config = ConfigCacher::new(Arc::new(MemoryConfigRepository::new()));
        for (limits, device_key, ip_key) in [
            (username_creation_limits as Limits, RateLimitKey::UsernameCreationPerDeviceLimit, RateLimitKey::UsernameCreationPerIpLimit),
            (referral_limits as Limits, RateLimitKey::ReferralPerDeviceLimit, RateLimitKey::ReferralPerIpLimit),
        ] {
            let rate_limiter = MemoryRateLimiter::default();
            let device_limit = config.get_rate_limit(device_key).await.unwrap().get(RateLimitWindow::Minute);
            let ip_limit = config.get_rate_limit(ip_key).await.unwrap().get(RateLimitWindow::Minute);

            for _ in 0..device_limit * 3 {
                first_limit_reached(&config, &rate_limiter, limits("device", IP)).await.unwrap();
            }
            assert_eq!(rate_limiter.consumed(GLOBAL_RATE_LIMIT_SCOPE), device_limit, "a device over {} spends no shared limit", device_key.as_ref());

            for device in 0..ip_limit * 3 {
                first_limit_reached(&config, &rate_limiter, limits(&device.to_string(), IP)).await.unwrap();
            }
            assert_eq!(rate_limiter.consumed(GLOBAL_RATE_LIMIT_SCOPE), ip_limit, "an address over {} spends no shared limit", ip_key.as_ref());
        }
    }
}
