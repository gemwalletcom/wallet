use std::collections::HashSet;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{NewRiskSignal, Platform as PrimitivePlatform, RiskSignal};

use crate::models::{NewRiskSignalRow, RiskSignalRow};
use crate::sql_types::{Platform, RewardStatus};
use crate::{DatabaseClient, DatabaseError};

#[derive(Debug, Clone, Default)]
pub struct AbusePatterns {
    pub max_countries_per_device: i64,
    pub max_referrers_per_device: i64,
    pub max_referrers_per_fingerprint: i64,
    pub max_devices_per_ip: i64,
    pub signals_in_velocity_window: i64,
}

pub trait RiskSignalsRepository {
    fn add_risk_signal(&mut self, signal: NewRiskSignal) -> Result<i32, DatabaseError>;
    fn has_fingerprint_for_referrer(&mut self, fingerprint: &str, referrer_username: &str, since: NaiveDateTime) -> Result<bool, DatabaseError>;
    fn get_matching_risk_signals(&mut self, fingerprint: &str, ip_address: &str, ip_isp: &str, device_model: &str, device_id: i32, since: NaiveDateTime) -> Result<Vec<RiskSignal>, DatabaseError>;
    fn sum_risk_scores_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_attempts_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn get_referrer_usernames_with_referrals(&mut self, since: NaiveDateTime, min_referrals: i64) -> Result<Vec<String>, DatabaseError>;
    fn count_unique_countries_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_referrers_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_referrers_for_fingerprint(&mut self, fingerprint: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_devices_for_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_referrers_for_device_model_pattern(&mut self, device_model: &str, device_platform: PrimitivePlatform, device_locale: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn get_abuse_patterns_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime, velocity_window_secs: i64) -> Result<AbusePatterns, DatabaseError>;
    fn count_disabled_users_by_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_disabled_users_by_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_countries_for_referrer(&mut self, username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
    fn count_unique_devices_for_referrer(&mut self, username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError>;
}

impl RiskSignalsRepository for DatabaseClient {
    fn add_risk_signal(&mut self, signal: NewRiskSignal) -> Result<i32, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        let signal = NewRiskSignalRow {
            fingerprint: signal.fingerprint,
            referrer_username: signal.referrer_username,
            device_id: signal.device_id,
            device_platform: signal.device_platform.into(),
            device_platform_store: signal.device_platform_store.into(),
            device_os: signal.device_os,
            device_model: signal.device_model,
            device_locale: signal.device_locale,
            device_currency: signal.device_currency,
            ip_address: signal.ip_address,
            ip_country_code: signal.ip_country_code,
            ip_usage_type: signal.ip_usage_type.into(),
            ip_isp: signal.ip_isp,
            ip_abuse_score: signal.ip_abuse_score,
            risk_score: signal.risk_score,
            user_agent: signal.user_agent,
            metadata: Some(signal.metadata),
        };
        Ok(diesel::insert_into(dsl::rewards_risk_signals).values(&signal).returning(dsl::id).get_result(&mut self.connection)?)
    }

    fn has_fingerprint_for_referrer(&mut self, fingerprint: &str, referrer_username: &str, since: NaiveDateTime) -> Result<bool, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::exists;

        Ok(diesel::select(exists(
            dsl::rewards_risk_signals
                .filter(dsl::fingerprint.eq(fingerprint))
                .filter(dsl::referrer_username.eq(referrer_username))
                .filter(dsl::created_at.ge(since)),
        ))
        .get_result(&mut self.connection)?)
    }

    fn get_matching_risk_signals(&mut self, fingerprint: &str, ip_address: &str, ip_isp: &str, device_model: &str, device_id: i32, since: NaiveDateTime) -> Result<Vec<RiskSignal>, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;

        let rows = dsl::rewards_risk_signals
            .filter(dsl::created_at.ge(since))
            .filter(
                dsl::fingerprint
                    .eq(fingerprint)
                    .or(dsl::ip_address.eq(ip_address))
                    .or(dsl::ip_isp.eq(ip_isp).and(dsl::device_model.eq(device_model)))
                    .or(dsl::device_id.eq(device_id)),
            )
            .order(dsl::created_at.desc())
            .limit(100)
            .select(RiskSignalRow::as_select())
            .load(&mut self.connection)?;
        Ok(rows
            .into_iter()
            .map(|row| RiskSignal {
                fingerprint: row.fingerprint,
                referrer_username: row.referrer_username,
                device_id: row.device_id,
                device_platform: row.device_platform.0,
                device_model: row.device_model,
                ip_address: row.ip_address,
                ip_isp: row.ip_isp,
                created_at: row.created_at,
            })
            .collect())
    }

    fn sum_risk_scores_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::sum;

        Ok(dsl::rewards_risk_signals
            .filter(dsl::referrer_username.eq(referrer_username))
            .filter(dsl::created_at.ge(since))
            .select(sum(dsl::risk_score))
            .first::<Option<i64>>(&mut self.connection)
            .map(|s| s.unwrap_or(0))?)
    }

    fn count_attempts_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_referral_attempts::dsl;

        Ok(dsl::rewards_referral_attempts
            .filter(dsl::referrer_username.eq(referrer_username))
            .filter(dsl::created_at.ge(since))
            .filter(dsl::risk_signal_id.is_not_null())
            .count()
            .get_result(&mut self.connection)?)
    }

    fn get_referrer_usernames_with_referrals(&mut self, since: NaiveDateTime, min_referrals: i64) -> Result<Vec<String>, DatabaseError> {
        use crate::schema::{rewards, rewards_referrals};
        use diesel::dsl::count_star;

        Ok(rewards_referrals::table
            .inner_join(rewards::table.on(rewards_referrals::referrer_username.eq(rewards::username)))
            .filter(rewards::status.ne(RewardStatus::Attribution))
            .filter(rewards::status.ne(RewardStatus::Disabled))
            .filter(rewards_referrals::created_at.ge(since))
            .group_by(rewards_referrals::referrer_username)
            .having(count_star().ge(min_referrals))
            .select(rewards_referrals::referrer_username)
            .load(&mut self.connection)?)
    }

    fn count_unique_countries_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(dsl::rewards_risk_signals
            .filter(dsl::device_id.eq(device_id))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::ip_country_code).aggregate_distinct())
            .first(&mut self.connection)?)
    }

    fn count_unique_referrers_for_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(dsl::rewards_risk_signals
            .filter(dsl::device_id.eq(device_id))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::referrer_username).aggregate_distinct())
            .first(&mut self.connection)?)
    }

    fn count_unique_referrers_for_fingerprint(&mut self, fingerprint: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(dsl::rewards_risk_signals
            .filter(dsl::fingerprint.eq(fingerprint))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::referrer_username).aggregate_distinct())
            .first(&mut self.connection)?)
    }

    fn count_unique_devices_for_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(dsl::rewards_risk_signals
            .filter(dsl::ip_address.eq(ip_address))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::device_id).aggregate_distinct())
            .first(&mut self.connection)?)
    }

    fn count_unique_referrers_for_device_model_pattern(&mut self, device_model: &str, device_platform: PrimitivePlatform, device_locale: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(dsl::rewards_risk_signals
            .filter(dsl::device_model.eq(device_model))
            .filter(dsl::device_platform.eq(Platform::from(device_platform)))
            .filter(dsl::device_locale.eq(device_locale))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::referrer_username).aggregate_distinct())
            .first(&mut self.connection)?)
    }

    fn get_abuse_patterns_for_referrer(&mut self, referrer_username: &str, since: NaiveDateTime, velocity_window_secs: i64) -> Result<AbusePatterns, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;

        let signals: Vec<RiskSignalRow> = dsl::rewards_risk_signals
            .filter(dsl::referrer_username.eq(referrer_username))
            .filter(dsl::created_at.ge(since))
            .select(RiskSignalRow::as_select())
            .load(&mut self.connection)?;

        if signals.is_empty() {
            return Ok(AbusePatterns::default());
        }

        let unique_devices: HashSet<i32> = signals.iter().map(|s| s.device_id).collect();
        let unique_fingerprints: HashSet<&str> = signals.iter().map(|s| s.fingerprint.as_str()).collect();
        let unique_ips: HashSet<&str> = signals.iter().map(|s| s.ip_address.as_str()).collect();

        let mut max_countries_per_device: i64 = 0;
        let mut max_referrers_per_device: i64 = 0;
        let mut max_referrers_per_fingerprint: i64 = 0;
        let mut max_devices_per_ip: i64 = 0;

        for device_id in unique_devices {
            let countries = self.count_unique_countries_for_device(device_id, since)?;
            max_countries_per_device = max_countries_per_device.max(countries);

            let referrers = self.count_unique_referrers_for_device(device_id, since)?;
            max_referrers_per_device = max_referrers_per_device.max(referrers);
        }

        for fingerprint in unique_fingerprints {
            let referrers = self.count_unique_referrers_for_fingerprint(fingerprint, since)?;
            max_referrers_per_fingerprint = max_referrers_per_fingerprint.max(referrers);
        }

        for ip_address in unique_ips {
            let devices = self.count_unique_devices_for_ip(ip_address, since)?;
            max_devices_per_ip = max_devices_per_ip.max(devices);
        }

        let max_signals_in_velocity_window = calculate_max_signals_in_window(&signals, velocity_window_secs);

        Ok(AbusePatterns {
            max_countries_per_device,
            max_referrers_per_device,
            max_referrers_per_fingerprint,
            max_devices_per_ip,
            signals_in_velocity_window: max_signals_in_velocity_window,
        })
    }

    fn count_disabled_users_by_ip(&mut self, ip_address: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::{rewards, rewards_risk_signals};
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(rewards_risk_signals::table
            .inner_join(rewards::table.on(rewards_risk_signals::referrer_username.eq(rewards::username)))
            .filter(rewards_risk_signals::ip_address.eq(ip_address))
            .filter(rewards_risk_signals::created_at.ge(since))
            .filter(rewards::status.eq(RewardStatus::Disabled))
            .select(count(rewards_risk_signals::referrer_username).aggregate_distinct())
            .first(&mut self.connection)?)
    }

    fn count_disabled_users_by_device(&mut self, device_id: i32, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::{rewards, rewards_risk_signals};
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(rewards_risk_signals::table
            .inner_join(rewards::table.on(rewards_risk_signals::referrer_username.eq(rewards::username)))
            .filter(rewards_risk_signals::device_id.eq(device_id))
            .filter(rewards_risk_signals::created_at.ge(since))
            .filter(rewards::status.eq(RewardStatus::Disabled))
            .select(count(rewards_risk_signals::referrer_username).aggregate_distinct())
            .first(&mut self.connection)?)
    }

    fn count_unique_countries_for_referrer(&mut self, username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(dsl::rewards_risk_signals
            .filter(dsl::referrer_username.eq(username))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::ip_country_code).aggregate_distinct())
            .first(&mut self.connection)?)
    }

    fn count_unique_devices_for_referrer(&mut self, username: &str, since: NaiveDateTime) -> Result<i64, DatabaseError> {
        use crate::schema::rewards_risk_signals::dsl;
        use diesel::dsl::count;
        use diesel::expression_methods::AggregateExpressionMethods;

        Ok(dsl::rewards_risk_signals
            .filter(dsl::referrer_username.eq(username))
            .filter(dsl::created_at.ge(since))
            .select(count(dsl::device_id).aggregate_distinct())
            .first(&mut self.connection)?)
    }
}

fn calculate_max_signals_in_window(signals: &[RiskSignalRow], window_secs: i64) -> i64 {
    if signals.is_empty() {
        return 0;
    }

    let mut timestamps: Vec<_> = signals.iter().map(|s| s.created_at).collect();
    timestamps.sort();

    let mut max_count: i64 = 1;
    let mut left = 0;

    for right in 0..timestamps.len() {
        while timestamps[right].signed_duration_since(timestamps[left]).num_seconds() > window_secs {
            left += 1;
        }
        max_count = max_count.max((right - left + 1) as i64);
    }

    max_count
}
