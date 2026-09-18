use crate::duration_formatter::countdown_parts;
use crate::formatted_number::GemFormattedNumber;
use crate::precision::GemValueStyle;
use chrono::{DateTime, Utc};
use number_formatter::BigNumberFormatter;
use primitives::{CoreEmoji, RewardRedemptionOption, RewardStatus, Rewards};

use super::model::{GemRewardsRedemption, GemRewardsState};
use crate::config::rewards::get_referral_url;

pub fn state(rewards: Option<&Rewards>, now: DateTime<Utc>) -> GemRewardsState {
    let Some(rewards) = rewards else {
        return GemRewardsState {
            invite_reward_points_text: Rewards::default().invite_reward_points.to_string(),
            ..GemRewardsState::default()
        };
    };
    let has_referral_code = has_value(rewards.code.as_deref());
    let has_used_referral_code = has_value(rewards.used_referral_code.as_deref());
    let has_pending_referral = has_used_referral_code && rewards.verify_after.is_some();
    let referral_code = rewards.code.clone().filter(|code| !code.is_empty());
    GemRewardsState {
        has_referral_code,
        has_used_referral_code,
        can_invite: has_referral_code && matches!(rewards.status, RewardStatus::Verified | RewardStatus::Trusted | RewardStatus::Attribution),
        can_use_referral_code: !has_referral_code && !has_used_referral_code,
        shows_info: has_referral_code || has_used_referral_code,
        is_unverified: has_referral_code && rewards.status == RewardStatus::Unverified && !has_pending_referral,
        has_pending_referral,
        can_activate_pending_referral: has_pending_referral && rewards.verify_after.is_some_and(|verify_after| now >= verify_after),
        invite_reward_points_text: rewards.invite_reward_points.to_string(),
        referral_code: referral_code.clone(),
        referral_link: referral_code.as_deref().map(get_referral_url),
        used_referral_code: rewards.used_referral_code.clone().filter(|code| !code.is_empty()),
        pending_countdown: rewards
            .verify_after
            .filter(|verify_after| *verify_after > now)
            .map(|verify_after| countdown_parts((verify_after - now).num_seconds()))
            .unwrap_or_default(),
        disable_reason: rewards.disable_reason.clone(),
        referral_count_text: rewards.referral_count.to_string(),
        points_text: points_text(rewards.points),
        redemptions: redemptions(rewards),
    }
}

fn points_text(points: i32) -> String {
    format!("{points} {}", CoreEmoji::Gem.glyph())
}

fn redemptions(rewards: &Rewards) -> Vec<GemRewardsRedemption> {
    rewards
        .redemption_options
        .iter()
        .filter_map(|option| {
            let asset = option.asset.as_ref()?;
            let value = BigNumberFormatter::value_as_f64(&option.value.to_string(), asset.decimals as u32).ok()?;
            Some(GemRewardsRedemption {
                points_text: points_text(option.points),
                value: GemFormattedNumber::amount(value, Some(asset.symbol.clone()), GemValueStyle::Short),
                option: option.clone(),
                can_redeem: can_redeem(rewards, option),
            })
        })
        .collect()
}

fn can_redeem(rewards: &Rewards, option: &RewardRedemptionOption) -> bool {
    rewards.points >= option.points && option.remaining.is_none_or(|remaining| remaining > 0)
}

fn has_value(code: Option<&str>) -> bool {
    code.is_some_and(|code| !code.is_empty())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_state_carries_the_referral_link() {
        let rewards = Rewards {
            code: Some("abc123".to_string()),
            ..Rewards::default()
        };
        let with_code = super::state(Some(&rewards), Utc::now());
        assert_eq!(with_code.referral_code.as_deref(), Some("abc123"));
        assert_eq!(with_code.referral_link, Some(get_referral_url("abc123")));

        let without_code = super::state(Some(&Rewards::default()), Utc::now());
        assert_eq!(without_code.referral_code, None);
        assert_eq!(without_code.referral_link, None);
    }

    #[test]
    fn test_points_read_with_the_gem_glyph() {
        assert_eq!(points_text(250), "250 \u{1f48e}");
    }

    use super::*;
    use chrono::TimeDelta;
    use num_bigint::BigUint;
    use primitives::Asset;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_767_694_414, 0).unwrap()
    }

    #[test]
    fn test_a_redemption_carries_its_payout_as_a_short_amount_in_the_assets_symbol() {
        let asset = Asset::mock_eth();
        let rewards = Rewards {
            points: 1000,
            redemption_options: vec![RewardRedemptionOption {
                value: BigUint::from(25_000_000_000_000_000u64),
                ..RewardRedemptionOption::mock(Some(asset.clone()))
            }],
            ..Rewards::default()
        };

        let redemption = redemptions(&rewards).pop().expect("an option with an asset is offered");
        assert_eq!(redemption.value.value, 0.025);
        assert_eq!(redemption.value.unit, crate::formatted_number::GemNumberUnit::Symbol { symbol: asset.symbol });
    }

    #[test]
    fn test_only_affordable_options_that_pay_out_an_asset_are_offered() {
        let asset = Some(Asset::mock_eth());
        let rewards = Rewards {
            points: 100,
            redemption_options: vec![
                RewardRedemptionOption {
                    id: "affordable".to_string(),
                    points: 100,
                    ..RewardRedemptionOption::mock(asset.clone())
                },
                RewardRedemptionOption {
                    id: "too-expensive".to_string(),
                    points: 101,
                    ..RewardRedemptionOption::mock(asset.clone())
                },
                RewardRedemptionOption {
                    id: "sold-out".to_string(),
                    points: 10,
                    remaining: Some(0),
                    ..RewardRedemptionOption::mock(asset.clone())
                },
                RewardRedemptionOption {
                    id: "last-one".to_string(),
                    points: 10,
                    remaining: Some(1),
                    ..RewardRedemptionOption::mock(asset)
                },
                RewardRedemptionOption {
                    id: "no-asset".to_string(),
                    points: 10,
                    ..RewardRedemptionOption::mock(None)
                },
            ],
            ..Rewards::default()
        };

        let redemptions = state(Some(&rewards), now()).redemptions;

        assert_eq!(
            redemptions.iter().map(|redemption| redemption.option.id.as_str()).collect::<Vec<_>>(),
            vec!["affordable", "too-expensive", "sold-out", "last-one"],
            "an option that pays out nothing the wallet can hold is not a row"
        );
        assert_eq!(
            redemptions.iter().map(|redemption| redemption.can_redeem).collect::<Vec<_>>(),
            vec![true, false, false, true]
        );
    }

    #[test]
    fn test_state_without_rewards_offers_nothing_but_still_names_the_invite_reward() {
        let state = state(None, now());

        assert_eq!(
            state,
            GemRewardsState {
                invite_reward_points_text: "100".to_string(),
                ..GemRewardsState::default()
            },
            "a wallet whose rewards failed to load still reads the invite pitch"
        );
    }

    #[test]
    fn test_state_carries_the_values_the_info_rows_read() {
        let rewards = Rewards {
            code: Some("gem".to_string()),
            used_referral_code: Some("friend".to_string()),
            referral_count: 5,
            points: 250,
            invite_reward_points: 150,
            disable_reason: Some("verification required".to_string()),
            ..Rewards::mock(Some("gem"), RewardStatus::Verified)
        };

        let state = state(Some(&rewards), now());

        assert_eq!(state.referral_code.as_deref(), Some("gem"));
        assert_eq!(state.used_referral_code.as_deref(), Some("friend"));
        assert_eq!(state.referral_count_text, "5");
        assert_eq!(state.points_text, "250 \u{1f48e}");
        assert_eq!(state.invite_reward_points_text, "150");
        assert_eq!(state.disable_reason.as_deref(), Some("verification required"));
    }

    #[test]
    fn test_state_reads_an_empty_code_as_no_code() {
        let state = state(Some(&Rewards::mock(Some(""), RewardStatus::Unverified)), now());

        assert_eq!(state.referral_code, None);
        assert_eq!(state.used_referral_code, None);
    }

    #[test]
    fn test_state_without_a_code_lets_the_wallet_start_or_use_a_code() {
        let state = state(Some(&Rewards::mock(Some(""), RewardStatus::Unverified)), now());

        assert!(!state.has_referral_code);
        assert!(state.can_use_referral_code);
        assert!(!state.shows_info);
        assert!(!state.is_unverified);
        assert!(!state.can_invite);
    }

    #[test]
    fn test_state_invites_only_from_a_verified_trusted_or_attribution_code() {
        for status in [RewardStatus::Verified, RewardStatus::Trusted, RewardStatus::Attribution] {
            let state = state(Some(&Rewards::mock(Some("gem"), status)), now());
            assert!(state.can_invite, "{status:?}");
            assert!(state.shows_info);
            assert!(!state.can_use_referral_code);
            assert!(!state.is_unverified);
        }
        for status in [RewardStatus::Unverified, RewardStatus::Pending, RewardStatus::Disabled] {
            assert!(!state(Some(&Rewards::mock(Some("gem"), status)), now()).can_invite, "{status:?}");
        }
    }

    #[test]
    fn test_state_flags_an_unverified_code_until_a_referral_is_pending() {
        assert!(state(Some(&Rewards::mock(Some("gem"), RewardStatus::Unverified)), now()).is_unverified);
        assert!(!state(Some(&Rewards::mock(None, RewardStatus::Unverified)), now()).is_unverified);

        let pending = Rewards {
            code: Some("gem".to_string()),
            status: RewardStatus::Unverified,
            ..Rewards::mock_pending(now() + TimeDelta::hours(1))
        };
        let state = state(Some(&pending), now());
        assert!(!state.is_unverified);
        assert!(state.has_pending_referral);
    }

    #[test]
    fn test_state_activates_a_pending_referral_once_verify_after_is_reached() {
        let waiting = state(Some(&Rewards::mock_pending(now() + TimeDelta::hours(1))), now());
        assert!(waiting.has_pending_referral);
        assert!(!waiting.can_activate_pending_referral);
        assert!(waiting.shows_info);
        assert!(!waiting.can_use_referral_code);

        assert!(state(Some(&Rewards::mock_pending(now())), now()).can_activate_pending_referral);
        assert!(state(Some(&Rewards::mock_pending(now() - TimeDelta::seconds(1))), now()).can_activate_pending_referral);

        let without_used_code = Rewards {
            code: Some("gem".to_string()),
            used_referral_code: None,
            ..Rewards::mock_pending(now() - TimeDelta::hours(1))
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
