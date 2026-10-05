use super::config::AbuseDetectionConfig;
use super::repository::{AbuseFacts, Repository};
use crate::ConfigCacher;
use gem_tracing::info_with_fields;
use primitives::rewards::RewardStatus;
use primitives::{NaiveDateTimeExt, now};
use std::error::Error;
use std::sync::Arc;
use storage::AbusePatterns;
use streamer::{RewardsNotificationPayload, StreamProducerQueue};

struct AbuseEvaluation {
    username: String,
    status: RewardStatus,
    referrals: i64,
    attempts: i64,
    risk_score: i64,
    patterns: AbusePatterns,
    score: AbuseScoreBreakdown,
    pattern_penalty: PatternPenaltyBreakdown,
    referrer_disabled: bool,
    disabled_referrer_penalty: f64,
    threshold: f64,
    abuse_score: f64,
    abuse_percent: f64,
}

struct AbuseScoreBreakdown {
    base_score: f64,
    risk_score_per_referral: f64,
    attempts_per_referral: f64,
    attempt_penalty_score: f64,
}

struct PatternPenaltyBreakdown {
    country_rotation_penalty: f64,
    ring_penalty: f64,
    device_farming_penalty: f64,
    velocity_penalty: f64,
}

impl PatternPenaltyBreakdown {
    fn total(&self) -> f64 {
        self.country_rotation_penalty + self.ring_penalty + self.device_farming_penalty + self.velocity_penalty
    }
}

pub struct RewardsAbuseChecker {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl RewardsAbuseChecker {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: Arc<ConfigCacher>, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { repository, config, stream_producer }
    }

    pub async fn check(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let config = AbuseDetectionConfig::from_config(&self.config).await?;
        let since = now().ago(config.lookback);

        let facts = self.repository.abuse_facts(since, config.min_referrals_to_evaluate, config.velocity_window.as_secs() as i64).await?;
        let mut evaluations: Vec<AbuseEvaluation> = facts.into_iter().map(|facts| Self::evaluate_user(facts, &config)).collect();

        evaluations.sort_by(|a, b| b.abuse_percent.partial_cmp(&a.abuse_percent).unwrap_or(std::cmp::Ordering::Equal));

        let (high_risk, low_risk): (Vec<_>, Vec<_>) = evaluations.iter().partition(|e| e.abuse_percent >= 50.0);

        for eval in &high_risk {
            Self::log_evaluation(eval);
        }

        if !low_risk.is_empty() {
            let summary: Vec<String> = low_risk.iter().map(|e| format!("{}({:.0}%)", e.username, e.abuse_percent)).collect();
            info_with_fields!("abuse evaluation summary", users = summary.join(", "));
        }

        let mut disabled_count = 0;
        for eval in evaluations {
            if eval.abuse_score >= eval.threshold
                && let Some(event_id) = self.disable_user(&eval).await?
            {
                self.stream_producer.publish_rewards_events(vec![RewardsNotificationPayload::new(event_id)]).await?;
                disabled_count += 1;
            }
        }

        Ok(disabled_count)
    }

    fn evaluate_user(facts: AbuseFacts, config: &AbuseDetectionConfig) -> AbuseEvaluation {
        let AbuseFacts {
            username,
            status,
            referral_count,
            attempt_count,
            risk_score_sum,
            patterns,
            referrer_disabled,
        } = facts;
        let score = calculate_abuse_score_breakdown(risk_score_sum, attempt_count, referral_count, config);
        let pattern_penalty = calculate_pattern_penalty_breakdown(&patterns, config, &status);
        let disabled_referrer_penalty = if referrer_disabled { config.disabled_referrer_penalty as f64 } else { 0.0 };

        let abuse_score = score.base_score + pattern_penalty.total() + disabled_referrer_penalty;
        let threshold = calculate_abuse_threshold(config, &status);
        let abuse_percent = (abuse_score / threshold * 100.0).min(100.0);

        AbuseEvaluation {
            username,
            status,
            referrals: referral_count,
            attempts: attempt_count,
            risk_score: risk_score_sum,
            patterns,
            score,
            pattern_penalty,
            referrer_disabled,
            disabled_referrer_penalty,
            threshold,
            abuse_score,
            abuse_percent,
        }
    }

    fn log_evaluation(eval: &AbuseEvaluation) {
        info_with_fields!(
            "abuse evaluation",
            username = eval.username,
            status = eval.status.as_ref(),
            referrals = eval.referrals.to_string(),
            attempts = eval.attempts.to_string(),
            risk_score = eval.risk_score.to_string(),
            countries_per_device = eval.patterns.max_countries_per_device.to_string(),
            referrers_per_device = eval.patterns.max_referrers_per_device.to_string(),
            referrers_per_fingerprint = eval.patterns.max_referrers_per_fingerprint.to_string(),
            devices_per_ip = eval.patterns.max_devices_per_ip.to_string(),
            velocity_burst = eval.patterns.signals_in_velocity_window.to_string(),
            referrer_disabled = eval.referrer_disabled.to_string(),
            base_score = format!("{:.2}", eval.score.base_score),
            risk_score_per_referral = format!("{:.2}", eval.score.risk_score_per_referral),
            attempts_per_referral = format!("{:.2}", eval.score.attempts_per_referral),
            attempt_penalty_score = format!("{:.2}", eval.score.attempt_penalty_score),
            country_rotation_penalty = format!("{:.0}", eval.pattern_penalty.country_rotation_penalty),
            ring_penalty = format!("{:.0}", eval.pattern_penalty.ring_penalty),
            device_farming_penalty = format!("{:.0}", eval.pattern_penalty.device_farming_penalty),
            velocity_penalty = format!("{:.0}", eval.pattern_penalty.velocity_penalty),
            pattern_penalty = format!("{:.0}", eval.pattern_penalty.total()),
            disabled_referrer_penalty = format!("{:.0}", eval.disabled_referrer_penalty),
            abuse_threshold = format!("{:.0}", eval.threshold),
            abuse_score = format!("{:.0}", eval.abuse_score),
            abuse_percent = format!("{:.0}%", eval.abuse_percent)
        );
    }

    async fn disable_user(&self, eval: &AbuseEvaluation) -> Result<Option<i32>, Box<dyn Error + Send + Sync>> {
        info_with_fields!(
            "disabled user for abuse",
            username = eval.username,
            abuse_score = format!("{:.0}", eval.abuse_score),
            threshold = format!("{:.0}", eval.threshold)
        );

        let reason = "Auto-disabled due to abuse detection";
        let comment = format!(
            "abuse_score={:.0}, threshold={:.0}, base_score={:.2}, risk_scores={}, attempts={}, referrals={}, risk_score/referral={:.2}, attempts/referral={:.2}, attempt_penalty_score={:.2}, pattern_penalty={:.0}, country_rotation_penalty={:.0}, ring_penalty={:.0}, device_farming_penalty={:.0}, velocity_penalty={:.0}, disabled_referrer_penalty={:.0}, countries/device={}, referrers/device={}, referrers/fingerprint={}, devices/ip={}, velocity_burst={}, referrer_disabled={}",
            eval.abuse_score,
            eval.threshold,
            eval.score.base_score,
            eval.risk_score,
            eval.attempts,
            eval.referrals,
            eval.score.risk_score_per_referral,
            eval.score.attempts_per_referral,
            eval.score.attempt_penalty_score,
            eval.pattern_penalty.total(),
            eval.pattern_penalty.country_rotation_penalty,
            eval.pattern_penalty.ring_penalty,
            eval.pattern_penalty.device_farming_penalty,
            eval.pattern_penalty.velocity_penalty,
            eval.disabled_referrer_penalty,
            eval.patterns.max_countries_per_device,
            eval.patterns.max_referrers_per_device,
            eval.patterns.max_referrers_per_fingerprint,
            eval.patterns.max_devices_per_ip,
            eval.patterns.signals_in_velocity_window,
            eval.referrer_disabled
        );
        let event_id = self.repository.disable_rewards(eval.username.clone(), reason.to_string(), comment).await?;

        Ok(Some(event_id))
    }
}

#[cfg(test)]
fn calculate_abuse_score(risk_score_sum: i64, attempt_count: i64, referral_count: i64, config: &AbuseDetectionConfig) -> f64 {
    calculate_abuse_score_breakdown(risk_score_sum, attempt_count, referral_count, config).base_score
}

fn calculate_abuse_score_breakdown(risk_score_sum: i64, attempt_count: i64, referral_count: i64, config: &AbuseDetectionConfig) -> AbuseScoreBreakdown {
    let referrals = referral_count.max(1) as f64;
    let risk_score_per_referral = risk_score_sum as f64 / referrals;
    let attempts_per_referral = attempt_count as f64 / referrals;
    let attempt_penalty_score = attempts_per_referral * config.attempt_penalty as f64;
    AbuseScoreBreakdown {
        base_score: risk_score_per_referral + attempt_penalty_score,
        risk_score_per_referral,
        attempts_per_referral,
        attempt_penalty_score,
    }
}

fn calculate_abuse_threshold(config: &AbuseDetectionConfig, status: &RewardStatus) -> f64 {
    let multiplier = if *status == RewardStatus::Trusted {
        config.trusted_multiplier as f64
    } else if status.is_verified() {
        config.verified_threshold_multiplier
    } else {
        1.0
    };
    config.disable_threshold as f64 * multiplier
}

#[cfg(test)]
fn calculate_pattern_penalty(patterns: &AbusePatterns, config: &AbuseDetectionConfig, status: &RewardStatus) -> f64 {
    calculate_pattern_penalty_breakdown(patterns, config, status).total()
}

fn calculate_pattern_penalty_breakdown(patterns: &AbusePatterns, config: &AbuseDetectionConfig, status: &RewardStatus) -> PatternPenaltyBreakdown {
    let country_rotation_penalty = if patterns.max_countries_per_device >= config.country_rotation_threshold {
        config.country_rotation_penalty as f64
    } else {
        0.0
    };

    let ring_penalty = if patterns.max_referrers_per_device >= config.ring_referrers_per_device_threshold || patterns.max_referrers_per_fingerprint >= config.ring_referrers_per_fingerprint_threshold {
        config.ring_penalty as f64
    } else {
        0.0
    };

    let device_farming_penalty = if patterns.max_devices_per_ip >= config.device_farming_threshold { config.device_farming_penalty as f64 } else { 0.0 };

    let mut velocity_penalty = 0.0;
    let multiplier = if *status == RewardStatus::Trusted {
        config.trusted_multiplier
    } else if status.is_verified() {
        config.verified_multiplier
    } else {
        1
    };
    let daily_limit = config.referral_per_user_daily * multiplier;
    let velocity_threshold = daily_limit / config.velocity_divisor.max(1);
    if patterns.signals_in_velocity_window >= velocity_threshold {
        let over_threshold = patterns.signals_in_velocity_window - velocity_threshold + 1;
        velocity_penalty = (over_threshold * config.velocity_penalty) as f64;
    }

    PatternPenaltyBreakdown {
        country_rotation_penalty,
        ring_penalty,
        device_farming_penalty,
        velocity_penalty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abuse_score() {
        assert_eq!(calculate_abuse_score(100, 5, 1, &AbuseDetectionConfig::mock()), 175.0);
        assert_eq!(calculate_abuse_score(0, 10, 1, &AbuseDetectionConfig::mock()), 150.0);
        assert_eq!(calculate_abuse_score(200, 0, 1, &AbuseDetectionConfig::mock()), 200.0);
        assert_eq!(calculate_abuse_score(100, 5, 10, &AbuseDetectionConfig::mock()), 17.5);
        assert_eq!(calculate_abuse_score(0, 10, 10, &AbuseDetectionConfig::mock()), 15.0);
        assert_eq!(calculate_abuse_score(200, 0, 10, &AbuseDetectionConfig::mock()), 20.0);
    }

    #[test]
    fn test_abuse_score_breakdown() {
        let score = calculate_abuse_score_breakdown(44, 0, 2, &AbuseDetectionConfig::mock());
        assert_eq!(score.risk_score_per_referral, 22.0);
        assert_eq!(score.attempts_per_referral, 0.0);
        assert_eq!(score.attempt_penalty_score, 0.0);
        assert_eq!(score.base_score, 22.0);
    }

    #[test]
    fn test_abuse_threshold() {
        assert_eq!(calculate_abuse_threshold(&AbuseDetectionConfig::mock(), &RewardStatus::Unverified), 200.0);
        assert_eq!(calculate_abuse_threshold(&AbuseDetectionConfig::mock(), &RewardStatus::Verified), 400.0);
        assert_eq!(calculate_abuse_threshold(&AbuseDetectionConfig::mock(), &RewardStatus::Trusted), 600.0);
    }

    #[test]
    fn test_pattern_penalty() {
        let config = AbuseDetectionConfig::mock();
        let base = AbusePatterns {
            max_countries_per_device: 1,
            max_referrers_per_device: 1,
            max_referrers_per_fingerprint: 1,
            max_devices_per_ip: 2,
            signals_in_velocity_window: 0,
        };

        assert_eq!(calculate_pattern_penalty(&base, &config, &RewardStatus::Unverified), 0.0);
        assert_eq!(calculate_pattern_penalty(&AbusePatterns { max_countries_per_device: 2, ..base }, &config, &RewardStatus::Unverified), 50.0);
        assert_eq!(calculate_pattern_penalty(&AbusePatterns { max_referrers_per_device: 2, ..base }, &config, &RewardStatus::Unverified), 80.0);
        assert_eq!(calculate_pattern_penalty(&AbusePatterns { max_referrers_per_fingerprint: 2, ..base }, &config, &RewardStatus::Unverified), 80.0);
        assert_eq!(calculate_pattern_penalty(&AbusePatterns { max_devices_per_ip: 5, ..base }, &config, &RewardStatus::Unverified), 10.0);

        assert_eq!(calculate_pattern_penalty(&AbusePatterns { signals_in_velocity_window: 1, ..base }, &config, &RewardStatus::Unverified), 0.0);
        assert_eq!(calculate_pattern_penalty(&AbusePatterns { signals_in_velocity_window: 2, ..base }, &config, &RewardStatus::Unverified), 100.0);

        assert_eq!(calculate_pattern_penalty(&AbusePatterns { signals_in_velocity_window: 4, ..base }, &config, &RewardStatus::Verified), 0.0);
        assert_eq!(calculate_pattern_penalty(&AbusePatterns { signals_in_velocity_window: 5, ..base }, &config, &RewardStatus::Verified), 100.0);

        assert_eq!(calculate_pattern_penalty(&AbusePatterns { signals_in_velocity_window: 6, ..base }, &config, &RewardStatus::Trusted), 0.0);
        assert_eq!(calculate_pattern_penalty(&AbusePatterns { signals_in_velocity_window: 7, ..base }, &config, &RewardStatus::Trusted), 100.0);

        assert_eq!(
            calculate_pattern_penalty(
                &AbusePatterns {
                    max_countries_per_device: 5,
                    max_referrers_per_device: 4,
                    max_referrers_per_fingerprint: 3,
                    max_devices_per_ip: 10,
                    signals_in_velocity_window: 5,
                },
                &config,
                &RewardStatus::Unverified
            ),
            540.0
        );
    }

    #[test]
    fn test_pattern_penalty_breakdown() {
        let penalties = calculate_pattern_penalty_breakdown(
            &AbusePatterns {
                max_countries_per_device: 2,
                max_referrers_per_device: 2,
                max_referrers_per_fingerprint: 1,
                max_devices_per_ip: 5,
                signals_in_velocity_window: 2,
            },
            &AbuseDetectionConfig::mock(),
            &RewardStatus::Unverified,
        );
        assert_eq!(penalties.country_rotation_penalty, 50.0);
        assert_eq!(penalties.ring_penalty, 80.0);
        assert_eq!(penalties.device_farming_penalty, 10.0);
        assert_eq!(penalties.velocity_penalty, 100.0);
        assert_eq!(penalties.total(), 240.0);
    }
}
