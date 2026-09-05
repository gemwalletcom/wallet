use chrono::{DateTime, Utc};
use primitives::{RewardStatus, Rewards};

use super::model::GemRewardsState;

pub fn state(rewards: Option<&Rewards>, now: DateTime<Utc>) -> GemRewardsState {
    let Some(rewards) = rewards else {
        return GemRewardsState::default();
    };
    let has_referral_code = has_value(rewards.code.as_deref());
    let has_used_referral_code = has_value(rewards.used_referral_code.as_deref());
    let has_pending_referral = has_used_referral_code && rewards.verify_after.is_some();
    GemRewardsState {
        has_referral_code,
        has_used_referral_code,
        can_invite: has_referral_code && matches!(rewards.status, RewardStatus::Verified | RewardStatus::Trusted | RewardStatus::Attribution),
        can_use_referral_code: !has_referral_code && !has_used_referral_code,
        shows_info: has_referral_code || has_used_referral_code,
        is_unverified: has_referral_code && rewards.status == RewardStatus::Unverified && !has_pending_referral,
        has_pending_referral,
        can_activate_pending_referral: has_pending_referral && rewards.verify_after.is_some_and(|verify_after| now >= verify_after),
    }
}

fn has_value(code: Option<&str>) -> bool {
    code.is_some_and(|code| !code.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_767_694_414, 0).unwrap()
    }

    fn rewards(code: Option<&str>, status: RewardStatus) -> Rewards {
        Rewards {
            code: code.map(str::to_string),
            status,
            ..Rewards::default()
        }
    }

    fn pending(code: Option<&str>, verify_after: DateTime<Utc>) -> Rewards {
        Rewards {
            used_referral_code: Some("friend".to_string()),
            verify_after: Some(verify_after),
            ..rewards(code, RewardStatus::Pending)
        }
    }

    #[test]
    fn test_state_without_rewards_offers_nothing() {
        assert_eq!(state(None, now()), GemRewardsState::default());
    }

    #[test]
    fn test_state_without_a_code_lets_the_wallet_start_or_use_a_code() {
        let state = state(Some(&rewards(Some(""), RewardStatus::Unverified)), now());

        assert!(!state.has_referral_code);
        assert!(state.can_use_referral_code);
        assert!(!state.shows_info);
        assert!(!state.is_unverified);
        assert!(!state.can_invite);
    }

    #[test]
    fn test_state_invites_only_from_a_verified_trusted_or_attribution_code() {
        for status in [RewardStatus::Verified, RewardStatus::Trusted, RewardStatus::Attribution] {
            let state = state(Some(&rewards(Some("gem"), status)), now());
            assert!(state.can_invite, "{status:?}");
            assert!(state.shows_info);
            assert!(!state.can_use_referral_code);
            assert!(!state.is_unverified);
        }
        for status in [RewardStatus::Unverified, RewardStatus::Pending, RewardStatus::Disabled] {
            assert!(!state(Some(&rewards(Some("gem"), status)), now()).can_invite, "{status:?}");
        }
    }

    #[test]
    fn test_state_flags_an_unverified_code_until_a_referral_is_pending() {
        assert!(state(Some(&rewards(Some("gem"), RewardStatus::Unverified)), now()).is_unverified);
        assert!(!state(Some(&rewards(None, RewardStatus::Unverified)), now()).is_unverified);

        let pending = Rewards {
            used_referral_code: Some("friend".to_string()),
            verify_after: Some(now() + TimeDelta::hours(1)),
            ..rewards(Some("gem"), RewardStatus::Unverified)
        };
        let state = state(Some(&pending), now());
        assert!(!state.is_unverified);
        assert!(state.has_pending_referral);
    }

    #[test]
    fn test_state_activates_a_pending_referral_once_verify_after_is_reached() {
        let waiting = state(Some(&pending(None, now() + TimeDelta::hours(1))), now());
        assert!(waiting.has_pending_referral);
        assert!(!waiting.can_activate_pending_referral);
        assert!(waiting.shows_info);
        assert!(!waiting.can_use_referral_code);

        assert!(state(Some(&pending(None, now())), now()).can_activate_pending_referral);
        assert!(state(Some(&pending(None, now() - TimeDelta::seconds(1))), now()).can_activate_pending_referral);

        let without_used_code = Rewards {
            used_referral_code: None,
            ..pending(Some("gem"), now() - TimeDelta::hours(1))
        };
        let state = state(Some(&without_used_code), now());
        assert!(!state.has_pending_referral);
        assert!(!state.can_activate_pending_referral);
    }

    #[test]
    fn test_state_lifts_the_rewards_the_apps_encode() {
        let json = r#"{"code":null,"referralCount":0,"points":0,"usedReferralCode":"friend","status":"pending","verifyAfter":"2026-01-06T10:13:34Z","redemptionOptions":[],"disableReason":null}"#;
        let rewards: Rewards = serde_json::from_str(json).unwrap();

        let state = state(Some(&rewards), now());
        assert!(state.has_pending_referral);
        assert!(state.can_activate_pending_referral);
    }
}
