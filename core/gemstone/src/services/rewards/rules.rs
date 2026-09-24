use crate::duration_formatter::countdown_parts;
use crate::precision::GemValueStyle;
use chrono::{DateTime, Utc};
use number_formatter::BigNumberFormatter;
use primitives::{CoreEmoji, RewardRedemptionOption, RewardStatus, Rewards, Wallet};

use super::model::{GemIncomingCode, GemRewardsAction, GemRewardsRedemption, GemRewardsState};
use crate::config::rewards::get_referral_url;
use crate::formatted_number::{GemFormattedNumber, GemNumberUnit};
use crate::models::list::{GemListRow, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle, GemNoticeKind};
use crate::services::localization::GemLocalizedText;

pub fn incoming_code(code: Option<&str>, wallets: &[Wallet]) -> Option<GemIncomingCode> {
    let code = code.map(str::trim).filter(|code| !code.is_empty())?.to_string();
    match wallets {
        [] => None,
        [_] => Some(GemIncomingCode::Activate { code }),
        _ => Some(GemIncomingCode::Confirm { code }),
    }
}

pub fn state(rewards: Option<&Rewards>, now: DateTime<Utc>) -> GemRewardsState {
    let Some(rewards) = rewards else {
        return GemRewardsState {
            invite_reward_points: points_number(Rewards::default().invite_reward_points),
            ..GemRewardsState::default()
        };
    };
    let has_referral_code = has_value(rewards.code.as_deref());
    let has_used_referral_code = has_value(rewards.used_referral_code.as_deref());
    let has_pending_referral = has_used_referral_code && rewards.verify_after.is_some();
    let can_activate_pending_referral = has_pending_referral && rewards.verify_after.is_some_and(|verify_after| now >= verify_after);
    let is_unverified = has_referral_code && rewards.status == RewardStatus::Unverified && !has_pending_referral;
    let referral_code = rewards.code.clone().filter(|code| !code.is_empty());
    let can_invite = has_referral_code && matches!(rewards.status, RewardStatus::Verified | RewardStatus::Trusted | RewardStatus::Attribution);
    let used_referral_code = rewards.used_referral_code.clone().filter(|code| !code.is_empty());
    let pending_code = (has_pending_referral && !is_unverified).then(|| used_referral_code.clone()).flatten();
    GemRewardsState {
        actions: actions(has_referral_code, can_invite, !has_referral_code && !has_used_referral_code, pending_code, can_activate_pending_referral),
        error_notice: rewards.disable_reason.clone().map(|reason| GemListRow::Notice {
            title: GemListRowTitle::Error,
            message: Some(GemLocalizedText::Text { text: reason }),
            kind: GemNoticeKind::Error,
        }),
        status_notice: status_notice(is_unverified, has_pending_referral, can_activate_pending_referral, rewards, now),
        sections: sections(
            has_referral_code || has_used_referral_code,
            info_rows(referral_code.as_deref(), rewards.referral_count, rewards.points, rewards.used_referral_code.as_deref()),
        ),
        invite_reward_points: points_number(rewards.invite_reward_points),
        referral_code: referral_code.clone(),
        referral_link: referral_code.as_deref().map(get_referral_url),
        used_referral_code,
        redemptions: redemptions(rewards),
    }
}

fn actions(has_referral_code: bool, can_invite: bool, can_use_referral_code: bool, pending_code: Option<String>, can_activate_pending: bool) -> Vec<GemRewardsAction> {
    [
        (!has_referral_code).then_some(GemRewardsAction::CreateCode),
        can_invite.then_some(GemRewardsAction::Share),
        can_use_referral_code.then_some(GemRewardsAction::UseReferralCode),
        pending_code.map(|code| GemRewardsAction::ActivatePendingReferral { code, is_enabled: can_activate_pending }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn sections(shows_info: bool, info_rows: Vec<GemListRow>) -> Vec<GemListSection> {
    match shows_info && !info_rows.is_empty() {
        true => vec![GemListSection {
            title: GemListSectionTitle::Info,
            footer: GemListSectionFooter::None,
            rows: info_rows,
        }],
        false => Vec::new(),
    }
}

fn status_notice(is_unverified: bool, has_pending_referral: bool, can_activate: bool, rewards: &Rewards, now: DateTime<Utc>) -> Option<GemListRow> {
    let notice = |title: GemListRowTitle, message: GemLocalizedText| GemListRow::Notice {
        title,
        message: Some(message),
        kind: GemNoticeKind::Info,
    };
    if is_unverified {
        return Some(notice(GemListRowTitle::RewardsUnverified, GemLocalizedText::RewardsUnverified));
    }
    if !has_pending_referral {
        return None;
    }
    let message = match can_activate {
        true => GemLocalizedText::RewardsPendingReady,
        false => GemLocalizedText::RewardsPending {
            countdown: rewards
                .verify_after
                .filter(|verify_after| *verify_after > now)
                .map(|verify_after| countdown_parts((verify_after - now).num_seconds()))
                .unwrap_or_default(),
        },
    };
    Some(notice(GemListRowTitle::RewardsPending, message))
}

fn points_number(points: i32) -> GemFormattedNumber {
    GemFormattedNumber {
        unit: GemNumberUnit::Symbol { symbol: CoreEmoji::Gem.glyph().to_string() },
        ..GemFormattedNumber::count(points.max(0) as u64)
    }
}

fn redemptions(rewards: &Rewards) -> Vec<GemRewardsRedemption> {
    rewards
        .redemption_options
        .iter()
        .filter_map(|option| {
            let asset = option.asset.as_ref()?;
            let value = BigNumberFormatter::f64_value(&option.value, asset.decimals as u32);
            let value = GemFormattedNumber::amount(value, Some(asset.symbol.clone()), GemValueStyle::Short);
            Some(GemRewardsRedemption {
                id: option.id.clone(),
                asset_id: asset.id.clone(),
                title: GemLocalizedText::RewardsRedeemAsset { value: value.clone() },
                points: points_number(option.points),
                value,
                can_redeem: can_redeem(rewards, option),
            })
        })
        .collect()
}

fn info_rows(referral_code: Option<&str>, referral_count: i32, points: i32, used_referral_code: Option<&str>) -> Vec<GemListRow> {
    let text = |title: GemListRowTitle, value: &str| GemListRow::Text { title, value: value.to_string() };
    let optional = |title: GemListRowTitle, value: Option<&str>| value.filter(|value| !value.is_empty()).map(|value| text(title, value));
    let amount = |title: GemListRowTitle, amount: GemFormattedNumber| GemListRow::Amount { title, amount, info: None };
    [
        optional(GemListRowTitle::MyReferralCode, referral_code),
        Some(amount(GemListRowTitle::Referrals, GemFormattedNumber::count(referral_count.max(0) as u64))),
        Some(amount(GemListRowTitle::Points, points_number(points))),
        optional(GemListRowTitle::InvitedBy, used_referral_code),
    ]
    .into_iter()
    .flatten()
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

    fn info_section_rows(state: &GemRewardsState) -> Vec<GemListRow> {
        state.sections.iter().flat_map(|section| section.rows.clone()).collect()
    }

    fn pending_activation(state: &GemRewardsState) -> Option<bool> {
        state.actions.iter().find_map(|action| match action {
            GemRewardsAction::ActivatePendingReferral { is_enabled, .. } => Some(*is_enabled),
            _ => None,
        })
    }
    #[test]
    fn test_the_info_rows_hold_every_value_the_screen_shows_and_skip_the_absent_ones() {
        let invited = Rewards {
            code: Some("GEM123".to_string()),
            used_referral_code: Some("FRIEND".to_string()),
            referral_count: 4,
            ..Rewards::mock(None, RewardStatus::Verified)
        };

        let titles = |rewards: &Rewards| {
            info_rows(rewards.code.as_deref(), rewards.referral_count, rewards.points, rewards.used_referral_code.as_deref())
                .into_iter()
                .map(|row| match row {
                    GemListRow::Text { title, value } => (title, value),
                    GemListRow::Amount { title, amount, .. } => (title, amount.value.to_string()),
                    _ => panic!("an info row is text or an amount"),
                })
                .collect::<Vec<_>>()
        };

        assert_eq!(
            titles(&invited),
            vec![
                (GemListRowTitle::MyReferralCode, "GEM123".to_string()),
                (GemListRowTitle::Referrals, "4".to_string()),
                (GemListRowTitle::Points, invited.points.to_string()),
                (GemListRowTitle::InvitedBy, "FRIEND".to_string()),
            ],
        );

        let alone = Rewards {
            code: None,
            used_referral_code: None,
            ..Rewards::mock(None, RewardStatus::Verified)
        };
        assert_eq!(
            titles(&alone).into_iter().map(|(title, _)| title).collect::<Vec<_>>(),
            vec![GemListRowTitle::Referrals, GemListRowTitle::Points],
            "a wallet with no code of its own and no inviter shows neither row",
        );
        assert!(state(Some(&alone), now()).sections.is_empty(), "and it has no info section at all, so no screen decides that for itself");
        assert_eq!(info_section_rows(&state(Some(&invited), now())).len(), 4, "the section the screen shows is the one the rows built");
    }

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
        let points = points_number(250);
        assert_eq!(points.value, 250.0);
        assert_eq!(points.unit, GemNumberUnit::Symbol { symbol: "\u{1f48e}".to_string() }, "the glyph is the number's unit, so each app groups the digits itself");
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
            redemptions.iter().map(|redemption| redemption.id.as_str()).collect::<Vec<_>>(),
            vec!["affordable", "too-expensive", "sold-out", "last-one"],
            "an option that pays out nothing the wallet can hold is not a row"
        );
        assert_eq!(redemptions.iter().map(|redemption| redemption.can_redeem).collect::<Vec<_>>(), vec![true, false, false, true]);
        assert!(
            redemptions.iter().all(|redemption| redemption.title == GemLocalizedText::RewardsRedeemAsset { value: redemption.value.clone() }),
            "the row title names the value it pays out"
        );
    }

    #[test]
    fn test_state_without_rewards_offers_nothing_but_still_names_the_invite_reward() {
        let state = state(None, now());

        assert_eq!(
            state,
            GemRewardsState {
                invite_reward_points: points_number(100),
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
        assert_eq!(
            info_section_rows(&state)
                .iter()
                .filter_map(|row| match row {
                    GemListRow::Amount { title, amount, .. } => Some((*title, amount.value)),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![(GemListRowTitle::Referrals, 5.0), (GemListRowTitle::Points, 250.0)]
        );
        assert_eq!(state.invite_reward_points.value, 150.0);
        assert_eq!(
            state.error_notice,
            Some(GemListRow::Notice {
                title: GemListRowTitle::Error,
                message: Some(GemLocalizedText::Text { text: "verification required".to_string() }),
                kind: GemNoticeKind::Error,
            })
        );
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

        assert_eq!(state.actions, vec![GemRewardsAction::CreateCode, GemRewardsAction::UseReferralCode]);
        assert!(state.sections.is_empty(), "a wallet with nothing to show has no info section");
        assert_eq!(state.status_notice, None);
    }

    #[test]
    fn test_state_invites_only_from_a_verified_trusted_or_attribution_code() {
        for status in [RewardStatus::Verified, RewardStatus::Trusted, RewardStatus::Attribution] {
            let state = state(Some(&Rewards::mock(Some("gem"), status)), now());
            assert_eq!(state.actions, vec![GemRewardsAction::Share], "{status:?}");
            assert_eq!(state.sections.iter().map(|section| section.title).collect::<Vec<_>>(), vec![GemListSectionTitle::Info]);
            assert_eq!(state.status_notice, None);
        }
        for status in [RewardStatus::Unverified, RewardStatus::Pending, RewardStatus::Disabled] {
            assert!(!state(Some(&Rewards::mock(Some("gem"), status)), now()).actions.contains(&GemRewardsAction::Share), "{status:?}");
        }
    }

    #[test]
    fn test_state_flags_an_unverified_code_until_a_referral_is_pending() {
        let unverified = GemListRow::Notice {
            title: GemListRowTitle::RewardsUnverified,
            message: Some(GemLocalizedText::RewardsUnverified),
            kind: GemNoticeKind::Info,
        };
        assert_eq!(state(Some(&Rewards::mock(Some("gem"), RewardStatus::Unverified)), now()).status_notice, Some(unverified));
        assert_eq!(state(Some(&Rewards::mock(None, RewardStatus::Unverified)), now()).status_notice, None);

        let pending = Rewards {
            code: Some("gem".to_string()),
            status: RewardStatus::Unverified,
            ..Rewards::mock_pending(now() + TimeDelta::hours(1))
        };
        let state = state(Some(&pending), now());
        assert_eq!(
            state.status_notice,
            Some(GemListRow::Notice {
                title: GemListRowTitle::RewardsPending,
                message: Some(GemLocalizedText::RewardsPending { countdown: countdown_parts(3600) }),
                kind: GemNoticeKind::Info,
            }),
            "a pending referral replaces the unverified notice"
        );
        assert!(pending_activation(&state).is_some());
    }

    #[test]
    fn test_state_activates_a_pending_referral_once_verify_after_is_reached() {
        let waiting = state(Some(&Rewards::mock_pending(now() + TimeDelta::hours(1))), now());
        assert_eq!(pending_activation(&waiting), Some(false), "the button waits in place while the countdown runs");
        assert_eq!(waiting.sections.iter().map(|section| section.title).collect::<Vec<_>>(), vec![GemListSectionTitle::Info]);
        assert!(!waiting.actions.contains(&GemRewardsAction::UseReferralCode));

        let ready = state(Some(&Rewards::mock_pending(now())), now());
        assert_eq!(pending_activation(&ready), Some(true));
        assert!(matches!(
            ready.status_notice,
            Some(GemListRow::Notice {
                message: Some(GemLocalizedText::RewardsPendingReady),
                ..
            })
        ));
        assert_eq!(pending_activation(&state(Some(&Rewards::mock_pending(now() - TimeDelta::seconds(1))), now())), Some(true));

        let without_used_code = Rewards {
            code: Some("gem".to_string()),
            used_referral_code: None,
            ..Rewards::mock_pending(now() - TimeDelta::hours(1))
        };
        assert_eq!(pending_activation(&state(Some(&without_used_code), now())), None);
    }

    #[test]
    fn test_state_lifts_the_rewards_the_apps_encode() {
        let json = r#"{"code":null,"referralCount":0,"points":0,"usedReferralCode":"friend","status":"pending","verifyAfter":"2026-01-06T10:13:34Z","redemptionOptions":[],"disableReason":null}"#;
        let rewards: Rewards = serde_json::from_str(json).unwrap();

        assert_eq!(pending_activation(&state(Some(&rewards), now())), Some(true));
    }
}
