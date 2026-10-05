mod units;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::rewards::{RedemptionResponse, RewardRedemptionOption, RewardStatus};
use primitives::{ReferralLeaderboard, RewardEvent, Rewards, WalletId, WalletSource, WalletType, now};
use rewards::{ReferralError, ReferralValidationError, RewardsRedemptionError, RiskScoreConfig, RiskScoringInput, UsernameRules, UsernameValidationError};
use storage::{
    AbusePatterns, Database, DatabaseError, NewWallet, RedemptionRecord, RedemptionUpdate, ReferrerInfo, RewardsEligibilityConfig, RewardsFilter, RewardsRedemptionsRepository, RewardsRepository, RiskSignalsRepository, WalletRecord,
    WalletsRepository,
};
use streamer::InAppNotificationPayload;

pub(crate) use units::create_username;

use super::config::ReferralVerificationConfig;
use super::risk::RiskAssessment;

pub(crate) enum ReferralCodeUse {
    Applied(Vec<RewardEvent>),
    NeedsScoring(String),
}

pub(crate) struct ReferralCodeRequest {
    pub(crate) code: String,
    pub(crate) wallet_id: i32,
    pub(crate) device_id: i32,
    pub(crate) device_created_at: NaiveDateTime,
    pub(crate) verification_config: ReferralVerificationConfig,
}

pub(crate) struct ReferralUseCheck {
    pub(crate) referrer_username: String,
    pub(crate) referrer_wallet_id: i32,
    pub(crate) wallet_id: i32,
    pub(crate) device_id: i32,
    pub(crate) device_created_at: NaiveDateTime,
    pub(crate) window_limits: Vec<(NaiveDateTime, i64)>,
    pub(crate) cooldown_since: NaiveDateTime,
    pub(crate) eligibility_days: i64,
}

pub(crate) struct AbuseFacts {
    pub(crate) username: String,
    pub(crate) status: RewardStatus,
    pub(crate) referral_count: i64,
    pub(crate) attempt_count: i64,
    pub(crate) risk_score_sum: i64,
    pub(crate) patterns: AbusePatterns,
    pub(crate) referrer_disabled: bool,
}

pub(crate) struct RedemptionStart {
    pub(crate) redemption: RedemptionRecord,
    pub(crate) recipient_address: String,
    pub(crate) option: RewardRedemptionOption,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn wallet_rewards(&self, wallet_id: i32, device_id: i32, device_created_at: NaiveDateTime, rules: UsernameRules, eligibility_days: i64) -> Result<Rewards, DatabaseError>;
    async fn rewards(&self, wallet_id: i32, rules: UsernameRules) -> Result<Rewards, DatabaseError>;
    async fn reward_events(&self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError>;
    async fn leaderboard(&self) -> Result<ReferralLeaderboard, DatabaseError>;
    async fn redemption_option(&self, code: String) -> Result<RewardRedemptionOption, DatabaseError>;
    async fn multicoin_wallet(&self, address: String) -> Result<WalletRecord, DatabaseError>;
    async fn create_username(&self, wallet_id: i32, username: String, rules: UsernameRules) -> Result<Result<(Rewards, i32), UsernameValidationError>, DatabaseError>;
    async fn apply_referral_code(&self, request: ReferralCodeRequest) -> Result<Result<ReferralCodeUse, ReferralError>, DatabaseError>;
    async fn referrer_info(&self, username: String) -> Result<ReferrerInfo, DatabaseError>;
    async fn check_referral_use(&self, check: ReferralUseCheck) -> Result<Result<(), ReferralError>, DatabaseError>;
    async fn assess_referral_risk(&self, input: RiskScoringInput, config: RiskScoreConfig, since: NaiveDateTime) -> Result<Result<RiskAssessment, ReferralError>, DatabaseError>;
    async fn verify_referral(
        &self,
        referrer_username: String,
        referrer_status: RewardStatus,
        wallet_id: i32,
        device_id: i32,
        risk_signal_id: Option<i32>,
        verification_config: ReferralVerificationConfig,
    ) -> Result<Result<Vec<RewardEvent>, ReferralError>, DatabaseError>;
    async fn add_referral_attempt(&self, referrer_username: String, wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: String) -> Result<(), DatabaseError>;
    async fn check_redemption_limits(&self, username: String, cooldown_since: NaiveDateTime, window_limits: Vec<(NaiveDateTime, i64)>) -> Result<Result<(), RewardsRedemptionError>, DatabaseError>;
    async fn redeem_points(&self, username: String, points: i32, option_id: String, device_id: i32, wallet_id: i32) -> Result<Result<RedemptionResponse, RewardsRedemptionError>, DatabaseError>;
    async fn event_notifications(&self, event_id: i32) -> Result<Vec<InAppNotificationPayload>, DatabaseError>;
    async fn abuse_facts(&self, since: NaiveDateTime, min_referrals: i64, velocity_window_secs: i64) -> Result<Vec<AbuseFacts>, DatabaseError>;
    async fn disable_rewards(&self, username: String, reason: String, comment: String) -> Result<i32, DatabaseError>;
    async fn usernames_with_status(&self, status: RewardStatus) -> Result<Vec<String>, DatabaseError>;
    async fn promote_if_eligible(&self, username: String, eligibility: RewardsEligibilityConfig) -> Result<Option<(i32, Vec<i32>)>, DatabaseError>;
    async fn redemption(&self, redemption_id: i32) -> Result<RedemptionRecord, DatabaseError>;
    async fn start_redemption(&self, redemption_id: i32) -> Result<RedemptionStart, DatabaseError>;
    async fn update_redemption(&self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn wallet_rewards(&self, wallet_id: i32, device_id: i32, device_created_at: NaiveDateTime, rules: UsernameRules, eligibility_days: i64) -> Result<Rewards, DatabaseError> {
        self.database
            .run(move |client| {
                let rewards = units::rewards_by_wallet_id(client, wallet_id, &rules)?;
                let facts = units::referral_use_facts(client, wallet_id, device_id)?;
                Ok(Rewards {
                    use_referral_code_until: Some(facts.eligibility_ends_at(device_created_at, eligibility_days).and_utc()),
                    ..rewards
                })
            })
            .await
    }

    async fn rewards(&self, wallet_id: i32, rules: UsernameRules) -> Result<Rewards, DatabaseError> {
        self.database.run(move |client| units::rewards_by_wallet_id(client, wallet_id, &rules)).await
    }

    async fn reward_events(&self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError> {
        self.database.run(move |client| client.get_reward_events_by_wallet_id(wallet_id)).await
    }

    async fn leaderboard(&self) -> Result<ReferralLeaderboard, DatabaseError> {
        self.database.run(RewardsRepository::get_rewards_leaderboard).await
    }

    async fn redemption_option(&self, code: String) -> Result<RewardRedemptionOption, DatabaseError> {
        self.database.run(move |client| client.get_redemption_option(&code)).await
    }

    async fn multicoin_wallet(&self, address: String) -> Result<WalletRecord, DatabaseError> {
        self.database
            .run(move |client| {
                client.get_or_create_wallet(NewWallet {
                    wallet_id: WalletId::Multicoin(address),
                    wallet_type: WalletType::Multicoin,
                    source: WalletSource::Import,
                })
            })
            .await
    }

    async fn create_username(&self, wallet_id: i32, username: String, rules: UsernameRules) -> Result<Result<(Rewards, i32), UsernameValidationError>, DatabaseError> {
        self.database.run(move |client| units::create_username(client, wallet_id, &username, &rules)).await
    }

    async fn apply_referral_code(&self, request: ReferralCodeRequest) -> Result<Result<ReferralCodeUse, ReferralError>, DatabaseError> {
        self.database
            .run(move |client| {
                let ReferralCodeRequest {
                    code,
                    wallet_id,
                    device_id,
                    device_created_at,
                    verification_config,
                } = request;
                let Some(referrer_username) = client.get_referral_code(&code)? else {
                    return Ok(Err(ReferralValidationError::CodeDoesNotExist.into()));
                };
                let referrer_info = client.get_referrer_info(&referrer_username)?;
                let facts = match units::referral_use_facts(client, wallet_id, device_id) {
                    Ok(facts) => facts,
                    Err(error) => return Ok(Err(ReferralError::internal(error))),
                };
                if facts.is_pending_referral(&referrer_username) {
                    if !referrer_info.status.is_verified() && referrer_info.status != RewardStatus::Attribution {
                        return Ok(Err(ReferralValidationError::RewardsNotEnabled(referrer_username).into()));
                    }
                    return Ok(units::use_or_verify_referral(client, &referrer_username, referrer_info.status, wallet_id, device_id, None, verification_config)?.map(ReferralCodeUse::Applied));
                }
                if referrer_info.status == RewardStatus::Attribution {
                    if let Err(error) = facts.validate_use(&referrer_username, referrer_info.wallet_id, device_created_at, None, now()) {
                        return Ok(Err(error.into()));
                    }
                    return Ok(units::use_or_verify_referral(client, &referrer_username, referrer_info.status, wallet_id, device_id, None, verification_config)?.map(ReferralCodeUse::Applied));
                }
                Ok(Ok(ReferralCodeUse::NeedsScoring(referrer_username)))
            })
            .await
    }

    async fn referrer_info(&self, username: String) -> Result<ReferrerInfo, DatabaseError> {
        self.database.run(move |client| client.get_referrer_info(&username)).await
    }

    async fn check_referral_use(&self, check: ReferralUseCheck) -> Result<Result<(), ReferralError>, DatabaseError> {
        self.database
            .run(move |client| {
                for (since, limit) in &check.window_limits {
                    if client.count_referrals_since(&check.referrer_username, *since)? >= *limit {
                        return Ok(Err(ReferralError::ReferrerLimitReached));
                    }
                }
                if client.count_referrals_since(&check.referrer_username, check.cooldown_since)? >= 1 {
                    return Ok(Err(ReferralError::ReferrerLimitReached));
                }
                let facts = units::referral_use_facts(client, check.wallet_id, check.device_id)?;
                Ok(facts
                    .validate_use(&check.referrer_username, check.referrer_wallet_id, check.device_created_at, Some(check.eligibility_days), now())
                    .map_err(ReferralError::from))
            })
            .await
    }

    async fn assess_referral_risk(&self, input: RiskScoringInput, config: RiskScoreConfig, since: NaiveDateTime) -> Result<Result<RiskAssessment, ReferralError>, DatabaseError> {
        self.database.run(move |client| units::assess_referral_risk(client, &input, &config, since)).await
    }

    async fn verify_referral(
        &self,
        referrer_username: String,
        referrer_status: RewardStatus,
        wallet_id: i32,
        device_id: i32,
        risk_signal_id: Option<i32>,
        verification_config: ReferralVerificationConfig,
    ) -> Result<Result<Vec<RewardEvent>, ReferralError>, DatabaseError> {
        self.database
            .run(move |client| units::use_or_verify_referral(client, &referrer_username, referrer_status, wallet_id, device_id, risk_signal_id, verification_config))
            .await
    }

    async fn add_referral_attempt(&self, referrer_username: String, wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: String) -> Result<(), DatabaseError> {
        self.database.run(move |client| client.add_referral_attempt(&referrer_username, wallet_id, device_id, risk_signal_id, &reason)).await
    }

    async fn check_redemption_limits(&self, username: String, cooldown_since: NaiveDateTime, window_limits: Vec<(NaiveDateTime, i64)>) -> Result<Result<(), RewardsRedemptionError>, DatabaseError> {
        self.database
            .run(move |client| {
                if client.count_referrals_since(&username, cooldown_since)? > 0 {
                    return Ok(Err(RewardsRedemptionError::CooldownNotElapsed));
                }
                for (since, limit) in window_limits {
                    if client.count_redemptions_since(&username, since)? >= limit {
                        return Ok(Err(RewardsRedemptionError::LimitReached));
                    }
                }
                Ok(Ok(()))
            })
            .await
    }

    async fn redeem_points(&self, username: String, points: i32, option_id: String, device_id: i32, wallet_id: i32) -> Result<Result<RedemptionResponse, RewardsRedemptionError>, DatabaseError> {
        self.database.run(move |client| units::redeem_points(client, &username, points, &option_id, device_id, wallet_id)).await
    }

    async fn event_notifications(&self, event_id: i32) -> Result<Vec<InAppNotificationPayload>, DatabaseError> {
        self.database
            .run(move |client| {
                let event = client.get_reward_event(event_id)?;
                units::create_in_app_notification_payloads(client, &event)
            })
            .await
    }

    async fn abuse_facts(&self, since: NaiveDateTime, min_referrals: i64, velocity_window_secs: i64) -> Result<Vec<AbuseFacts>, DatabaseError> {
        self.database
            .run(move |client| {
                let usernames = client.get_referrer_usernames_with_referrals(since, min_referrals)?;
                Ok(usernames.iter().filter_map(|username| units::abuse_facts(client, username, since, velocity_window_secs).ok()).collect())
            })
            .await
    }

    async fn disable_rewards(&self, username: String, reason: String, comment: String) -> Result<i32, DatabaseError> {
        self.database.run(move |client| client.disable_rewards(&username, &reason, &comment)).await
    }

    async fn usernames_with_status(&self, status: RewardStatus) -> Result<Vec<String>, DatabaseError> {
        self.database.run(move |client| client.get_usernames_by_filter(vec![RewardsFilter::Statuses(vec![status])])).await
    }

    async fn promote_if_eligible(&self, username: String, eligibility: RewardsEligibilityConfig) -> Result<Option<(i32, Vec<i32>)>, DatabaseError> {
        self.database
            .run(move |client| {
                let Some(wallet_id) = client.check_eligibility(&username, eligibility)? else {
                    return Ok(None);
                };
                Ok(Some((wallet_id, client.promote_to_verified(&username)?)))
            })
            .await
    }

    async fn redemption(&self, redemption_id: i32) -> Result<RedemptionRecord, DatabaseError> {
        self.database.run(move |client| client.get_redemption(redemption_id)).await
    }

    async fn start_redemption(&self, redemption_id: i32) -> Result<RedemptionStart, DatabaseError> {
        self.database
            .run(move |client| {
                let redemption = client.get_redemption(redemption_id)?;
                client.update_redemption(redemption_id, vec![RedemptionUpdate::Status(primitives::rewards::RedemptionStatus::Processing)])?;
                let recipient_address = client.get_address_by_username(&redemption.username)?;
                let option = client.get_redemption_option(&redemption.option_id)?;
                Ok(RedemptionStart { redemption, recipient_address, option })
            })
            .await
    }

    async fn update_redemption(&self, redemption_id: i32, updates: Vec<RedemptionUpdate>) -> Result<(), DatabaseError> {
        self.database.run(move |client| client.update_redemption(redemption_id, updates)).await
    }
}
