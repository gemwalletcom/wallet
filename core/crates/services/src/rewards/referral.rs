use std::error::Error;
use std::time::Duration;

use config_keys::ConfigKey;
use primitives::rewards::RewardStatus;
use primitives::{Chain, RewardEvent, now};
use rewards::{DeviceWallet, NewReferralVerification, Referral, ReferralError, ReferralUseFacts, ReferredRewards, new_referral_verification, referral_verification_delay};
use storage::{DatabaseClient, DatabaseError, ReferralRecord, RewardsRepository, WalletsRepository};

use crate::ConfigCacher;

#[derive(Debug, Clone, Copy)]
pub struct ReferralVerificationConfig {
    base_delay: Duration,
    verified_multiplier: i64,
}

impl ReferralVerificationConfig {
    pub async fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        Ok(Self {
            base_delay: config.get_duration(ConfigKey::ReferralVerificationDelay).await?,
            verified_multiplier: config.get_i64(ConfigKey::ReferralVerifiedMultiplier).await?,
        })
    }
}

fn referral(record: ReferralRecord) -> Referral {
    Referral {
        referrer_username: record.referrer_username,
        referred_username: record.referred_username,
        referred_device_id: record.referred_device_id,
        is_verified: record.verified_at.is_some(),
    }
}

pub fn referral_use_facts(client: &mut DatabaseClient, wallet_id: i32, device_id: i32) -> Result<ReferralUseFacts, DatabaseError> {
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

pub fn use_or_verify_referral(
    client: &mut DatabaseClient,
    referrer_username: &str,
    referrer_status: RewardStatus,
    wallet_id: i32,
    device_id: i32,
    risk_signal_id: Option<i32>,
    verification_config: ReferralVerificationConfig,
) -> Result<Vec<RewardEvent>, Box<dyn Error + Send + Sync>> {
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
        referral(record).validate_confirmation(referrer_username, device_id).map_err(ReferralError::from)?;
        if !can_verify {
            return Ok(vec![]);
        }
        return Ok(client.verify_referral(referral_id, referrer_username, &referrer_status, &referred_username)?);
    }

    let delay = referral_verification_delay(verification_config.base_delay, verification_config.verified_multiplier, referrer_status);
    let verified_at = match new_referral_verification(can_verify, delay, now) {
        NewReferralVerification::Verified => Some(now),
        NewReferralVerification::Delayed { verify_after } => {
            client.delay_rewards_verification(&referred_username, verify_after)?;
            None
        }
    };
    Ok(client.record_referral(referrer_username, &referred_username, device_id, risk_signal_id, verified_at, &referrer_status)?)
}
