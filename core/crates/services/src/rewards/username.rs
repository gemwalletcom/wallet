use config_keys::ConfigKey;
use primitives::Rewards;
use rewards::{RewardIdentity, UsernameRules, UsernameValidationError, validate_username, validate_username_available, validate_wallet_without_username};
use storage::{DatabaseClient, DatabaseError, RewardIdentityRecord, RewardsRepository};

use super::summary::rewards_by_wallet_id;
use crate::ConfigCacher;

pub async fn username_rules(config: &ConfigCacher) -> Result<UsernameRules, DatabaseError> {
    Ok(UsernameRules {
        min_length: config.get_usize(ConfigKey::UsernameMinLength).await?,
        max_length: config.get_usize(ConfigKey::UsernameMaxLength).await?,
    })
}

pub(super) fn reward_identity(record: RewardIdentityRecord) -> RewardIdentity {
    RewardIdentity {
        username: record.username,
        wallet_address: record.wallet_address,
    }
}

pub fn create_username(client: &mut DatabaseClient, wallet_id: i32, username: &str, rules: &UsernameRules) -> Result<Result<(Rewards, i32), UsernameValidationError>, DatabaseError> {
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
