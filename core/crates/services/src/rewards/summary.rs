use primitives::Rewards;
use primitives::rewards::RewardRedemptionType;
use rewards::{UsernameRules, available_redemption_options, invite_reward_points, offers_redemptions};
use storage::{DatabaseClient, DatabaseError, RewardsRedemptionsRepository, RewardsRepository};

use super::username::reward_identity;

pub fn rewards_by_wallet_id(client: &mut DatabaseClient, wallet_id: i32, rules: &UsernameRules) -> Result<Rewards, DatabaseError> {
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
