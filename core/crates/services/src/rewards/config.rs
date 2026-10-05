use std::time::Duration;

use config_keys::{ConfigKey, RateLimit, RateLimitKey, RateLimitWindow};
use primitives::rewards::RewardStatus;
use rewards::{RiskScoreConfig, UsernameRules};
use storage::DatabaseError;

use crate::ConfigCacher;

pub async fn username_rules(config: &ConfigCacher) -> Result<UsernameRules, DatabaseError> {
    let rules = UsernameRules {
        min_length: config.get_usize(ConfigKey::UsernameMinLength).await?,
        max_length: config.get_usize(ConfigKey::UsernameMaxLength).await?,
    };
    if rules.min_length > rules.max_length {
        return Err(DatabaseError::Error(format!("Username min length {} exceeds max length {}", rules.min_length, rules.max_length)));
    }
    Ok(rules)
}

#[derive(Debug, Clone, Copy)]
pub struct ReferralVerificationConfig {
    pub(crate) base_delay: Duration,
    pub(crate) verified_multiplier: i64,
}

impl ReferralVerificationConfig {
    pub async fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        Ok(Self {
            base_delay: config.get_duration(ConfigKey::ReferralVerificationDelay).await?,
            verified_multiplier: config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await?,
        })
    }
}

pub(crate) struct ReferralSecurityConfig {
    pub(crate) tor_allowed: bool,
    pub(crate) ineligible_countries: Vec<String>,
}

impl ReferralSecurityConfig {
    pub(crate) async fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        Ok(Self {
            tor_allowed: config.get_bool(ConfigKey::ReferralIpTorAllowed).await?,
            ineligible_countries: config.get_json(ConfigKey::ReferralIneligibleCountries).await?,
        })
    }
}

pub(crate) struct RedemptionConfig {
    pub(crate) min_account_age: Duration,
    pub(crate) cooldown_after_referral: Duration,
    pub(crate) limits: RateLimit,
}

impl RedemptionConfig {
    pub(crate) async fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        Ok(Self {
            min_account_age: config.get_duration(ConfigKey::RedemptionMinAccountAge).await?,
            cooldown_after_referral: config.get_duration(ConfigKey::RedemptionCooldownAfterReferral).await?,
            limits: config.get_rate_limit(RateLimitKey::RedemptionPerUserLimit).await?,
        })
    }
}

pub(crate) async fn referrer_multiplier(config: &ConfigCacher, status: &RewardStatus) -> Result<i64, DatabaseError> {
    if *status == RewardStatus::Trusted {
        config.get_i64(ConfigKey::ReferralTrustedMultiplier).await
    } else {
        config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await
    }
}

pub(crate) async fn risk_score_config(config: &ConfigCacher) -> Result<RiskScoreConfig, DatabaseError> {
    Ok(RiskScoreConfig {
        fingerprint_match_penalty_per_referrer: config.get_i64(ConfigKey::ReferralRiskScoreFingerprintMatchPerReferrer).await?,
        fingerprint_match_max_penalty: config.get_i64(ConfigKey::ReferralRiskScoreFingerprintMatchMaxPenalty).await?,
        ip_reuse_score: config.get_i64(ConfigKey::ReferralRiskScoreIpReuse).await?,
        isp_model_match_score: config.get_i64(ConfigKey::ReferralRiskScoreIspModelMatch).await?,
        device_id_reuse_penalty_per_referrer: config.get_i64(ConfigKey::ReferralRiskScoreDeviceIdReusePerReferrer).await?,
        device_id_reuse_max_penalty: config.get_i64(ConfigKey::ReferralRiskScoreDeviceIdReuseMaxPenalty).await?,
        ineligible_ip_type_score: config.get_i64(ConfigKey::ReferralRiskScoreIneligibleIpType).await?,
        blocked_ip_types: config.get_json(ConfigKey::ReferralBlockedIpTypes).await?,
        blocked_ip_type_penalty: config.get_i64(ConfigKey::ReferralBlockedIpTypePenalty).await?,
        max_abuse_score: config.get_i64(ConfigKey::ReferralMaxAbuseScore).await?,
        penalty_isps: config.get_json(ConfigKey::ReferralPenaltyIsps).await?,
        isp_penalty_score: config.get_i64(ConfigKey::ReferralPenaltyIspsScore).await?,
        verified_user_reduction: config.get_i64(ConfigKey::ReferralRiskScoreVerifiedUserReduction).await?,
        early_referral_reduction_initial: config.get_i64(ConfigKey::ReferralRiskScoreEarlyReferralReductionInitial).await?,
        early_referral_reduction_step: config.get_i64(ConfigKey::ReferralRiskScoreEarlyReferralReductionStep).await?,
        max_allowed_score: config.get_i64(ConfigKey::ReferralRiskScoreMaxAllowed).await?,
        same_referrer_pattern_threshold: config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternThreshold).await?,
        same_referrer_pattern_penalty: config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerPatternPenalty).await?,
        same_referrer_fingerprint_threshold: config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintThreshold).await?,
        same_referrer_fingerprint_penalty: config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerFingerprintPenalty).await?,
        same_referrer_device_model_threshold: config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerDeviceModelThreshold).await?,
        same_referrer_device_model_penalty: config.get_i64(ConfigKey::ReferralRiskScoreSameReferrerDeviceModelPenalty).await?,
        device_model_ring_threshold: config.get_i64(ConfigKey::ReferralRiskScoreDeviceModelRingThreshold).await?,
        device_model_ring_penalty_per_member: config.get_i64(ConfigKey::ReferralRiskScoreDeviceModelRingPenaltyPerMember).await?,
        lookback: config.get_duration(ConfigKey::ReferralRiskScoreLookback).await?,
        high_risk_platform_stores: config.get_json(ConfigKey::ReferralRiskScoreHighRiskPlatformStores).await?,
        high_risk_platform_store_penalty: config.get_i64(ConfigKey::ReferralRiskScoreHighRiskPlatformStorePenalty).await?,
        high_risk_countries: config.get_json(ConfigKey::ReferralRiskScoreHighRiskCountries).await?,
        high_risk_country_penalty: config.get_i64(ConfigKey::ReferralRiskScoreHighRiskCountryPenalty).await?,
        high_risk_locales: config.get_json(ConfigKey::ReferralRiskScoreHighRiskLocales).await?,
        high_risk_locale_penalty: config.get_i64(ConfigKey::ReferralRiskScoreHighRiskLocalePenalty).await?,
        high_risk_device_models: config.get_json(ConfigKey::ReferralRiskScoreHighRiskDeviceModels).await?,
        high_risk_device_model_penalty: config.get_i64(ConfigKey::ReferralRiskScoreHighRiskDeviceModelPenalty).await?,
        high_risk_user_agents: config.get_json(ConfigKey::ReferralRiskScoreHighRiskUserAgents).await?,
        high_risk_user_agent_penalty: config.get_i64(ConfigKey::ReferralRiskScoreHighRiskUserAgentPenalty).await?,
        ip_history_penalty_per_abuser: config.get_i64(ConfigKey::ReferralRiskScoreIpHistoryPenaltyPerAbuser).await?,
        ip_history_max_penalty: config.get_i64(ConfigKey::ReferralRiskScoreIpHistoryMaxPenalty).await?,
        velocity_window: config.get_duration(ConfigKey::ReferralAbuseVelocityWindow).await?,
        velocity_divisor: config.get_i64(ConfigKey::ReferralAbuseVelocityDivisor).await?,
        velocity_penalty: config.get_i64(ConfigKey::ReferralAbuseVelocityPenaltyPerSignal).await?,
        referral_per_user_daily: config.get_rate_limit(RateLimitKey::ReferralPerUserLimit).await?.get(RateLimitWindow::Day),
        verified_multiplier: config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await?,
        trusted_multiplier: config.get_i64(ConfigKey::ReferralTrustedMultiplier).await?,
        cross_referrer_device_penalty: config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerDevicePenalty).await?,
        cross_referrer_fingerprint_threshold: config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerFingerprintThreshold).await?,
        cross_referrer_fingerprint_penalty: config.get_i64(ConfigKey::ReferralRiskScoreCrossReferrerFingerprintPenalty).await?,
        country_diversity_threshold: config.get_i64(ConfigKey::ReferralRiskScoreCountryDiversityThreshold).await?,
        country_diversity_penalty_per_country: config.get_i64(ConfigKey::ReferralRiskScoreCountryDiversityPenaltyPerCountry).await?,
        device_farming_threshold: config.get_i64(ConfigKey::ReferralRiskScoreDeviceFarmingThreshold).await?,
        device_farming_penalty_per_device: config.get_i64(ConfigKey::ReferralRiskScoreDeviceFarmingPenaltyPerDevice).await?,
    })
}

pub(crate) struct AbuseDetectionConfig {
    pub(crate) disable_threshold: i64,
    pub(crate) attempt_penalty: i64,
    pub(crate) verified_threshold_multiplier: f64,
    pub(crate) lookback: Duration,
    pub(crate) min_referrals_to_evaluate: i64,
    pub(crate) country_rotation_threshold: i64,
    pub(crate) country_rotation_penalty: i64,
    pub(crate) ring_referrers_per_device_threshold: i64,
    pub(crate) ring_referrers_per_fingerprint_threshold: i64,
    pub(crate) ring_penalty: i64,
    pub(crate) device_farming_threshold: i64,
    pub(crate) device_farming_penalty: i64,
    pub(crate) velocity_window: Duration,
    pub(crate) velocity_divisor: i64,
    pub(crate) velocity_penalty: i64,
    pub(crate) referral_per_user_daily: i64,
    pub(crate) verified_multiplier: i64,
    pub(crate) trusted_multiplier: i64,
    pub(crate) disabled_referrer_penalty: i64,
}

impl AbuseDetectionConfig {
    pub(crate) async fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        Ok(Self {
            disable_threshold: config.get_i64(ConfigKey::ReferralAbuseDisableThreshold).await?,
            attempt_penalty: config.get_i64(ConfigKey::ReferralAbuseAttemptPenalty).await?,
            verified_threshold_multiplier: config.get_f64(ConfigKey::ReferralAbuseVerifiedThresholdMultiplier).await?,
            lookback: config.get_duration(ConfigKey::ReferralAbuseLookback).await?,
            min_referrals_to_evaluate: config.get_i64(ConfigKey::ReferralAbuseMinReferralsToEvaluate).await?,
            country_rotation_threshold: config.get_i64(ConfigKey::ReferralAbuseCountryRotationThreshold).await?,
            country_rotation_penalty: config.get_i64(ConfigKey::ReferralAbuseCountryRotationPenalty).await?,
            ring_referrers_per_device_threshold: config.get_i64(ConfigKey::ReferralAbuseRingReferrersPerDeviceThreshold).await?,
            ring_referrers_per_fingerprint_threshold: config.get_i64(ConfigKey::ReferralAbuseRingReferrersPerFingerprintThreshold).await?,
            ring_penalty: config.get_i64(ConfigKey::ReferralAbuseRingPenalty).await?,
            device_farming_threshold: config.get_i64(ConfigKey::ReferralAbuseDeviceFarmingThreshold).await?,
            device_farming_penalty: config.get_i64(ConfigKey::ReferralAbuseDeviceFarmingPenalty).await?,
            velocity_window: config.get_duration(ConfigKey::ReferralAbuseVelocityWindow).await?,
            velocity_divisor: config.get_i64(ConfigKey::ReferralAbuseVelocityDivisor).await?,
            velocity_penalty: config.get_i64(ConfigKey::ReferralAbuseVelocityPenaltyPerSignal).await?,
            referral_per_user_daily: config.get_rate_limit(RateLimitKey::ReferralPerUserLimit).await?.get(RateLimitWindow::Day),
            verified_multiplier: config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await?,
            trusted_multiplier: config.get_i64(ConfigKey::ReferralTrustedMultiplier).await?,
            disabled_referrer_penalty: config.get_i64(ConfigKey::ReferralAbuseDisabledReferrerPenalty).await?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::testkit::MemoryConfigRepository;

    fn config(repository: MemoryConfigRepository) -> ConfigCacher {
        ConfigCacher::new(Arc::new(repository))
    }

    #[tokio::test]
    async fn test_seeded_defaults_load() {
        let config = config(MemoryConfigRepository::new());

        assert!(username_rules(&config).await.is_ok());
        assert!(ReferralVerificationConfig::from_config(&config).await.is_ok());
        assert!(ReferralSecurityConfig::from_config(&config).await.is_ok());
        assert!(RedemptionConfig::from_config(&config).await.is_ok());
        assert!(risk_score_config(&config).await.is_ok());
        assert!(AbuseDetectionConfig::from_config(&config).await.is_ok());
        assert_eq!(referrer_multiplier(&config, &RewardStatus::Trusted).await.unwrap(), 3);
        assert_eq!(referrer_multiplier(&config, &RewardStatus::Verified).await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_malformed_value_fails() {
        let config = config(MemoryConfigRepository::new().with_value(ConfigKey::ReferralRiskScoreLookback.as_ref(), "soon"));

        assert!(risk_score_config(&config).await.is_err());
    }

    #[tokio::test]
    async fn test_missing_value_fails() {
        let config = config(MemoryConfigRepository::new().without(ConfigKey::ReferralAbuseDisableThreshold.as_ref()));

        assert!(matches!(AbuseDetectionConfig::from_config(&config).await, Err(error) if error.is_not_found()));
    }

    #[tokio::test]
    async fn test_reload_reads_updated_value() {
        let config = config(MemoryConfigRepository::new());
        let max_allowed_score = risk_score_config(&config).await.unwrap().max_allowed_score;

        config.set(ConfigKey::ReferralRiskScoreMaxAllowed, &(max_allowed_score + 10).to_string()).await.unwrap();

        assert_eq!(risk_score_config(&config).await.unwrap().max_allowed_score, max_allowed_score + 10);
    }

    #[tokio::test]
    async fn test_username_rules_reject_inverted_lengths() {
        let config = config(MemoryConfigRepository::new().with_value(ConfigKey::UsernameMinLength.as_ref(), "30").with_value(ConfigKey::UsernameMaxLength.as_ref(), "20"));

        assert!(username_rules(&config).await.is_err());
    }
}
