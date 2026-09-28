use chrono::NaiveDateTime;
use rewards::{ReferralError, RiskScoreConfig, RiskScoringInput, evaluate_risk};
use storage::{DatabaseClient, DatabaseError, RiskSignalsRepository};

pub enum RiskAssessment {
    Allowed { risk_signal_id: i32 },
    Exceeded { risk_signal_id: i32, error: ReferralError },
}

pub fn assess_referral_risk(client: &mut DatabaseClient, input: &RiskScoringInput, config: &RiskScoreConfig, since: NaiveDateTime) -> Result<Result<RiskAssessment, ReferralError>, DatabaseError> {
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
