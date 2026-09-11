use crate::models::{EpochInfo, TokenAccountInfo, VoteAccount};
use chrono::{DateTime, Duration, Utc};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};

pub fn calculate_network_apy(inflation_rate: f64, total_supply: u64, total_active_stake: u64) -> f64 {
    if total_active_stake == 0 {
        return 0.0;
    }

    inflation_rate * (total_supply as f64 / total_active_stake as f64) * 100.0
}

pub fn map_staking_validators(vote_accounts: Vec<VoteAccount>, chain: Chain, network_apy: f64) -> Vec<DelegationValidator> {
    vote_accounts
        .into_iter()
        .map(|validator| {
            let commission_rate = validator.commission as f64 / 100.0;
            let is_active = true;
            let validator_apr = if is_active { network_apy - (network_apy * commission_rate) } else { 0.0 };

            DelegationValidator::stake(chain, validator.vote_pubkey, String::new(), is_active, validator.commission as f64, validator_apr)
        })
        .collect()
}

pub fn map_staking_delegations(stake_accounts: Vec<TokenAccountInfo>, epoch: EpochInfo, asset_id: AssetId, now: DateTime<Utc>) -> Vec<DelegationBase> {
    stake_accounts
        .into_iter()
        .filter_map(|account| {
            if let Some(stake_info) = &account.account.data.parsed.info.stake {
                let balance = BigUint::from(account.account.lamports);
                let validator_id = stake_info.delegation.voter.clone();

                let activation_epoch = stake_info.delegation.activation_epoch;
                let deactivation_epoch = stake_info.delegation.deactivation_epoch;

                let is_active = deactivation_epoch == u64::MAX;

                let state = if !is_active {
                    if deactivation_epoch == epoch.epoch {
                        DelegationState::Deactivating
                    } else if deactivation_epoch < epoch.epoch {
                        DelegationState::AwaitingWithdrawal
                    } else {
                        DelegationState::Active
                    }
                } else if activation_epoch == epoch.epoch {
                    DelegationState::Activating
                } else if activation_epoch <= epoch.epoch {
                    DelegationState::Active
                } else {
                    DelegationState::Pending
                };

                let completion_date = match state {
                    DelegationState::Activating | DelegationState::Deactivating => {
                        let remaining_slots = epoch.slots_in_epoch.saturating_sub(epoch.slot_index);
                        let completion_seconds = remaining_slots as f64 * 0.420;
                        Some(now + Duration::seconds(completion_seconds as i64))
                    }
                    _ => None,
                };

                let rewards = BigUint::from(0u32);

                return Some(DelegationBase {
                    asset_id: asset_id.clone(),
                    state,
                    balance,
                    shares: BigUint::from(0u32),
                    rewards,
                    completion_date,
                    delegation_id: account.pubkey.clone(),
                    validator_id,
                });
            }
            None
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EpochInfo, Info, Parsed, StakeDelegation, StakeInfo, TokenAccountData, TokenAccountInfo, TokenAccountInfoData, VoteAccount};
    use primitives::{AssetId, Chain, DelegationState};

    #[test]
    fn test_map_staking_validators() {
        let vote_accounts = vec![VoteAccount {
            vote_pubkey: "validator1".to_string(),
            node_pubkey: "node1".to_string(),
            commission: 5,
            activated_stake: 1_000_000,
        }];

        let result = map_staking_validators(vote_accounts, Chain::Solana, 8.0);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "validator1");
        assert_eq!(result[0].commission, 5.0);
        assert_eq!(result[0].apr, 7.6);
    }

    #[test]
    fn test_calculate_network_apy() {
        let apy = calculate_network_apy(0.125, 400, 100);

        assert_eq!(apy, 50.0);
    }

    fn stake_account(activation_epoch: u64, deactivation_epoch: u64) -> TokenAccountInfo {
        TokenAccountInfo {
            pubkey: "stake1".to_string(),
            account: TokenAccountData {
                data: Parsed {
                    parsed: Info {
                        info: TokenAccountInfoData {
                            mint: None,
                            token_amount: None,
                            stake: Some(StakeInfo {
                                delegation: StakeDelegation {
                                    activation_epoch,
                                    deactivation_epoch,
                                    stake: "1000000".to_string(),
                                    voter: "validator1".to_string(),
                                },
                            }),
                        },
                    },
                },
                owner: "owner1".to_string(),
                lamports: 1000000,
            },
        }
    }

    #[test]
    fn test_map_staking_delegations() {
        let now = DateTime::from_timestamp(1_757_000_000, 0).unwrap();

        let result = map_staking_delegations(vec![stake_account(100, u64::MAX)], EpochInfo::mock(0), AssetId::from_chain(Chain::Solana), now);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].validator_id, "validator1");
        assert_eq!(result[0].balance.to_string(), "1000000");
        assert_eq!(result[0].state, DelegationState::Active);
        assert_eq!(result[0].completion_date, None);
    }

    #[test]
    fn test_map_staking_delegations_completion_date() {
        let now = DateTime::from_timestamp(1_757_000_000, 0).unwrap();
        let deactivating =
            |slot_index: u64| map_staking_delegations(vec![stake_account(100, 200)], EpochInfo::mock(slot_index), AssetId::from_chain(Chain::Solana), now)[0].clone();

        assert_eq!(deactivating(0).state, DelegationState::Deactivating);

        assert_eq!(deactivating(0).completion_date, Some(now + Duration::seconds(181_440)));
        assert_eq!(deactivating(216_000).completion_date, Some(now + Duration::seconds(90_720)));
        assert_eq!(deactivating(431_000).completion_date, Some(now + Duration::seconds(420)));
        assert_eq!(deactivating(432_000).completion_date, Some(now));
    }
}
