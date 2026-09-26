use std::error::Error;
use std::sync::Arc;

use cacher::{CacherClient, GLOBAL_RATE_LIMIT_SCOPE, RateLimiter};
use config_keys::{ConfigKey, RateLimitKey, RateLimitWindow};
use gem_tracing::error_with_fields;
use localizer::LanguageLocalizer;
use primitives::rewards::{RewardRedemptionOption, RewardStatus};
use primitives::{Localize, NaiveDateTimeExt, Platform, ReferralLeaderboard, RewardEvent, Rewards, WalletId, WalletSource, WalletType, now};
use pusher::PusherClient;
use rewards::{ReferralError, ReferralValidationError, RewardsError, RiskScoreConfig, RiskScoringInput, UsernameError};
use storage::{Database, DatabaseClient, DatabaseError, DeviceRecord, NewWallet, RewardsRedemptionsRepository, RewardsRepository, WalletRecord, WalletsRepository};
use streamer::{RewardsNotificationPayload, StreamProducer, StreamProducerQueue};

use super::ip_security_client::IpSecurityClient;
use super::referral::{ReferralVerificationConfig, referral_use_facts, use_or_verify_referral};
use super::risk::{RiskAssessment, assess_referral_risk};
use super::summary::rewards_by_wallet_id;
use super::username::{create_username, username_rules};
use crate::ConfigCacher;

enum ReferralCodeUse {
    Applied(Vec<RewardEvent>),
    NeedsScoring(String),
}

enum ReferralProcessResult {
    Success { risk_signal_id: i32, referrer_status: RewardStatus },
    Failed(ReferralError),
    RiskScoreExceeded(i32, ReferralError),
}

struct ReferralSecurityConfig {
    tor_allowed: bool,
    ineligible_countries: Vec<String>,
}

async fn referrer_multiplier(config: &ConfigCacher, status: &RewardStatus) -> Result<i64, DatabaseError> {
    if *status == RewardStatus::Trusted {
        config.get_i64(ConfigKey::ReferralTrustedMultiplier).await
    } else {
        config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await
    }
}

pub struct RewardsClient {
    db: Database,
    config: Arc<ConfigCacher>,
    stream_producer: StreamProducer,
    ip_security_client: IpSecurityClient,
    rate_limiter: RateLimiter,
    pusher: PusherClient,
}

impl RewardsClient {
    pub fn new(database: Database, config: Arc<ConfigCacher>, cacher: CacherClient, stream_producer: StreamProducer, ip_security_client: IpSecurityClient, pusher: PusherClient) -> Self {
        Self {
            db: database,
            config,
            stream_producer,
            ip_security_client,
            rate_limiter: RateLimiter::new(cacher),
            pusher,
        }
    }

    fn map_username_error(&self, error: Box<dyn Error + Send + Sync>, locale: &str) -> RewardsError {
        let error = match error.downcast::<UsernameError>() {
            Ok(error) => *error,
            Err(error) => UsernameError::internal(error),
        };
        if matches!(error, UsernameError::Internal(_)) {
            error_with_fields!("username creation failed", &error);
        }
        RewardsError::Username(error.localize(locale))
    }

    pub async fn get_rewards_by_wallet_id(&self, device: &DeviceRecord, wallet_id: i32, locale: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let rules = username_rules(&self.config).await?;
        let eligibility_days = self.referral_eligibility_days().await?;
        let (device_id, device_created_at) = (device.id, device.created_at);
        let summary = move |client: &mut DatabaseClient| -> Result<Rewards, DatabaseError> {
            let rewards = rewards_by_wallet_id(client, wallet_id, &rules)?;
            let facts = referral_use_facts(client, wallet_id, device_id)?;
            Ok(Rewards {
                use_referral_code_until: Some(facts.eligibility_ends_at(device_created_at, eligibility_days).and_utc()),
                ..rewards
            })
        };
        match self.db.run(summary).await {
            Ok(rewards) => Ok(Rewards {
                disable_reason: rewards.disable_reason.map(|_| LanguageLocalizer::new_with_language(locale).notification_rewards_disabled_description()),
                ..rewards
            }),
            Err(error) if error.is_not_found() => Ok(Rewards::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn get_rewards_events_by_wallet_id(&self, wallet_id: i32) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        Ok(self.db.run(move |client| client.get_reward_events_by_wallet_id(wallet_id)).await?)
    }

    pub async fn get_rewards_leaderboard(&self) -> Result<ReferralLeaderboard, Box<dyn Error + Send + Sync>> {
        Ok(self.db.run(|client| client.get_rewards_leaderboard()).await?)
    }

    pub async fn get_rewards_redemption_option(&self, code: &str) -> Result<RewardRedemptionOption, Box<dyn Error + Send + Sync>> {
        let code = code.to_string();
        Ok(self.db.run(move |client| client.get_redemption_option(&code)).await?)
    }

    pub async fn create_username(&self, address: &str, code: &str, device_id: i32, ip_address: &str, locale: &str) -> Result<Rewards, Box<dyn Error + Send + Sync>> {
        let wallet = self.multicoin_wallet(address).await?;

        self.consume_username_creation_limits(ip_address, device_id).await.map_err(|error| self.map_username_error(error, locale))?;

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;

        self.consume_username_creation_limit(RateLimitKey::UsernameCreationPerCountryLimit, &ip_result.country_code)
            .await
            .map_err(|error| self.map_username_error(error, locale))?;

        let username = code.to_string();
        let rules = username_rules(&self.config).await?;
        let wallet_id = wallet.id;
        let (rewards, event_id) = self
            .db
            .run(move |client| create_username(client, wallet_id, &username, &rules))
            .await
            .map_err(|error| self.map_username_error(error.into(), locale))?
            .map_err(|error| RewardsError::Username(UsernameError::Validation(error).localize(locale)))?;
        self.publish_events(vec![event_id]).await?;
        Ok(rewards)
    }

    async fn multicoin_wallet(&self, address: &str) -> Result<WalletRecord, Box<dyn Error + Send + Sync>> {
        let wallet_id = WalletId::Multicoin(address.to_string());
        Ok(self
            .db
            .run(move |client| {
                client.get_or_create_wallet(NewWallet {
                    wallet_id,
                    wallet_type: WalletType::Multicoin,
                    source: WalletSource::Import,
                })
            })
            .await?)
    }

    async fn consume_username_creation_limits(&self, ip_address: &str, device_id: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    async fn consume_username_creation_limit(&self, key: RateLimitKey, scope: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.consume_rate_limit(key, scope).await? { Ok(()) } else { Err(UsernameError::LimitReached(key).into()) }
    }

    pub async fn use_referral_code(&self, device: &DeviceRecord, address: &str, code: &str, ip_address: &str, user_agent: &str) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
        let locale = device.device.locale.as_ref();
        let wallet = self.multicoin_wallet(address).await?;

        let wallet_id = wallet.id;
        let device_id = device.id;
        let device_created_at = device.created_at;
        let code = code.to_string();
        let referral_locale = locale.to_string();
        let verification_config = ReferralVerificationConfig::from_config(&self.config).await?;
        let referral = self
            .db
            .run(move |client| -> Result<_, Box<dyn Error + Send + Sync>> {
                let referrer_username = client.get_referral_code(&code)?.ok_or_else(|| {
                    let error = ReferralError::from(ReferralValidationError::CodeDoesNotExist);
                    RewardsError::Referral(error.localize(&referral_locale))
                })?;

                let referrer_info = client.get_referrer_info(&referrer_username)?;
                let facts = referral_use_facts(client, wallet_id, device_id).map_err(|error| RewardsError::Referral(ReferralError::internal(error).localize(&referral_locale)))?;
                if facts.is_pending_referral(&referrer_username) {
                    if !referrer_info.status.is_verified() && referrer_info.status != RewardStatus::Attribution {
                        return Err(RewardsError::Referral(ReferralError::from(ReferralValidationError::RewardsNotEnabled(referrer_username.clone())).localize(&referral_locale)).into());
                    }
                    let events = use_or_verify_referral(client, &referrer_username, referrer_info.status, wallet_id, device_id, None, verification_config).map_err(|error| localized_referral_error(error, &referral_locale))?;
                    return Ok(ReferralCodeUse::Applied(events));
                }

                if referrer_info.status == RewardStatus::Attribution {
                    facts
                        .validate_use(&referrer_username, referrer_info.wallet_id, device_created_at, None, now())
                        .map_err(|error| RewardsError::Referral(ReferralError::from(error).localize(&referral_locale)))?;
                    let events = use_or_verify_referral(client, &referrer_username, referrer_info.status, wallet_id, device_id, None, verification_config).map_err(|error| localized_referral_error(error, &referral_locale))?;
                    return Ok(ReferralCodeUse::Applied(events));
                }
                Ok(ReferralCodeUse::NeedsScoring(referrer_username))
            })
            .await?;

        let referrer_username = match referral {
            ReferralCodeUse::Applied(events) => return Ok(events),
            ReferralCodeUse::NeedsScoring(referrer_username) => referrer_username,
        };

        match self.validate_and_score_referral(device, wallet_id, &referrer_username, ip_address, user_agent).await {
            ReferralProcessResult::Success { risk_signal_id, referrer_status } => {
                let events = self
                    .db
                    .run(move |client| use_or_verify_referral(client, &referrer_username, referrer_status, wallet_id, device_id, Some(risk_signal_id), verification_config))
                    .await
                    .map_err(|error| localized_referral_error(error, locale))?;
                Ok(events)
            }
            ReferralProcessResult::Failed(error) => {
                let reason = error.to_string();
                let _ = self.db.run(move |client| client.add_referral_attempt(&referrer_username, wallet_id, device_id, None, &reason)).await;
                Err(RewardsError::Referral(error.localize(locale)).into())
            }
            ReferralProcessResult::RiskScoreExceeded(risk_signal_id, error) => {
                let reason = error.to_string();
                let _ = self.db.run(move |client| client.add_referral_attempt(&referrer_username, wallet_id, device_id, Some(risk_signal_id), &reason)).await;
                Err(RewardsError::Referral(error.localize(locale)).into())
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

        let username = referrer_username.to_string();
        let referrer_info = self.db.run(move |client| client.get_referrer_info(&username)).await.map_err(ReferralError::internal)?;
        if !referrer_info.status.is_verified() {
            return Err(ReferralValidationError::RewardsNotEnabled(referrer_username.to_string()).into());
        }

        let multiplier = referrer_multiplier(&self.config, &referrer_info.status).await.map_err(ReferralError::internal)?;
        let current = now();
        let cooldown = self.config.get_duration(ConfigKey::ReferralCooldown).await.map_err(ReferralError::internal)?;
        let limits = self.config.get_rate_limit(RateLimitKey::ReferralPerUserLimit).await.map_err(ReferralError::internal)?;
        let eligibility_days = self.referral_eligibility_days().await.map_err(ReferralError::internal)?;

        let username = referrer_username.to_string();
        let referrer_wallet_id = referrer_info.wallet_id;
        let device_id = device.id;
        let device_created_at = device.created_at;
        self.db
            .run(move |client| -> Result<Result<(), ReferralError>, DatabaseError> {
                for window in RateLimitWindow::ALL {
                    if client.count_referrals_since(&username, current.ago(window.duration()))? >= limits.get(window) * multiplier {
                        return Ok(Err(ReferralError::ReferrerLimitReached));
                    }
                }

                if client.count_referrals_since(&username, current.ago(cooldown))? >= 1 {
                    return Ok(Err(ReferralError::ReferrerLimitReached));
                }

                let facts = referral_use_facts(client, wallet_id, device_id)?;
                Ok(facts.validate_use(&username, referrer_wallet_id, device_created_at, Some(eligibility_days), now()).map_err(ReferralError::from))
            })
            .await
            .map_err(ReferralError::internal)??;

        if device.device.platform == Platform::Android {
            match self.pusher.is_device_token_valid(&device.device.token, device.device.platform.as_i32()).await {
                Ok(true) => {}
                Ok(false) => return Err(ReferralError::InvalidDeviceToken("token_not_registered".to_string())),
                Err(error) => return Err(ReferralError::InvalidDeviceToken(error.to_string())),
            }
        }

        let ip_result = self.ip_security_client.check_ip(ip_address).await?;
        let security_config = self.load_referral_security_config().await.map_err(ReferralError::internal)?;
        if !security_config.tor_allowed && ip_result.is_tor {
            return Err(ReferralError::IpTorNotAllowed);
        }
        if security_config.ineligible_countries.contains(&ip_result.country_code) {
            return Err(ReferralError::IpCountryIneligible(ip_result.country_code));
        }
        self.consume_referral_limits([(RateLimitKey::ReferralPerCountryLimit, ip_result.country_code.as_str())]).await?;

        let risk_score_config = self.load_risk_score_config().await.map_err(ReferralError::internal)?;
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
        let assessment = self.db.run(move |client| assess_referral_risk(client, &scoring_input, &risk_score_config, since)).await.map_err(ReferralError::internal)??;

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

    async fn load_referral_security_config(&self) -> Result<ReferralSecurityConfig, DatabaseError> {
        Ok(ReferralSecurityConfig {
            tor_allowed: self.config.get_bool(ConfigKey::ReferralIpTorAllowed).await?,
            ineligible_countries: self.config.get_vec_string(ConfigKey::ReferralIneligibleCountries).await?,
        })
    }

    async fn load_risk_score_config(&self) -> Result<RiskScoreConfig, DatabaseError> {
        Ok(RiskScoreConfig {
            fingerprint_match_penalty_per_referrer: self.config.get_i64(ConfigKey::ReferralRiskScoreFingerprintMatchPerReferrer).await?,
            fingerprint_match_max_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreFingerprintMatchMaxPenalty).await?,
            ip_reuse_score: self.config.get_i64(ConfigKey::ReferralRiskScoreIpReuse).await?,
            isp_model_match_score: self.config.get_i64(ConfigKey::ReferralRiskScoreIspModelMatch).await?,
            device_id_reuse_penalty_per_referrer: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceIdReusePerReferrer).await?,
            device_id_reuse_max_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceIdReuseMaxPenalty).await?,
            ineligible_ip_type_score: self.config.get_i64(ConfigKey::ReferralRiskScoreIneligibleIpType).await?,
            blocked_ip_types: self.config.get_vec(ConfigKey::ReferralBlockedIpTypes).await?,
            blocked_ip_type_penalty: self.config.get_i64(ConfigKey::ReferralBlockedIpTypePenalty).await?,
            max_abuse_score: self.config.get_i64(ConfigKey::ReferralMaxAbuseScore).await?,
            penalty_isps: self.config.get_vec_string(ConfigKey::ReferralPenaltyIsps).await?,
            isp_penalty_score: self.config.get_i64(ConfigKey::ReferralPenaltyIspsScore).await?,
            verified_user_reduction: self.config.get_i64(ConfigKey::ReferralRiskScoreVerifiedUserReduction).await?,
            early_referral_reduction_initial: self.config.get_i64(ConfigKey::ReferralRiskScoreEarlyReferralReductionInitial).await?,
            early_referral_reduction_step: self.config.get_i64(ConfigKey::ReferralRiskScoreEarlyReferralReductionStep).await?,
            max_allowed_score: self.config.get_i64(ConfigKey::ReferralRiskScoreMaxAllowed).await?,
            same_referrer_pattern_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternThreshold).await?,
            same_referrer_pattern_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternPenalty).await?,
            same_referrer_fingerprint_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintThreshold).await?,
            same_referrer_fingerprint_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintPenalty).await?,
            same_referrer_device_model_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerDeviceModelThreshold).await?,
            same_referrer_device_model_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerDeviceModelPenalty).await?,
            device_model_ring_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceModelRingThreshold).await?,
            device_model_ring_penalty_per_member: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceModelRingPenaltyPerMember).await?,
            lookback: self.config.get_duration(ConfigKey::ReferralRiskScoreLookback).await?,
            high_risk_platform_stores: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskPlatformStores).await?,
            high_risk_platform_store_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskPlatformStorePenalty).await?,
            high_risk_countries: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskCountries).await?,
            high_risk_country_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskCountryPenalty).await?,
            high_risk_locales: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskLocales).await?,
            high_risk_locale_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskLocalePenalty).await?,
            high_risk_device_models: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskDeviceModels).await?,
            high_risk_device_model_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskDeviceModelPenalty).await?,
            high_risk_user_agents: self.config.get_vec_string(ConfigKey::ReferralRiskScoreHighRiskUserAgents).await?,
            high_risk_user_agent_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreHighRiskUserAgentPenalty).await?,
            ip_history_penalty_per_abuser: self.config.get_i64(ConfigKey::ReferralRiskScoreIpHistoryPenaltyPerAbuser).await?,
            ip_history_max_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreIpHistoryMaxPenalty).await?,
            velocity_window: self.config.get_duration(ConfigKey::ReferralAbuseVelocityWindow).await?,
            velocity_divisor: self.config.get_i64(ConfigKey::ReferralAbuseVelocityDivisor).await?,
            velocity_penalty: self.config.get_i64(ConfigKey::ReferralAbuseVelocityPenaltyPerSignal).await?,
            referral_per_user_daily: self.config.get_rate_limit(RateLimitKey::ReferralPerUserLimit).await?.get(RateLimitWindow::Day),
            verified_multiplier: self.config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await?,
            trusted_multiplier: self.config.get_i64(ConfigKey::ReferralTrustedMultiplier).await?,
            cross_referrer_device_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerDevicePenalty).await?,
            cross_referrer_fingerprint_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerFingerprintThreshold).await?,
            cross_referrer_fingerprint_penalty: self.config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerFingerprintPenalty).await?,
            country_diversity_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreCountryDiversityThreshold).await?,
            country_diversity_penalty_per_country: self.config.get_i64(ConfigKey::ReferralRiskScoreCountryDiversityPenaltyPerCountry).await?,
            device_farming_threshold: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceFarmingThreshold).await?,
            device_farming_penalty_per_device: self.config.get_i64(ConfigKey::ReferralRiskScoreDeviceFarmingPenaltyPerDevice).await?,
        })
    }

    async fn publish_events(&self, event_ids: Vec<i32>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer.publish_rewards_events(event_ids.into_iter().map(RewardsNotificationPayload::new).collect()).await?;
        Ok(())
    }
}

fn localized_referral_error(error: Box<dyn Error + Send + Sync>, locale: &str) -> Box<dyn Error + Send + Sync> {
    match error.downcast::<ReferralError>() {
        Ok(error) => RewardsError::Referral(error.localize(locale)).into(),
        Err(error) => error,
    }
}
