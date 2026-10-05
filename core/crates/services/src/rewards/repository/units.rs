use chrono::NaiveDateTime;
use primitives::rewards::{RedemptionResponse, RedemptionResult, RewardRedemptionType, RewardStatus};
use primitives::{Chain, NotificationRewardsMetadata, NotificationType, RewardEvent, RewardEventType, Rewards, now};
use rewards::{
    DeviceWallet, NewReferralVerification, Referral, ReferralError, ReferralUseFacts, ReferredRewards, RewardIdentity, RewardsRedemptionError, RiskScoreConfig, RiskScoringInput, UsernameRules, UsernameValidationError,
    available_redemption_options, evaluate_risk, invite_reward_points, new_referral_verification, offers_redemptions, referral_verification_delay, validate_username, validate_username_available, validate_wallet_without_username,
};
use storage::{DatabaseClient, DatabaseError, ReferralRecord, RewardIdentityRecord, RewardsRedemptionsRepository, RewardsRepository, RiskSignalsRepository, WalletsRepository};
use streamer::InAppNotificationPayload;

use crate::rewards::config::ReferralVerificationConfig;
use crate::rewards::redemption::redemption_rejection;
use crate::rewards::risk::RiskAssessment;

fn referral(record: ReferralRecord) -> Referral {
    Referral {
        referrer_username: record.referrer_username,
        referred_username: record.referred_username,
        referred_device_id: record.referred_device_id,
        is_verified: record.verified_at.is_some(),
    }
}

pub(super) fn referral_use_facts(client: &mut DatabaseClient, wallet_id: i32, device_id: i32) -> Result<ReferralUseFacts, DatabaseError> {
    let referred_username = client.get_referred_username(wallet_id)?;
    let referred_status = client.get_rewards_verification(&referred_username).ok().map(|verification| verification.status);
    let wallet_first_subscription_at = client.get_first_subscription_date_by_wallet_id(wallet_id)?;
    let device_wallets = client
        .get_device_multicoin_wallet_ids(device_id, Chain::Ethereum)?
        .into_iter()
        .map(|wallet_id| {
            Ok(DeviceWallet {
                wallet_id,
                first_subscription_at: client.get_first_subscription_date_by_wallet_id(wallet_id)?,
            })
        })
        .collect::<Result<Vec<_>, DatabaseError>>()?;
    let device_referral = client.get_referral_by_referred_device(device_id)?.map(referral);
    Ok(ReferralUseFacts {
        referred_username,
        referred_status,
        wallet_first_subscription_at,
        device_wallets,
        device_referral,
    })
}

pub(super) fn use_or_verify_referral(
    client: &mut DatabaseClient,
    referrer_username: &str,
    referrer_status: RewardStatus,
    wallet_id: i32,
    device_id: i32,
    risk_signal_id: Option<i32>,
    verification_config: ReferralVerificationConfig,
) -> Result<Result<Vec<RewardEvent>, ReferralError>, DatabaseError> {
    let referred_username = client.ensure_reward_identity(wallet_id)?.username;
    let verification = client.get_rewards_verification(&referred_username)?;
    let referred = ReferredRewards {
        status: verification.status,
        verify_after: verification.verify_after,
    };
    let now = now();
    let can_verify = referred.can_verify_referral(now);
    if referred.clears_verification_delay(now) {
        client.clear_rewards_verification_delay(&referred_username)?;
    }

    if let Some(record) = client.get_referral_by_referred_username(&referred_username)? {
        let referral_id = record.id;
        if let Err(error) = referral(record).validate_confirmation(referrer_username, device_id) {
            return Ok(Err(error.into()));
        }
        if !can_verify {
            return Ok(Ok(vec![]));
        }
        return Ok(Ok(client.verify_referral(referral_id, referrer_username, &referrer_status, &referred_username)?));
    }

    let delay = referral_verification_delay(verification_config.base_delay, verification_config.verified_multiplier, referrer_status);
    let verified_at = match new_referral_verification(can_verify, delay, now) {
        NewReferralVerification::Verified => Some(now),
        NewReferralVerification::Delayed { verify_after } => {
            client.delay_rewards_verification(&referred_username, verify_after)?;
            None
        }
    };
    Ok(Ok(client.record_referral(referrer_username, &referred_username, device_id, risk_signal_id, verified_at, &referrer_status)?))
}

fn reward_identity(record: RewardIdentityRecord) -> RewardIdentity {
    RewardIdentity {
        username: record.username,
        wallet_address: record.wallet_address,
    }
}

pub(crate) fn create_username(client: &mut DatabaseClient, wallet_id: i32, username: &str, rules: &UsernameRules) -> Result<Result<(Rewards, i32), UsernameValidationError>, DatabaseError> {
    if let Err(error) = validate_username(username, rules) {
        return Ok(Err(error));
    }
    if let Err(error) = validate_username_available(client.get_referral_code(username)?.is_some()) {
        return Ok(Err(error));
    }
    if let Err(error) = validate_wallet_without_username(&reward_identity(client.ensure_reward_identity(wallet_id)?), rules) {
        return Ok(Err(error));
    }
    let event_id = client.set_username(wallet_id, username)?;
    Ok(Ok((rewards_by_wallet_id(client, wallet_id, rules)?, event_id)))
}

pub(super) fn rewards_by_wallet_id(client: &mut DatabaseClient, wallet_id: i32, rules: &UsernameRules) -> Result<Rewards, DatabaseError> {
    let identity = reward_identity(client.ensure_reward_identity(wallet_id)?);
    let record = client.get_rewards_record(&identity.username)?;
    let redemption_options = if offers_redemptions(record.status) {
        available_redemption_options(client.get_redemption_options(&[RewardRedemptionType::Asset])?)
    } else {
        vec![]
    };

    Ok(Rewards {
        code: identity.referral_code(rules),
        invite_reward_points: invite_reward_points(record.status),
        referral_count: record.referral_count,
        points: record.points,
        used_referral_code: record.referrer_username,
        status: record.status,
        created_at: record.created_at,
        verify_after: record.verify_after.map(|verify_after| verify_after.and_utc()),
        redemption_options,
        disable_reason: record.disable_reason,
        referral_allowance: Default::default(),
        use_referral_code_until: None,
    })
}

pub(super) fn redeem_points(client: &mut DatabaseClient, username: &str, points: i32, option_id: &str, device_id: i32, wallet_id: i32) -> Result<Result<RedemptionResponse, RewardsRedemptionError>, DatabaseError> {
    if let Some(error) = redemption_rejection(points, &client.get_redemption_option(option_id)?) {
        return Ok(Err(error));
    }
    let redemption = client.add_redemption(username, option_id, device_id, wallet_id)?;
    let redemption_id = redemption.id;
    Ok(Ok(RedemptionResponse {
        result: RedemptionResult { redemption },
        redemption_id,
    }))
}

pub(super) fn assess_referral_risk(client: &mut DatabaseClient, input: &RiskScoringInput, config: &RiskScoreConfig, since: NaiveDateTime) -> Result<Result<RiskAssessment, ReferralError>, DatabaseError> {
    if client.count_disabled_users_by_device(input.device_id, since)? > 0 {
        return Ok(Err(ReferralError::LimitReached));
    }

    let signal_input = input.to_signal_input();
    let fingerprint = signal_input.generate_fingerprint();
    if client.has_fingerprint_for_referrer(&fingerprint, &input.username, since).unwrap_or(false) {
        return Ok(Err(ReferralError::DuplicateAttempt));
    }

    let existing_signals = client.get_matching_risk_signals(&fingerprint, &signal_input.ip_address, &signal_input.ip_isp, &signal_input.device_model, signal_input.device_id, since)?;
    let device_model_ring_count = client.count_unique_referrers_for_device_model_pattern(&signal_input.device_model, signal_input.device_platform, &signal_input.device_locale, since)?;
    let ip_abuser_count = client.count_disabled_users_by_ip(&signal_input.ip_address, since)?;
    let cross_referrer_fingerprint_count = client.count_unique_referrers_for_fingerprint(&fingerprint, since)?;
    let referrer_country_count = client.count_unique_countries_for_referrer(&input.username, since)?;
    let referrer_device_count = client.count_unique_devices_for_referrer(&input.username, since)?;

    let risk_result = evaluate_risk(
        input,
        &existing_signals,
        device_model_ring_count,
        ip_abuser_count,
        cross_referrer_fingerprint_count,
        referrer_country_count,
        referrer_device_count,
        config,
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
}

pub(super) fn create_in_app_notification_payloads(client: &mut DatabaseClient, event: &RewardEvent) -> Result<Vec<InAppNotificationPayload>, DatabaseError> {
    let metadata = NotificationRewardsMetadata {
        username: Some(event.username.clone()),
        points: (event.points > 0).then_some(event.points),
    };
    let metadata_value = serde_json::to_value(metadata).ok();

    match event.event {
        RewardEventType::CreateUsername => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsCreateUsername, metadata_value)])
        }
        RewardEventType::InvitePending | RewardEventType::InviteNew => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsInvite, metadata_value)])
        }
        RewardEventType::Joined => {
            let Some(referrer) = client.get_referrer_username(&event.username)? else {
                return Ok(vec![]);
            };
            let wallet_id = client.get_wallet_id_by_username(&referrer)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::ReferralJoined, metadata_value)])
        }
        RewardEventType::Enabled => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsEnabled, metadata_value)])
        }
        RewardEventType::Disabled => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsCodeDisabled, metadata_value)])
        }
        RewardEventType::Redeemed => {
            let wallet_id = client.get_wallet_id_by_username(&event.username)?;
            Ok(vec![InAppNotificationPayload::new(wallet_id, NotificationType::RewardsRedeemed, metadata_value)])
        }
    }
}

pub(super) fn abuse_facts(client: &mut DatabaseClient, username: &str, since: NaiveDateTime, velocity_window_secs: i64) -> Result<super::AbuseFacts, DatabaseError> {
    let status = client.get_status_by_username(username)?;
    let referral_count = client.count_referrals_since(username, since)?;
    let attempt_count = client.count_attempts_for_referrer(username, since)?;
    let risk_score_sum = client.sum_risk_scores_for_referrer(username, since)?;
    let patterns = client.get_abuse_patterns_for_referrer(username, since, velocity_window_secs)?;
    let referrer_disabled = client
        .get_referrer_username(username)
        .ok()
        .flatten()
        .and_then(|referrer| client.get_status_by_username(&referrer).ok())
        .is_some_and(|status| status == RewardStatus::Disabled);
    Ok(super::AbuseFacts {
        username: username.to_string(),
        status,
        referral_count,
        attempt_count,
        risk_score_sum,
        patterns,
        referrer_disabled,
    })
}
