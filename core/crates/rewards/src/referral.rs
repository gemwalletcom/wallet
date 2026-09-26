use std::time::Duration;

use chrono::{NaiveDateTime, TimeDelta};
use primitives::rewards::RewardStatus;

use crate::error::{ReferralConfirmationError, ReferralValidationError};

#[derive(Debug, Clone, PartialEq)]
pub struct Referral {
    pub referrer_username: String,
    pub referred_username: String,
    pub referred_device_id: i32,
    pub is_verified: bool,
}

impl Referral {
    fn is_pending_confirmation(&self, referrer_username: &str, referred_username: &str) -> bool {
        !self.is_verified && self.referrer_username == referrer_username && self.referred_username == referred_username
    }

    pub fn validate_confirmation(&self, referrer_username: &str, device_id: i32) -> Result<(), ReferralConfirmationError> {
        if self.is_verified {
            return Err(ReferralConfirmationError::AlreadyVerified);
        }
        if self.referrer_username != referrer_username {
            return Err(ReferralConfirmationError::CodeMismatch);
        }
        if self.referred_device_id != device_id {
            return Err(ReferralConfirmationError::DeviceMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceWallet {
    pub wallet_id: i32,
    pub first_subscription_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferralUseFacts {
    pub referred_username: String,
    pub referred_status: Option<RewardStatus>,
    pub wallet_first_subscription_at: Option<NaiveDateTime>,
    pub device_wallets: Vec<DeviceWallet>,
    pub device_referral: Option<Referral>,
}

impl ReferralUseFacts {
    pub fn is_pending_referral(&self, referrer_username: &str) -> bool {
        self.referred_status == Some(RewardStatus::Pending) && self.device_referral.as_ref().is_some_and(|referral| referral.is_pending_confirmation(referrer_username, &self.referred_username))
    }

    pub fn eligibility_ends_at(&self, device_created_at: NaiveDateTime, days: i64) -> NaiveDateTime {
        self.device_wallets
            .iter()
            .filter_map(|wallet| wallet.first_subscription_at)
            .chain(self.wallet_first_subscription_at)
            .fold(device_created_at, NaiveDateTime::min)
            + TimeDelta::days(days)
    }

    pub fn validate_use(&self, referrer_username: &str, referrer_wallet_id: i32, device_created_at: NaiveDateTime, eligibility_days: Option<i64>, now: NaiveDateTime) -> Result<(), ReferralValidationError> {
        if let Some(days) = eligibility_days.filter(|days| now >= self.eligibility_ends_at(device_created_at, *days)) {
            return Err(ReferralValidationError::EligibilityExpired(days));
        }

        if self.device_wallets.iter().any(|wallet| wallet.wallet_id == referrer_wallet_id) {
            return Err(ReferralValidationError::CannotReferSelf);
        }

        if self.device_referral.as_ref().is_some_and(|referral| !referral.is_pending_confirmation(referrer_username, &self.referred_username)) {
            return Err(ReferralValidationError::DeviceAlreadyUsed);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReferredRewards {
    pub status: RewardStatus,
    pub verify_after: Option<NaiveDateTime>,
}

impl ReferredRewards {
    pub fn can_verify_referral(&self, now: NaiveDateTime) -> bool {
        self.status.is_verified() || self.verify_after.is_some_and(|verify_after| verify_after <= now)
    }

    pub fn clears_verification_delay(&self, now: NaiveDateTime) -> bool {
        self.can_verify_referral(now) && !self.status.is_verified()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewReferralVerification {
    Verified,
    Delayed { verify_after: NaiveDateTime },
}

pub fn referral_verification_delay(base_delay: Duration, verified_multiplier: i64, referrer_status: RewardStatus) -> Option<Duration> {
    let multiplier = if referrer_status.is_verified() || referrer_status == RewardStatus::Attribution { verified_multiplier } else { 1 };
    if referrer_status == RewardStatus::Trusted || multiplier <= 0 {
        return None;
    }
    Some(Duration::from_secs(base_delay.as_secs() / multiplier as u64))
}

pub fn new_referral_verification(can_verify: bool, delay: Option<Duration>, now: NaiveDateTime) -> NewReferralVerification {
    match delay {
        Some(delay) if !can_verify => NewReferralVerification::Delayed {
            verify_after: now + TimeDelta::seconds(delay.as_secs() as i64),
        },
        _ => NewReferralVerification::Verified,
    }
}

#[cfg(test)]
mod tests {
    use primitives::{DAY, HOUR, now};

    use super::*;

    #[test]
    fn test_is_pending_referral() {
        let pending = ReferralUseFacts {
            referred_status: Some(RewardStatus::Pending),
            device_referral: Some(Referral::mock()),
            ..ReferralUseFacts::mock()
        };

        assert!(pending.is_pending_referral("alice"));
        assert!(!pending.is_pending_referral("charlie"));
        assert!(
            !ReferralUseFacts {
                referred_status: Some(RewardStatus::Unverified),
                ..pending.clone()
            }
            .is_pending_referral("alice")
        );
        assert!(!ReferralUseFacts { referred_status: None, ..pending.clone() }.is_pending_referral("alice"));
        assert!(
            !ReferralUseFacts {
                referred_username: "dave".to_string(),
                ..pending.clone()
            }
            .is_pending_referral("alice")
        );
        assert!(
            !ReferralUseFacts {
                device_referral: Some(Referral { is_verified: true, ..Referral::mock() }),
                ..pending.clone()
            }
            .is_pending_referral("alice")
        );
        assert!(!ReferralUseFacts { device_referral: None, ..pending }.is_pending_referral("alice"));
    }

    #[test]
    fn test_validate_use() {
        let now = now();
        let recent = now - TimeDelta::days(1);
        let old = now - TimeDelta::days(60);
        let facts = ReferralUseFacts::mock();

        assert_eq!(facts.validate_use("alice", 1, recent, Some(30), now), Ok(()));
        assert_eq!(facts.validate_use("alice", 1, old, Some(30), now), Err(ReferralValidationError::EligibilityExpired(30)));
        assert_eq!(facts.validate_use("alice", 1, old, None, now), Ok(()));
        assert_eq!(
            ReferralUseFacts {
                wallet_first_subscription_at: Some(old),
                ..facts.clone()
            }
            .validate_use("alice", 1, recent, Some(30), now),
            Err(ReferralValidationError::EligibilityExpired(30))
        );

        let device_wallets = vec![
            DeviceWallet {
                wallet_id: 2,
                first_subscription_at: Some(old),
            },
            DeviceWallet {
                wallet_id: 1,
                first_subscription_at: Some(recent),
            },
        ];
        let shared_device = ReferralUseFacts { device_wallets, ..facts.clone() };
        assert_eq!(shared_device.validate_use("alice", 1, recent, Some(30), now), Err(ReferralValidationError::EligibilityExpired(30)));
        assert_eq!(shared_device.validate_use("alice", 1, recent, None, now), Err(ReferralValidationError::CannotReferSelf));
        assert_eq!(shared_device.validate_use("alice", 3, recent, None, now), Ok(()));

        let used_device = ReferralUseFacts {
            device_referral: Some(Referral::mock()),
            ..facts
        };
        assert_eq!(used_device.validate_use("alice", 1, recent, Some(30), now), Ok(()));
        assert_eq!(used_device.validate_use("charlie", 1, recent, Some(30), now), Err(ReferralValidationError::DeviceAlreadyUsed));
        assert_eq!(
            ReferralUseFacts {
                device_referral: Some(Referral { is_verified: true, ..Referral::mock() }),
                ..used_device
            }
            .validate_use("alice", 1, recent, Some(30), now),
            Err(ReferralValidationError::DeviceAlreadyUsed)
        );
    }

    #[test]
    fn test_eligibility_ends_at_counts_from_the_oldest_device_or_wallet() {
        let now = now();
        let facts = ReferralUseFacts::mock();
        assert_eq!(facts.eligibility_ends_at(now, 30), now + TimeDelta::days(30));

        let device_wallets = vec![DeviceWallet {
            wallet_id: 2,
            first_subscription_at: Some(now - TimeDelta::days(10)),
        }];
        let shared_device = ReferralUseFacts {
            wallet_first_subscription_at: Some(now - TimeDelta::days(5)),
            device_wallets,
            ..facts
        };
        assert_eq!(shared_device.eligibility_ends_at(now - TimeDelta::days(1), 30), now + TimeDelta::days(20));
    }

    #[test]
    fn test_validate_confirmation() {
        let referral = Referral::mock();

        assert_eq!(referral.validate_confirmation("alice", 10), Ok(()));
        assert_eq!(referral.validate_confirmation("charlie", 10), Err(ReferralConfirmationError::CodeMismatch));
        assert_eq!(referral.validate_confirmation("alice", 11), Err(ReferralConfirmationError::DeviceMismatch));
        assert_eq!(Referral { is_verified: true, ..referral }.validate_confirmation("charlie", 11), Err(ReferralConfirmationError::AlreadyVerified));
    }

    #[test]
    fn test_can_verify_referral() {
        let now = now();
        let past = Some(now - TimeDelta::hours(1));
        let future = Some(now + TimeDelta::hours(1));

        assert!(
            ReferredRewards {
                status: RewardStatus::Verified,
                verify_after: None
            }
            .can_verify_referral(now)
        );
        assert!(
            ReferredRewards {
                status: RewardStatus::Trusted,
                verify_after: None
            }
            .can_verify_referral(now)
        );
        assert!(
            ReferredRewards {
                status: RewardStatus::Verified,
                verify_after: future
            }
            .can_verify_referral(now)
        );
        assert!(
            !ReferredRewards {
                status: RewardStatus::Unverified,
                verify_after: None
            }
            .can_verify_referral(now)
        );
        assert!(
            !ReferredRewards {
                status: RewardStatus::Pending,
                verify_after: None
            }
            .can_verify_referral(now)
        );
        assert!(
            !ReferredRewards {
                status: RewardStatus::Attribution,
                verify_after: None
            }
            .can_verify_referral(now)
        );
        assert!(
            ReferredRewards {
                status: RewardStatus::Unverified,
                verify_after: past
            }
            .can_verify_referral(now)
        );
        assert!(
            ReferredRewards {
                status: RewardStatus::Pending,
                verify_after: past
            }
            .can_verify_referral(now)
        );
        assert!(
            !ReferredRewards {
                status: RewardStatus::Unverified,
                verify_after: future
            }
            .can_verify_referral(now)
        );
        assert!(
            !ReferredRewards {
                status: RewardStatus::Pending,
                verify_after: future
            }
            .can_verify_referral(now)
        );
    }

    #[test]
    fn test_clears_verification_delay() {
        let now = now();
        let past = Some(now - TimeDelta::hours(1));

        assert!(
            ReferredRewards {
                status: RewardStatus::Pending,
                verify_after: past
            }
            .clears_verification_delay(now)
        );
        assert!(
            !ReferredRewards {
                status: RewardStatus::Verified,
                verify_after: past
            }
            .clears_verification_delay(now)
        );
        assert!(
            !ReferredRewards {
                status: RewardStatus::Pending,
                verify_after: None
            }
            .clears_verification_delay(now)
        );
    }

    #[test]
    fn test_referral_verification_delay() {
        assert_eq!(referral_verification_delay(DAY, 2, RewardStatus::Trusted), None);
        assert_eq!(referral_verification_delay(DAY, 2, RewardStatus::Verified), Some(HOUR * 12));
        assert_eq!(referral_verification_delay(DAY, 2, RewardStatus::Attribution), Some(HOUR * 12));
        assert_eq!(referral_verification_delay(DAY, 2, RewardStatus::Unverified), Some(DAY));
        assert_eq!(referral_verification_delay(DAY, 0, RewardStatus::Verified), None);
    }

    #[test]
    fn test_new_referral_verification() {
        let now = now();

        assert_eq!(new_referral_verification(true, Some(HOUR), now), NewReferralVerification::Verified);
        assert_eq!(new_referral_verification(false, None, now), NewReferralVerification::Verified);
        assert_eq!(new_referral_verification(false, Some(HOUR), now), NewReferralVerification::Delayed { verify_after: now + TimeDelta::hours(1) });
    }
}
