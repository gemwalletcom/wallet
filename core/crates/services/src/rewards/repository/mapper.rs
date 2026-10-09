use primitives::rewards::RewardRedemptionOption;
use primitives::{NotificationType, RewardEventType, Rewards};
use rewards::{DeviceWallet, Referral, ReferralUseFacts, RewardIdentity, UsernameRules, invite_reward_points};
use storage::{ReferralRecord, ReferralUseFactsRecord, RewardIdentityRecord, RewardsRecord};

pub(super) fn referral(record: ReferralRecord) -> Referral {
    Referral {
        referrer_username: record.referrer_username,
        referred_username: record.referred_username,
        referred_device_id: record.referred_device_id,
        is_verified: record.verified_at.is_some(),
    }
}

pub(super) fn reward_identity(record: RewardIdentityRecord) -> RewardIdentity {
    RewardIdentity {
        username: record.username,
        wallet_address: record.wallet_address,
    }
}

pub(super) fn referral_use_facts(record: ReferralUseFactsRecord) -> ReferralUseFacts {
    ReferralUseFacts {
        referred_username: record.referred_username,
        referred_status: record.referred_status,
        wallet_first_subscription_at: record.wallet_first_subscription_at,
        device_wallets: record.device_wallets.into_iter().map(|(wallet_id, first_subscription_at)| DeviceWallet { wallet_id, first_subscription_at }).collect(),
        device_referral: record.device_referral.map(referral),
    }
}

pub(super) fn rewards(identity: RewardIdentity, record: RewardsRecord, redemption_options: Vec<RewardRedemptionOption>, rules: &UsernameRules) -> Rewards {
    Rewards {
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
    }
}

pub(super) fn notification_type(event: RewardEventType) -> NotificationType {
    match event {
        RewardEventType::CreateUsername => NotificationType::RewardsCreateUsername,
        RewardEventType::InvitePending | RewardEventType::InviteNew => NotificationType::RewardsInvite,
        RewardEventType::Joined => NotificationType::ReferralJoined,
        RewardEventType::Enabled => NotificationType::RewardsEnabled,
        RewardEventType::Disabled => NotificationType::RewardsCodeDisabled,
        RewardEventType::Redeemed => NotificationType::RewardsRedeemed,
    }
}
