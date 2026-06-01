use std::error::Error;

use num_bigint::BigUint;
use primitives::{
    AssetBalance, AssetId, Chain, Resource,
    asset_balance::{Balance, BalanceMetadata},
    decode_hex,
};

use crate::models::{TronAccount, TronAccountUsage, TronReward};

pub fn map_coin_balance(account: &TronAccount) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let available_balance = BigUint::from(account.balance.unwrap_or(0));
    Ok(AssetBalance::new(AssetId::from_chain(Chain::Tron), available_balance))
}

pub fn map_token_balance(balance_hex: &str, asset_id: AssetId) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let balance_bytes = decode_hex(balance_hex).map_err(|e| format!("Failed to parse hex balance: {e}"))?;
    let balance = BigUint::from_bytes_be(&balance_bytes);

    Ok(AssetBalance::new(asset_id, balance))
}

pub fn map_metadata_from_usage(usage: &TronAccountUsage, votes: u32) -> BalanceMetadata {
    let energy_total = usage.energy_limit;
    let energy_available = energy_total.saturating_sub(usage.energy_used);
    let bandwidth_total = usage.free_net_limit + usage.net_limit;
    let bandwidth_available = usage.free_net_limit.saturating_sub(usage.free_net_used) + usage.net_limit.saturating_sub(usage.net_used);

    BalanceMetadata {
        votes,
        energy_available: energy_available as u32,
        energy_total: energy_total as u32,
        bandwidth_available: bandwidth_available as u32,
        bandwidth_total: bandwidth_total as u32,
    }
}

pub fn map_staking_balance(account: &TronAccount, reward: &TronReward, usage: &TronAccountUsage) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let (bandwidth_frozen, energy_frozen) = account
        .frozen_v2
        .as_deref()
        .unwrap_or_default()
        .iter()
        .fold((0u64, 0u64), |(bandwidth, energy), frozen| match frozen.resource() {
            Some(Resource::Bandwidth) => (bandwidth + frozen.amount, energy),
            Some(Resource::Energy) => (bandwidth, energy + frozen.amount),
            None => (bandwidth, energy),
        });
    let votes: u64 = account.votes.as_ref().map_or(0, |votes| votes.iter().map(|vote| vote.vote_count).sum());
    let pending_amount: u64 = account
        .unfrozen_v2
        .as_ref()
        .map_or(0, |unfrozen_list| unfrozen_list.iter().map(|unfrozen| unfrozen.unfreeze_amount).sum());
    let metadata = map_metadata_from_usage(usage, votes as u32);

    Ok(AssetBalance::new_balance(
        AssetId::from_chain(Chain::Tron),
        new_stake_balance(
            BigUint::from(bandwidth_frozen),
            BigUint::from(energy_frozen),
            BigUint::from(0u32),
            BigUint::from(pending_amount),
            BigUint::from(reward.reward),
            metadata,
        ),
    ))
}

pub fn map_balance_staking(account: &TronAccount, reward: &TronReward, usage: &TronAccountUsage) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    if account.is_staking() {
        map_staking_balance(account, reward, usage)
    } else {
        let metadata = map_metadata_from_usage(usage, 0);
        Ok(AssetBalance::new_balance(
            AssetId::from_chain(Chain::Tron),
            new_stake_balance(
                BigUint::from(0u32),
                BigUint::from(0u32),
                BigUint::from(0u32),
                BigUint::from(0u32),
                BigUint::from(0u32),
                metadata,
            ),
        ))
    }
}

fn new_stake_balance(
    frozen: BigUint,  // bandwidth frozen
    locked: BigUint,  // energy frozen
    staked: BigUint,  // vote amount
    pending: BigUint, // unfreezing amount
    rewards: BigUint, // voting rewards
    metadata: BalanceMetadata,
) -> Balance {
    Balance {
        available: BigUint::from(0u32),
        frozen,
        locked,
        staked,
        pending,
        pending_unconfirmed: BigUint::from(0u32),
        rewards,
        reserved: BigUint::from(0u32),
        earn: BigUint::from(0u32),
        withdrawable: BigUint::from(0u32),
        metadata: Some(metadata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TronAccount, TronFrozen, TronReward, TronSmartContractResult, TronUnfrozen, TronVote};
    use primitives::{AssetId, Chain, asset_constants::TRON_USDT_ASSET_ID};
    use serde_json;

    #[test]
    fn test_map_coin_balance_with_real_payload() {
        let account: TronAccount = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();
        let balance = map_coin_balance(&account).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.available, BigUint::from(2928601454_u64));
    }

    #[test]
    fn test_map_token_balance_with_real_payload() {
        let response: TronSmartContractResult = serde_json::from_str(include_str!("../../testdata/balance_token.json")).unwrap();
        let asset_id: AssetId = TRON_USDT_ASSET_ID.clone();
        let balance = map_token_balance(&response.constant_result[0], asset_id.clone()).unwrap();

        assert_eq!(balance.asset_id, asset_id);
        assert_eq!(balance.balance.available, BigUint::from(136389002_u64));
    }

    #[test]
    fn test_map_token_balance_edge_cases() {
        let asset_id: AssetId = TRON_USDT_ASSET_ID.clone();

        let balance = map_token_balance("", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(0u32));

        let balance = map_token_balance("0x", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(0u32));

        let balance = map_token_balance("0x0", asset_id.clone()).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(0u32));

        let balance = map_token_balance("0x821218a", asset_id).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(136389002_u64));
    }

    #[test]
    fn test_map_coin_balance_zero_balance() {
        let account = TronAccount {
            balance: None,
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            owner_permission: None,
            active_permission: None,
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        };

        let balance = map_coin_balance(&account).unwrap();
        assert_eq!(balance.balance.available, BigUint::from(0u32));
    }

    #[test]
    fn test_map_staking_balance() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            owner_permission: None,
            active_permission: None,
            votes: None,
            frozen_v2: Some(vec![
                TronFrozen {
                    frozen_type: Some("BANDWIDTH".to_string()),
                    amount: 5000000,
                },
                TronFrozen {
                    frozen_type: Some("ENERGY".to_string()),
                    amount: 3000000,
                },
                TronFrozen {
                    frozen_type: Some("TRON_POWER".to_string()),
                    amount: 4000000,
                },
                TronFrozen {
                    frozen_type: Some("UNKNOWN".to_string()),
                    amount: 6000000,
                },
            ]),
            unfrozen_v2: Some(vec![TronUnfrozen {
                unfreeze_amount: 2000000,
                unfreeze_expire_time: Some(1234567890),
            }]),
        };

        let reward = TronReward { reward: 100000 };
        let usage = TronAccountUsage {
            energy_limit: 1000000,
            energy_used: 500000,
            free_net_limit: 1000000,
            free_net_used: 500000,
            net_used: 200000,
            net_limit: 1000000,
        };

        let balance = map_staking_balance(&account, &reward, &usage).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.frozen, BigUint::from(5000000_u64));
        assert_eq!(balance.balance.locked, BigUint::from(3000000_u64));
        assert_eq!(balance.balance.staked, BigUint::from(0_u64));
        assert_eq!(balance.balance.pending, BigUint::from(2000000_u64));
        assert_eq!(balance.balance.rewards, BigUint::from(100000_u64));
    }

    #[test]
    fn test_map_staking_balance_empty_fields() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            owner_permission: None,
            active_permission: None,
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        };

        let reward = TronReward { reward: 0 };
        let usage = TronAccountUsage {
            energy_limit: 1000000,
            energy_used: 500000,
            free_net_limit: 1000000,
            free_net_used: 500000,
            net_used: 200000,
            net_limit: 1000000,
        };
        let balance = map_staking_balance(&account, &reward, &usage).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.frozen, BigUint::from(0_u64));
        assert_eq!(balance.balance.locked, BigUint::from(0_u64));
        assert_eq!(balance.balance.staked, BigUint::from(0_u64));
        assert_eq!(balance.balance.pending, BigUint::from(0_u64));
        assert_eq!(balance.balance.rewards, BigUint::from(0_u64));
    }

    #[test]
    fn test_map_staking_balance_with_votes() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            owner_permission: None,
            active_permission: None,
            votes: Some(vec![
                TronVote {
                    vote_address: "TJApZYJwPKuQR7tL6FmvD6jDjbYpHESZGH".to_string(),
                    vote_count: 3000000,
                },
                TronVote {
                    vote_address: "TEqyWRKCzREYC2bK2fc3j7pp8XjAa6tJK1".to_string(),
                    vote_count: 2000000,
                },
            ]),
            frozen_v2: Some(vec![TronFrozen {
                frozen_type: Some("BANDWIDTH".to_string()),
                amount: 8000000,
            }]),
            unfrozen_v2: None,
        };

        let reward = TronReward { reward: 50000 };
        let usage = TronAccountUsage {
            energy_limit: 1000000,
            energy_used: 500000,
            free_net_limit: 1000000,
            free_net_used: 500000,
            net_used: 200000,
            net_limit: 1000000,
        };

        let balance = map_staking_balance(&account, &reward, &usage).unwrap();

        assert_eq!(balance.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(balance.balance.metadata.unwrap().votes, 5000000);
        assert_eq!(balance.balance.frozen, BigUint::from(8000000_u64));
        assert_eq!(balance.balance.locked, BigUint::from(0_u64));
        assert_eq!(balance.balance.staked, BigUint::from(0_u64));
        assert_eq!(balance.balance.pending, BigUint::from(0_u64));
        assert_eq!(balance.balance.rewards, BigUint::from(50000_u64));
    }

    #[test]
    fn test_map_staking_balance_metadata() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            owner_permission: None,
            active_permission: None,
            votes: None,
            frozen_v2: Some(vec![TronFrozen {
                frozen_type: Some("ENERGY".to_string()),
                amount: 1000000,
            }]),
            unfrozen_v2: None,
        };

        let reward = TronReward { reward: 50000 };
        let usage = TronAccountUsage {
            energy_limit: 2000000,
            energy_used: 800000,
            free_net_limit: 1500,
            free_net_used: 500,
            net_used: 0,
            net_limit: 5000,
        };

        let balance = map_staking_balance(&account, &reward, &usage).unwrap();
        let metadata = balance.balance.metadata.as_ref().unwrap();

        assert_eq!(metadata.energy_available, 1200000);
        assert_eq!(metadata.energy_total, 2000000);
        assert_eq!(metadata.bandwidth_available, 6000);
        assert_eq!(metadata.bandwidth_total, 6500);
    }

    #[test]
    fn test_new_stake_balance() {
        let metadata = BalanceMetadata {
            votes: 0,
            energy_available: 1000,
            energy_total: 2000,
            bandwidth_available: 500,
            bandwidth_total: 1000,
        };

        let balance = new_stake_balance(
            BigUint::from(100_u64),
            BigUint::from(200_u64),
            BigUint::from(300_u64),
            BigUint::from(400_u64),
            BigUint::from(500_u64),
            metadata.clone(),
        );

        assert_eq!(balance.available, BigUint::from(0_u32));
        assert_eq!(balance.frozen, BigUint::from(100_u64));
        assert_eq!(balance.locked, BigUint::from(200_u64));
        assert_eq!(balance.staked, BigUint::from(300_u64));
        assert_eq!(balance.pending, BigUint::from(400_u64));
        assert_eq!(balance.rewards, BigUint::from(500_u64));
        assert_eq!(balance.reserved, BigUint::from(0_u32));
        assert_eq!(balance.withdrawable, BigUint::from(0_u32));
        assert_eq!(balance.metadata, Some(metadata));
    }

    #[test]
    fn test_map_staking_balance_metadata_with_none_values() {
        let account = TronAccount {
            balance: None,
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            owner_permission: None,
            active_permission: None,
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        };

        let reward = TronReward { reward: 0 };
        let usage = TronAccountUsage {
            energy_limit: 0,
            energy_used: 0,
            free_net_limit: 0,
            free_net_used: 0,
            net_used: 0,
            net_limit: 0,
        };

        let balance = map_staking_balance(&account, &reward, &usage).unwrap();
        let metadata = balance.balance.metadata.as_ref().unwrap();

        assert_eq!(metadata.energy_available, 0);
        assert_eq!(metadata.energy_total, 0);
        assert_eq!(metadata.bandwidth_available, 0);
        assert_eq!(metadata.bandwidth_total, 0);
    }

    #[test]
    fn test_map_balance_staking_non_staker() {
        let account = TronAccount {
            balance: Some(1000),
            address: Some("TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string()),
            owner_permission: None,
            active_permission: None,
            votes: None,
            frozen_v2: None,
            unfrozen_v2: None,
        };
        let reward = TronReward { reward: 0 };
        let usage = TronAccountUsage {
            energy_limit: 0,
            energy_used: 0,
            free_net_limit: 600,
            free_net_used: 100,
            net_used: 0,
            net_limit: 0,
        };

        let balance = map_balance_staking(&account, &reward, &usage).unwrap();
        let metadata = balance.balance.metadata.unwrap();

        assert_eq!(metadata.bandwidth_available, 500);
        assert_eq!(metadata.bandwidth_total, 600);
        assert_eq!(metadata.votes, 0);
    }

    #[test]
    fn test_map_metadata_from_usage() {
        let usage = TronAccountUsage {
            energy_limit: 1000000,
            energy_used: 500000,
            free_net_limit: 1500,
            free_net_used: 500,
            net_used: 200,
            net_limit: 5000,
        };

        let metadata = map_metadata_from_usage(&usage, 100);

        assert_eq!(metadata.votes, 100);
        assert_eq!(metadata.energy_available, 500000);
        assert_eq!(metadata.energy_total, 1000000);
        assert_eq!(metadata.bandwidth_available, 5800);
        assert_eq!(metadata.bandwidth_total, 6500);
    }
}
