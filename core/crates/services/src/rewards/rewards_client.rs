use std::error::Error;
use std::sync::Arc;

use cacher::{GLOBAL_RATE_LIMIT_SCOPE, RateLimitCacher};
use config_keys::{ConfigKey, RateLimitKey, RateLimitWindow};
use gem_tracing::error_with_fields;
use localizer::LanguageLocalizer;
use primitives::rewards::{RewardRedemptionOption, RewardStatus};
use primitives::{Localize, NaiveDateTimeExt, Platform, ReferralLeaderboard, RewardEvent, Rewards, now};
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
        let eligibility_days = self.referral_eligibility_days().await?;
        match self.wallet_rewards(device, wallet_id, rules, eligibility_days).await {
            Ok(rewards) => Ok(Rewards {
                disable_reason: rewards.disable_reason.map(|_| LanguageLocalizer::new_with_language(locale).notification_rewards_disabled_description()),
                ..rewards
            }),
            Err(error) if error.is_not_found() => Ok(Rewards::default()),
            Err(error) => Err(error.into()),
        }
    }

    async fn wallet_rewards(&self, device: &DeviceRecord, wallet_id: i32, rules: UsernameRules, eligibility_days: i64) -> Result<Rewards, DatabaseError> {
        let rewards = self.repository.get_rewards(wallet_id, rules).await?;
        let facts = self.repository.get_referral_use_facts(wallet_id, device.id).await?;
        Ok(Rewards {
            use_referral_code_until: Some(facts.eligibility_ends_at(device.created_at, eligibility_days).and_utc()),
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

        self.consume_username_creation_limits(ip_address, device_id).await.map_err(|error| self.map_username_error(error, locale))?;

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;

        self.consume_username_creation_limit(RateLimitKey::UsernameCreationPerCountryLimit, &ip_result.country_code)
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

    async fn consume_username_creation_limits(&self, ip_address: &str, device_id: i32) -> Result<(), UsernameError> {
        let device_id = device_id.to_string();
        for (key, scope) in [
            (RateLimitKey::UsernameCreationGlobalLimit, GLOBAL_RATE_LIMIT_SCOPE),
            (RateLimitKey::UsernameCreationPerIpLimit, ip_address),
            (RateLimitKey::UsernameCreationPerDeviceLimit, device_id.as_str()),
        ] {
            self.consume_username_creation_limit(key, scope).await?;
        }
        Ok(())
    }

    async fn consume_username_creation_limit(&self, key: RateLimitKey, scope: &str) -> Result<(), UsernameError> {
        match self.consume_rate_limit(key, scope).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(UsernameError::LimitReached(key)),
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

    async fn referral_eligibility_days(&self) -> Result<i64, DatabaseError> {
        Ok((self.config.get_duration(ConfigKey::ReferralEligibility).await?.as_secs() / 86400) as i64)
    }

    async fn validate_and_score_referral(&self, device: &DeviceRecord, wallet_id: i32, referrer_username: &str, ip_address: &str, user_agent: &str) -> ReferralProcessResult {
        match self.validate_and_score_referral_inner(device, wallet_id, referrer_username, ip_address, user_agent).await {
            Ok(result) => result,
            Err(error) => ReferralProcessResult::Failed(error),
        }
    }

    async fn validate_and_score_referral_inner(&self, device: &DeviceRecord, wallet_id: i32, referrer_username: &str, ip_address: &str, user_agent: &str) -> Result<ReferralProcessResult, ReferralError> {
        let device_id = device.id.to_string();
        self.consume_referral_limits([
            (RateLimitKey::ReferralGlobalLimit, GLOBAL_RATE_LIMIT_SCOPE),
            (RateLimitKey::ReferralPerDeviceLimit, device_id.as_str()),
            (RateLimitKey::ReferralPerIpLimit, ip_address),
        ])
        .await?;

        let referrer_info = self.repository.get_referrer_info(referrer_username.to_string()).await.map_err(ReferralError::internal)?;
        if !referrer_info.status.is_verified() {
            return Err(ReferralValidationError::RewardsNotEnabled(referrer_username.to_string()).into());
        }

        let multiplier = referrer_multiplier(&self.config, &referrer_info.status).await.map_err(ReferralError::internal)?;
        let current = now();
        let cooldown = self.config.get_duration(ConfigKey::ReferralCooldown).await.map_err(ReferralError::internal)?;
        let limits = self.config.get_rate_limit(RateLimitKey::ReferralPerUserLimit).await.map_err(ReferralError::internal)?;
        let eligibility_days = self.referral_eligibility_days().await.map_err(ReferralError::internal)?;

        let check = ReferralUseCheck {
            referrer_username: referrer_username.to_string(),
            referrer_wallet_id: referrer_info.wallet_id,
            wallet_id,
            device_id: device.id,
            device_created_at: device.created_at,
            window_limits: RateLimitWindow::ALL.iter().map(|window| (current.ago(window.duration()), limits.get(*window) * multiplier)).collect(),
            cooldown_since: current.ago(cooldown),
            eligibility_days,
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
        let since = now().ago(risk_score_config.lookback);

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
        let mut allowed = true;
        for (key, scope) in limits {
            allowed &= self.consume_rate_limit(key, scope).await?;
        }
        if !allowed {
            return Err(ReferralError::LimitReached);
        }
        Ok(())
    }

    async fn consume_rate_limit(&self, key: RateLimitKey, scope: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.rate_limiter.consume(key, scope, self.config.get_rate_limit(key).await?).await
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer.publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect()).await?;
        Ok(())
    }
}
