mod mapper;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::rewards::{RedemptionResponse, RedemptionResult, RedemptionStatus, RewardRedemptionOption, RewardRedemptionType, RewardStatus};
use primitives::{NotificationRewardsMetadata, ReferralLeaderboard, RewardEvent, RewardEventType, Rewards, WalletId, WalletSource, WalletType, now};
use rewards::{
    NewReferralVerification, ReferralError, ReferralUseFacts, ReferralValidationError, ReferredRewards, RewardsRedemptionError, RiskScoreConfig, RiskScoringInput, UsernameRules, UsernameValidationError, available_redemption_options,
    evaluate_risk, new_referral_verification, offers_redemptions, referral_verification_delay, validate_username, validate_username_available, validate_wallet_without_username,
};
use storage::{
    Database, DatabaseError, NewWallet, RedemptionFilter, RedemptionRecord, RedemptionUpdate, ReferrerInfo, RewardsEligibilityConfig, RewardsFilter, RewardsRedemptionsRepository, RewardsRepository, RiskSignalsRepository, WalletRecord,
    WalletsRepository,
};
use streamer::InAppNotificationPayload;

pub(crate) use storage::AbuseFacts;

use super::config::ReferralVerificationConfig;
use super::redemption::redemption_rejection;
use super::risk::RiskAssessment;

pub(crate) enum ReferralCodeUse {
    Apply { referrer_username: String, referrer_status: RewardStatus },
    NeedsScoring(String),
}

pub(crate) struct ReferralCodeRequest {
    pub(crate) code: String,
    pub(crate) wallet_id: i32,
    pub(crate) device_id: i32,
    pub(crate) device_created_at: NaiveDateTime,
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

pub(crate) struct RedemptionStart {
    pub(crate) redemption: RedemptionRecord,
    pub(crate) recipient_address: String,
    pub(crate) option: RewardRedemptionOption,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn get_rewards(&self, wallet_id: i32, rules: UsernameRules) -> Result<Rewards, DatabaseError>;
    async fn get_referral_use_facts(&self, wallet_id: i32, device_id: i32) -> Result<ReferralUseFacts, DatabaseError>;
    async fn get_reward_events(&self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError>;
    async fn get_leaderboard(&self) -> Result<ReferralLeaderboard, DatabaseError>;
    async fn get_redemption_option(&self, code: String) -> Result<RewardRedemptionOption, DatabaseError>;
    async fn get_or_add_multicoin_wallet(&self, address: String) -> Result<WalletRecord, DatabaseError>;
    async fn add_username(&self, wallet_id: i32, username: String, rules: UsernameRules) -> Result<Result<i32, UsernameValidationError>, DatabaseError>;
    async fn get_referral_code_use(&self, request: ReferralCodeRequest) -> Result<Result<ReferralCodeUse, ReferralError>, DatabaseError>;
    async fn get_referrer_info(&self, username: String) -> Result<ReferrerInfo, DatabaseError>;
    async fn get_referral_use_check(&self, check: ReferralUseCheck) -> Result<Result<(), ReferralError>, DatabaseError>;
    async fn add_referral_risk_signal(&self, input: RiskScoringInput, config: RiskScoreConfig, since: NaiveDateTime) -> Result<Result<RiskAssessment, ReferralError>, DatabaseError>;
    async fn set_referral(
        &self,
        referrer_username: String,
        referrer_status: RewardStatus,
        wallet_id: i32,
        device_id: i32,
        risk_signal_id: Option<i32>,
        verification_config: ReferralVerificationConfig,
    ) -> Result<Result<Vec<RewardEvent>, ReferralError>, DatabaseError>;
    async fn add_referral_attempt(&self, referrer_username: String, wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: String) -> Result<(), DatabaseError>;
    async fn get_redemption_limit_check(&self, username: String, cooldown_since: NaiveDateTime, window_limits: Vec<(NaiveDateTime, i64)>) -> Result<Result<(), RewardsRedemptionError>, DatabaseError>;
    async fn add_redemption(&self, username: String, points: i32, option_id: String, device_id: i32, wallet_id: i32) -> Result<Result<RedemptionResponse, RewardsRedemptionError>, DatabaseError>;
    async fn get_event_notifications(&self, event_id: i32) -> Result<Vec<InAppNotificationPayload>, DatabaseError>;
    async fn get_abuse_facts(&self, since: NaiveDateTime, min_referrals: i64, velocity_window_secs: i64) -> Result<Vec<AbuseFacts>, DatabaseError>;
    async fn set_rewards_disabled(&self, username: String, reason: String, comment: String) -> Result<i32, DatabaseError>;
    async fn get_usernames_with_status(&self, status: RewardStatus) -> Result<Vec<String>, DatabaseError>;
    async fn update_rewards_verified_if_eligible(&self, username: String, eligibility: RewardsEligibilityConfig) -> Result<Option<(i32, Vec<i32>)>, DatabaseError>;
    async fn get_redemption(&self, redemption_id: i32) -> Result<RedemptionRecord, DatabaseError>;
    async fn update_redemption_processing(&self, redemption_id: i32) -> Result<RedemptionStart, DatabaseError>;
    async fn update_redemptions(&self, filters: Vec<RedemptionFilter>, updates: Vec<RedemptionUpdate>) -> Result<usize, DatabaseError>;
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
    async fn get_rewards(&self, wallet_id: i32, rules: UsernameRules) -> Result<Rewards, DatabaseError> {
        self.database
            .run(move |client| {
                let identity = mapper::reward_identity(client.get_or_add_reward_identity(wallet_id)?);
                let record = client.get_rewards_record(&identity.username)?;
                let redemption_options = if offers_redemptions(record.status) {
                    available_redemption_options(client.get_redemption_options(&[RewardRedemptionType::Asset])?)
                } else {
                    vec![]
                };
                Ok(mapper::rewards(identity, record, redemption_options, &rules))
            })
            .await
    }

    async fn get_referral_use_facts(&self, wallet_id: i32, device_id: i32) -> Result<ReferralUseFacts, DatabaseError> {
        self.database.run(move |client| Ok(mapper::referral_use_facts(client.get_referral_use_facts(wallet_id, device_id)?))).await
    }

    async fn get_reward_events(&self, wallet_id: i32) -> Result<Vec<RewardEvent>, DatabaseError> {
        self.database.run(move |client| client.get_reward_events_by_wallet_id(wallet_id)).await
    }

    async fn get_leaderboard(&self) -> Result<ReferralLeaderboard, DatabaseError> {
        self.database.run(RewardsRepository::get_rewards_leaderboard).await
    }

    async fn get_redemption_option(&self, code: String) -> Result<RewardRedemptionOption, DatabaseError> {
        self.database.run(move |client| client.get_redemption_option(&code)).await
    }

    async fn get_or_add_multicoin_wallet(&self, address: String) -> Result<WalletRecord, DatabaseError> {
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

    async fn add_username(&self, wallet_id: i32, username: String, rules: UsernameRules) -> Result<Result<i32, UsernameValidationError>, DatabaseError> {
        self.database
            .run(move |client| {
                if let Err(error) = validate_username(&username, &rules) {
                    return Ok(Err(error));
                }
                if let Err(error) = validate_username_available(client.get_referral_code(&username)?.is_some()) {
                    return Ok(Err(error));
                }
                if let Err(error) = validate_wallet_without_username(&mapper::reward_identity(client.get_or_add_reward_identity(wallet_id)?), &rules) {
                    return Ok(Err(error));
                }
                Ok(Ok(client.set_username(wallet_id, &username)?))
            })
            .await
    }

    async fn get_referral_code_use(&self, request: ReferralCodeRequest) -> Result<Result<ReferralCodeUse, ReferralError>, DatabaseError> {
        self.database
            .run(move |client| {
                let ReferralCodeRequest {
                    code,
                    wallet_id,
                    device_id,
                    device_created_at,
                } = request;
                let Some(referrer_username) = client.get_referral_code(&code)? else {
                    return Ok(Err(ReferralValidationError::CodeDoesNotExist.into()));
                };
                let referrer_info = client.get_referrer_info(&referrer_username)?;
                let facts = match client.get_referral_use_facts(wallet_id, device_id) {
                    Ok(record) => mapper::referral_use_facts(record),
                    Err(error) => return Ok(Err(ReferralError::internal(error))),
                };
                if facts.is_pending_referral(&referrer_username) {
                    if !referrer_info.status.is_verified() && referrer_info.status != RewardStatus::Attribution {
                        return Ok(Err(ReferralValidationError::RewardsNotEnabled(referrer_username).into()));
                    }
                    return Ok(Ok(ReferralCodeUse::Apply {
                        referrer_username,
                        referrer_status: referrer_info.status,
                    }));
                }
                if referrer_info.status == RewardStatus::Attribution {
                    if let Err(error) = facts.validate_use(&referrer_username, referrer_info.wallet_id, device_created_at, None, now()) {
                        return Ok(Err(error.into()));
                    }
                    return Ok(Ok(ReferralCodeUse::Apply {
                        referrer_username,
                        referrer_status: referrer_info.status,
                    }));
                }
                Ok(Ok(ReferralCodeUse::NeedsScoring(referrer_username)))
            })
            .await
    }

    async fn get_referrer_info(&self, username: String) -> Result<ReferrerInfo, DatabaseError> {
        self.database.run(move |client| client.get_referrer_info(&username)).await
    }

    async fn get_referral_use_check(&self, check: ReferralUseCheck) -> Result<Result<(), ReferralError>, DatabaseError> {
        self.database
            .run(move |client| {
                for (since, limit) in &check.window_limits {
                    if client.get_referrals_count_since(&check.referrer_username, *since)? >= *limit {
                        return Ok(Err(ReferralError::ReferrerLimitReached));
                    }
                }
                if client.get_referrals_count_since(&check.referrer_username, check.cooldown_since)? >= 1 {
                    return Ok(Err(ReferralError::ReferrerLimitReached));
                }
                let facts = mapper::referral_use_facts(client.get_referral_use_facts(check.wallet_id, check.device_id)?);
                Ok(facts
                    .validate_use(&check.referrer_username, check.referrer_wallet_id, check.device_created_at, Some(check.eligibility_days), now())
                    .map_err(ReferralError::from))
            })
            .await
    }

    async fn add_referral_risk_signal(&self, input: RiskScoringInput, config: RiskScoreConfig, since: NaiveDateTime) -> Result<Result<RiskAssessment, ReferralError>, DatabaseError> {
        self.database
            .run(move |client| {
                if client.get_disabled_users_count_by_device(input.device_id, since)? > 0 {
                    return Ok(Err(ReferralError::LimitReached));
                }

                let signal_input = input.to_signal_input();
                let fingerprint = signal_input.generate_fingerprint();
                if client.get_fingerprint_exists_for_referrer(&fingerprint, &input.username, since).unwrap_or(false) {
                    return Ok(Err(ReferralError::DuplicateAttempt));
                }

                let existing_signals = client.get_matching_risk_signals(&fingerprint, &signal_input.ip_address, &signal_input.ip_isp, &signal_input.device_model, signal_input.device_id, since)?;
                let device_model_ring_count = client.get_unique_referrers_count_for_device_model_pattern(&signal_input.device_model, signal_input.device_platform, &signal_input.device_locale, since)?;
                let ip_abuser_count = client.get_disabled_users_count_by_ip(&signal_input.ip_address, since)?;
                let cross_referrer_fingerprint_count = client.get_unique_referrers_count_for_fingerprint(&fingerprint, since)?;
                let referrer_country_count = client.get_unique_countries_count_for_referrer(&input.username, since)?;
                let referrer_device_count = client.get_unique_devices_count_for_referrer(&input.username, since)?;

                let risk_result = evaluate_risk(
                    &input,
                    &existing_signals,
                    device_model_ring_count,
                    ip_abuser_count,
                    cross_referrer_fingerprint_count,
                    referrer_country_count,
                    referrer_device_count,
                    &config,
                );
                let risk_signal_id = client.add_risk_signal(risk_result.signal)?;

                if !risk_result.score.is_allowed {
                    return Ok(Ok(RiskAssessment::Exceeded {
                        risk_signal_id,
                        error: ReferralError::RiskScoreExceeded {
                            score: risk_result.score.score,
                            max_allowed: config.max_allowed_score,
                        },
                    }));
                }
                Ok(Ok(RiskAssessment::Allowed { risk_signal_id }))
            })
            .await
    }

    async fn set_referral(
        &self,
        referrer_username: String,
        referrer_status: RewardStatus,
        wallet_id: i32,
        device_id: i32,
        risk_signal_id: Option<i32>,
        verification_config: ReferralVerificationConfig,
    ) -> Result<Result<Vec<RewardEvent>, ReferralError>, DatabaseError> {
        self.database
            .run(move |client| {
                let referred_username = client.get_or_add_reward_identity(wallet_id)?.username;
                let verification = client.get_rewards_verification(&referred_username)?;
                let referred = ReferredRewards {
                    status: verification.status,
                    verify_after: verification.verify_after,
                };
                let now = now();
                let can_verify = referred.can_verify_referral(now);
                if referred.clears_verification_delay(now) {
                    client.delete_rewards_verification_delay(&referred_username)?;
                }

                if let Some(record) = client.get_referral_by_referred_username(&referred_username)? {
                    let referral_id = record.id;
                    if let Err(error) = mapper::referral(record).validate_confirmation(&referrer_username, device_id) {
                        return Ok(Err(error.into()));
                    }
                    if !can_verify {
                        return Ok(Ok(vec![]));
                    }
                    return Ok(Ok(client.update_referral_verified(referral_id, &referrer_username, &referrer_status, &referred_username)?));
                }

                let delay = referral_verification_delay(verification_config.base_delay, verification_config.verified_multiplier, referrer_status);
                let verified_at = match new_referral_verification(can_verify, delay, now) {
                    NewReferralVerification::Verified => Some(now),
                    NewReferralVerification::Delayed { verify_after } => {
                        client.set_rewards_verification_delay(&referred_username, verify_after)?;
                        None
                    }
                };
                Ok(Ok(client.add_referral(&referrer_username, &referred_username, device_id, risk_signal_id, verified_at, &referrer_status)?))
            })
            .await
    }

    async fn add_referral_attempt(&self, referrer_username: String, wallet_id: i32, device_id: i32, risk_signal_id: Option<i32>, reason: String) -> Result<(), DatabaseError> {
        self.database.run(move |client| client.add_referral_attempt(&referrer_username, wallet_id, device_id, risk_signal_id, &reason)).await
    }

    async fn get_redemption_limit_check(&self, username: String, cooldown_since: NaiveDateTime, window_limits: Vec<(NaiveDateTime, i64)>) -> Result<Result<(), RewardsRedemptionError>, DatabaseError> {
        self.database
            .run(move |client| {
                if client.get_referrals_count_since(&username, cooldown_since)? > 0 {
                    return Ok(Err(RewardsRedemptionError::CooldownNotElapsed));
                }
                for (since, limit) in window_limits {
                    if client.get_redemptions_count_since(&username, since)? >= limit {
                        return Ok(Err(RewardsRedemptionError::LimitReached));
                    }
                }
                Ok(Ok(()))
            })
            .await
    }

    async fn add_redemption(&self, username: String, points: i32, option_id: String, device_id: i32, wallet_id: i32) -> Result<Result<RedemptionResponse, RewardsRedemptionError>, DatabaseError> {
        self.database
            .run(move |client| {
                if let Some(error) = redemption_rejection(points, &client.get_redemption_option(&option_id)?) {
                    return Ok(Err(error));
                }
                let redemption = client.add_redemption(&username, &option_id, device_id, wallet_id)?;
                let redemption_id = redemption.id;
                Ok(Ok(RedemptionResponse {
                    result: RedemptionResult { redemption },
                    redemption_id,
                }))
            })
            .await
    }

    async fn get_event_notifications(&self, event_id: i32) -> Result<Vec<InAppNotificationPayload>, DatabaseError> {
        self.database
            .run(move |client| {
                let event = client.get_reward_event(event_id)?;
                let metadata = serde_json::to_value(NotificationRewardsMetadata {
                    username: Some(event.username.clone()),
                    points: (event.points > 0).then_some(event.points),
                })
                .ok();
                let recipient = match event.event {
                    RewardEventType::Joined => client.get_referrer_username(&event.username)?,
                    RewardEventType::CreateUsername | RewardEventType::InvitePending | RewardEventType::InviteNew | RewardEventType::Enabled | RewardEventType::Disabled | RewardEventType::Redeemed => Some(event.username.clone()),
                };
                let Some(recipient) = recipient else {
                    return Ok(vec![]);
                };
                let wallet_id = client.get_wallet_id_by_username(&recipient)?;
                Ok(vec![InAppNotificationPayload::new(wallet_id, mapper::notification_type(event.event), metadata)])
            })
            .await
    }

    async fn get_abuse_facts(&self, since: NaiveDateTime, min_referrals: i64, velocity_window_secs: i64) -> Result<Vec<AbuseFacts>, DatabaseError> {
        self.database
            .run(move |client| {
                let usernames = client.get_referrer_usernames_with_referrals(since, min_referrals)?;
                Ok(usernames.iter().filter_map(|username| client.get_abuse_facts(username, since, velocity_window_secs).ok()).collect())
            })
            .await
    }

    async fn set_rewards_disabled(&self, username: String, reason: String, comment: String) -> Result<i32, DatabaseError> {
        self.database.run(move |client| client.set_rewards_disabled(&username, &reason, &comment)).await
    }

    async fn get_usernames_with_status(&self, status: RewardStatus) -> Result<Vec<String>, DatabaseError> {
        self.database.run(move |client| client.get_usernames_by_filter(vec![RewardsFilter::Statuses(vec![status])])).await
    }

    async fn update_rewards_verified_if_eligible(&self, username: String, eligibility: RewardsEligibilityConfig) -> Result<Option<(i32, Vec<i32>)>, DatabaseError> {
        self.database
            .run(move |client| {
                let Some(wallet_id) = client.get_eligible_wallet_id(&username, eligibility)? else {
                    return Ok(None);
                };
                Ok(Some((wallet_id, client.update_rewards_verified(&username)?)))
            })
            .await
    }

    async fn get_redemption(&self, redemption_id: i32) -> Result<RedemptionRecord, DatabaseError> {
        self.database.run(move |client| client.get_redemption(redemption_id)).await
    }

    async fn update_redemption_processing(&self, redemption_id: i32) -> Result<RedemptionStart, DatabaseError> {
        self.database
            .run(move |client| {
                let redemption = client.get_redemption(redemption_id)?;
                client.update_redemptions(vec![RedemptionFilter::Ids(vec![redemption_id])], vec![RedemptionUpdate::Status(RedemptionStatus::Processing)])?;
                let recipient_address = client.get_address_by_username(&redemption.username)?;
                let option = client.get_redemption_option(&redemption.option_id)?;
                Ok(RedemptionStart { redemption, recipient_address, option })
            })
            .await
    }

    async fn update_redemptions(&self, filters: Vec<RedemptionFilter>, updates: Vec<RedemptionUpdate>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_redemptions(filters, updates)).await
    }
}
