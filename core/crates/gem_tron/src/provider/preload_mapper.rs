use std::collections::HashMap;
use std::error::Error;

use num_bigint::BigInt;

use crate::models::ChainParameter;
use crate::models::TronAccountUsage;
use crate::models::account::{TronAccount, TronFrozen};
use crate::rpc::constants::{DEFAULT_BANDWIDTH_BYTES, GET_CREATE_ACCOUNT_FEE, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT, GET_ENERGY_FEE, GET_MEMO_FEE, GET_TRANSACTION_FEE};
use primitives::{StakeType, TransactionFee, TronStakeData, TronUnfreeze, TronVote};

const FEE_LIMIT_BUFFER_PERCENT: u64 = 20;

pub(crate) struct FeeEstimateContext<'a> {
    pub(crate) chain_parameters: &'a [ChainParameter],
    pub(crate) account_usage: &'a TronAccountUsage,
    pub(crate) is_new_account: bool,
}

pub(crate) fn native_transfer_fee(context: &FeeEstimateContext<'_>, has_memo: bool) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
    Ok(TransactionFee::new_from_fee(calculate_transfer_fee_rate(
        context.chain_parameters,
        context.account_usage,
        context.is_new_account,
        has_memo,
    )?))
}

pub fn calculate_transfer_fee_rate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    is_new_account: bool,
    has_memo: bool,
) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)?;
    let memo_fee = if has_memo { get_chain_parameter_value(chain_parameters, GET_MEMO_FEE)? } else { 0 };

    let base_fee: i64 = if is_new_account {
        let activation_fee = get_chain_parameter_value(chain_parameters, GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT)?;
        let new_account_bandwidth_fee = get_chain_parameter_value(chain_parameters, GET_CREATE_ACCOUNT_FEE)?;

        // Account activation only waives this fixed bandwidth fee when the sender has staked/delegated bandwidth.
        if account_usage.available_staked_bandwidth() >= DEFAULT_BANDWIDTH_BYTES {
            activation_fee
        } else {
            activation_fee + new_account_bandwidth_fee
        }
    } else {
        bandwidth_fee(account_usage, DEFAULT_BANDWIDTH_BYTES, bandwidth_price as u64) as i64
    };

    Ok(BigInt::from(base_fee + memo_fee))
}

pub fn calculate_transfer_token_fee_rate(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    estimated_energy: u64,
) -> Result<TokenTransferFee, Box<dyn Error + Send + Sync>> {
    calculate_transfer_token_fee_rate_with_data(chain_parameters, account_usage, estimated_energy, 0, false)
}

pub fn calculate_transfer_token_fee_rate_with_data(
    chain_parameters: &[ChainParameter],
    account_usage: &TronAccountUsage,
    estimated_energy: u64,
    data_bytes: u64,
    has_memo: bool,
) -> Result<TokenTransferFee, Box<dyn Error + Send + Sync>> {
    let energy_price = get_chain_parameter_value(chain_parameters, GET_ENERGY_FEE)? as u64;
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)? as u64;
    let memo_fee = if has_memo {
        get_chain_parameter_value(chain_parameters, GET_MEMO_FEE)? as u64
    } else {
        0
    };
    let bandwidth_bytes = DEFAULT_BANDWIDTH_BYTES.saturating_add(data_bytes);

    let fee = calculate_token_transfer_fee_for_bandwidth(account_usage, estimated_energy, energy_price, bandwidth_price, bandwidth_bytes, memo_fee);
    Ok(fee)
}

pub fn calculate_stake_fee_rate(chain_parameters: &[ChainParameter], account_usage: &TronAccountUsage, _stake_type: &StakeType) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let bandwidth_price = get_chain_parameter_value(chain_parameters, GET_TRANSACTION_FEE)? as u64;
    let fee = bandwidth_fee(account_usage, DEFAULT_BANDWIDTH_BYTES, bandwidth_price);
    Ok(BigInt::from(fee))
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenTransferFee {
    pub fee: u64,
    pub fee_limit: u64,
    pub energy_price: u64,
}

pub fn calculate_token_transfer_fee(account_usage: &TronAccountUsage, estimated_energy: u64, energy_price: u64, bandwidth_price: u64) -> TokenTransferFee {
    calculate_token_transfer_fee_for_bandwidth(account_usage, estimated_energy, energy_price, bandwidth_price, DEFAULT_BANDWIDTH_BYTES, 0)
}

fn calculate_token_transfer_fee_for_bandwidth(
    account_usage: &TronAccountUsage,
    estimated_energy: u64,
    energy_price: u64,
    bandwidth_price: u64,
    bandwidth_bytes: u64,
    memo_fee: u64,
) -> TokenTransferFee {
    let energy_with_buffer = apply_buffer(estimated_energy, FEE_LIMIT_BUFFER_PERCENT);
    let chargeable_energy = account_usage.missing_energy(energy_with_buffer);

    let energy_fee = chargeable_energy * energy_price;
    let bandwidth_fee = bandwidth_fee(account_usage, bandwidth_bytes, bandwidth_price);

    TokenTransferFee {
        fee: energy_fee + bandwidth_fee + memo_fee,
        fee_limit: energy_with_buffer * energy_price,
        energy_price,
    }
}

fn bandwidth_fee(account_usage: &TronAccountUsage, required: u64, price: u64) -> u64 {
    if account_usage.available_bandwidth() >= required { 0 } else { required * price }
}

fn apply_buffer(value: u64, percent: u64) -> u64 {
    value * (100 + percent) / 100
}

fn get_chain_parameter_value(parameters: &[ChainParameter], key: &str) -> Result<i64, Box<dyn Error + Send + Sync>> {
    parameters
        .iter()
        .find(|param| param.key == key)
        .and_then(|param| param.value)
        .ok_or_else(|| format!("Missing chain parameter: {}", key).into())
}

fn calculate_unfreeze_amounts(frozen: Option<&[TronFrozen]>, total: u64) -> Vec<TronUnfreeze> {
    frozen
        .map(|frozen| {
            frozen
                .iter()
                .filter_map(|frozen| frozen.resource().map(|resource| (resource, frozen.amount)))
                .filter(|(_, amount)| *amount > 0)
                .scan(total, |remaining, (resource, amount)| {
                    (*remaining > 0).then(|| {
                        let take = (*remaining).min(amount);
                        *remaining -= take;
                        TronUnfreeze { resource, amount: take }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn map_stake_data(account: &TronAccount, stake_type: &StakeType, raw_amount: u64, vote_amount: u64) -> Result<TronStakeData, Box<dyn Error + Send + Sync>> {
    let mut votes: HashMap<String, u64> = account
        .votes
        .as_ref()
        .map(|votes| votes.iter().map(|vote| (vote.vote_address.clone(), vote.vote_count)).collect())
        .unwrap_or_default();

    match stake_type {
        StakeType::Stake(validator) => *votes.entry(validator.id.clone()).or_default() += vote_amount,
        StakeType::Unstake(delegation) => {
            votes
                .entry(delegation.base.validator_id.clone())
                .and_modify(|value| *value = value.saturating_sub(vote_amount));
            votes.retain(|_, value| *value > 0);
            if votes.is_empty() {
                return Ok(TronStakeData::Unfreeze(calculate_unfreeze_amounts(account.frozen_v2.as_deref(), raw_amount)));
            }
        }
        StakeType::Redelegate(data) => {
            votes
                .entry(data.delegation.base.validator_id.clone())
                .and_modify(|value| *value = value.saturating_sub(vote_amount));
            *votes.entry(data.to_validator.id.clone()).or_default() += vote_amount;
        }
        StakeType::Rewards(_) | StakeType::Withdraw(_) | StakeType::Freeze(_) => {}
        StakeType::Unfreeze(resource) => {
            let available = account
                .frozen_v2
                .as_deref()
                .unwrap_or_default()
                .iter()
                .filter(|frozen| frozen.resource() == Some(*resource))
                .map(|frozen| frozen.amount)
                .sum::<u64>();
            if raw_amount > available {
                return Err(format!("Insufficient frozen {} balance: requested {}, available {}", resource.as_ref(), raw_amount, available).into());
            }
            return Ok(TronStakeData::Unfreeze(vec![TronUnfreeze {
                resource: *resource,
                amount: raw_amount,
            }]));
        }
    }

    Ok(TronStakeData::Votes(
        votes
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(validator, count)| TronVote { validator, count })
            .collect(),
    ))
}

impl TronAccountUsage {
    pub fn available_bandwidth(&self) -> u64 {
        let free = self.free_net_limit.saturating_sub(self.free_net_used);
        free.saturating_add(self.available_staked_bandwidth())
    }

    pub fn available_staked_bandwidth(&self) -> u64 {
        self.net_limit.saturating_sub(self.net_used)
    }

    pub fn missing_bandwidth(&self, required: u64) -> u64 {
        required.saturating_sub(self.available_bandwidth())
    }

    pub fn available_energy(&self) -> u64 {
        self.energy_limit.saturating_sub(self.energy_used)
    }

    pub fn missing_energy(&self, required: u64) -> u64 {
        required.saturating_sub(self.available_energy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::{TronAccount, TronFrozen, TronVote as AccountVote};
    use primitives::{Chain, Delegation, DelegationValidator, Resource};

    fn chain_parameter(key: &str, value: i64) -> ChainParameter {
        ChainParameter {
            key: key.to_string(),
            value: Some(value),
        }
    }

    fn account_usage(free_bandwidth: u64, staked_bandwidth: u64, available_energy: u64) -> TronAccountUsage {
        TronAccountUsage {
            free_net_used: 0,
            free_net_limit: free_bandwidth,
            net_used: 0,
            net_limit: staked_bandwidth,
            energy_used: 0,
            energy_limit: available_energy,
        }
    }

    #[test]
    fn test_apply_buffer() {
        assert_eq!(apply_buffer(100, 20), 120);
        assert_eq!(apply_buffer(64285, 20), 77142);
        assert_eq!(apply_buffer(1000, 0), 1000);
    }

    #[test]
    fn test_account_usage_bandwidth() {
        let usage = TronAccountUsage {
            free_net_used: 100,
            free_net_limit: 1000,
            net_used: 200,
            net_limit: 500,
            energy_used: 0,
            energy_limit: 0,
        };

        assert_eq!(usage.available_bandwidth(), 1200); // (1000-100) + (500-200)
        assert_eq!(usage.available_staked_bandwidth(), 300);
        assert_eq!(usage.missing_bandwidth(1500), 300);
        assert_eq!(usage.missing_bandwidth(1000), 0);
    }

    #[test]
    fn test_account_usage_energy() {
        let usage = TronAccountUsage {
            free_net_used: 0,
            free_net_limit: 0,
            net_used: 0,
            net_limit: 0,
            energy_used: 10000,
            energy_limit: 60000,
        };

        assert_eq!(usage.available_energy(), 50000);
        assert_eq!(usage.missing_energy(70000), 20000);
        assert_eq!(usage.missing_energy(40000), 0);
    }

    #[test]
    fn test_calculate_token_transfer_fee_no_staked_resources() {
        let usage = account_usage(0, 0, 0);

        let fee = calculate_token_transfer_fee(&usage, 64285, 420, 1000);

        // energy_with_buffer = 64285 * 1.2 = 77142
        // chargeable_energy = 77142 (no staked energy)
        // energy_fee = 77142 * 420 = 32,399,640
        // bandwidth_fee = 345 * 1000 = 345,000
        assert_eq!(fee.fee, 77142 * 420 + DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(fee.fee_limit, 77142 * 420);
        assert_eq!(fee.energy_price, 420);
    }

    #[test]
    fn test_calculate_token_transfer_fee_with_staked_energy() {
        let usage = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 60000);

        let fee = calculate_token_transfer_fee(&usage, 64285, 420, 1000);

        // energy_with_buffer = 77142
        // chargeable_energy = 77142 - 60000 = 17142
        // energy_fee = 17142 * 420 = 7,199,640
        // bandwidth_fee = 0 (has enough)
        assert_eq!(fee.fee, 17142 * 420);
        assert_eq!(fee.fee_limit, 77142 * 420);
    }

    #[test]
    fn test_calculate_token_transfer_fee_with_full_coverage() {
        let usage = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 100000);

        let fee = calculate_token_transfer_fee(&usage, 64285, 420, 1000);

        // User has more than enough staked energy
        assert_eq!(fee.fee, 0);
        assert_eq!(fee.fee_limit, 77142 * 420);
    }

    #[test]
    fn test_calculate_transfer_token_fee_rate_with_data() {
        let params = vec![
            chain_parameter(GET_ENERGY_FEE, 420),
            chain_parameter(GET_TRANSACTION_FEE, 1000),
            chain_parameter(GET_MEMO_FEE, 1_000_000),
        ];
        let usage = account_usage(0, 0, 0);

        let fee = calculate_transfer_token_fee_rate_with_data(&params, &usage, 100, 65, true).unwrap();

        assert_eq!(fee.fee, 120 * 420 + (DEFAULT_BANDWIDTH_BYTES + 65) * 1000 + 1_000_000);
        assert_eq!(fee.fee_limit, 120 * 420);
        assert_eq!(fee.energy_price, 420);
    }

    #[test]
    fn test_calculate_transfer_fee_rate_existing_account() {
        let params = vec![chain_parameter(GET_TRANSACTION_FEE, 1000), chain_parameter(GET_MEMO_FEE, 1_000_000)];

        let with_bandwidth = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 0);
        assert_eq!(calculate_transfer_fee_rate(&params, &with_bandwidth, false, false).unwrap(), BigInt::from(0));
        assert_eq!(
            calculate_transfer_fee_rate(&params, &with_bandwidth, false, true).unwrap(),
            BigInt::from(1_000_000), // memo fee only
        );

        let without_bandwidth = account_usage(100, 0, 0);
        let burn_bandwidth = DEFAULT_BANDWIDTH_BYTES * 1000;
        assert_eq!(
            calculate_transfer_fee_rate(&params, &without_bandwidth, false, false).unwrap(),
            BigInt::from(burn_bandwidth),
        );
        assert_eq!(
            calculate_transfer_fee_rate(&params, &without_bandwidth, false, true).unwrap(),
            BigInt::from(burn_bandwidth + 1_000_000),
        );
    }

    #[test]
    fn test_calculate_transfer_fee_rate_new_account() {
        let params = vec![
            chain_parameter(GET_TRANSACTION_FEE, 1000),
            chain_parameter(GET_CREATE_ACCOUNT_FEE, 100_000),
            chain_parameter(GET_CREATE_NEW_ACCOUNT_FEE_IN_SYSTEM_CONTRACT, 1_000_000),
            chain_parameter(GET_MEMO_FEE, 1_000_000),
        ];

        let without_bandwidth = account_usage(0, 0, 0);
        assert_eq!(
            calculate_transfer_fee_rate(&params, &without_bandwidth, true, false).unwrap(),
            BigInt::from(1_100_000), // activation + bandwidth
        );
        assert_eq!(
            calculate_transfer_fee_rate(&params, &without_bandwidth, true, true).unwrap(),
            BigInt::from(2_100_000), // activation + bandwidth + memo
        );

        let with_free_bandwidth = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 0);
        assert_eq!(
            calculate_transfer_fee_rate(&params, &with_free_bandwidth, true, false).unwrap(),
            BigInt::from(1_100_000), // activation + fixed account creation bandwidth fee
        );

        let with_staked_bandwidth = account_usage(0, DEFAULT_BANDWIDTH_BYTES, 0);
        assert_eq!(
            calculate_transfer_fee_rate(&params, &with_staked_bandwidth, true, false).unwrap(),
            BigInt::from(1_000_000), // only activation
        );
    }

    #[test]
    fn test_calculate_stake_fee_rate() {
        let params = vec![chain_parameter(GET_TRANSACTION_FEE, 1000)];
        let stake_type = StakeType::Stake(DelegationValidator::stake(Chain::Tron, "validator".to_string(), "validator".to_string(), true, 0.0, 0.0));

        let with_bandwidth = account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 0);
        assert_eq!(calculate_stake_fee_rate(&params, &with_bandwidth, &stake_type).unwrap(), BigInt::from(0));

        let without_bandwidth = account_usage(100, 0, 0);
        let expected = BigInt::from(DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(calculate_stake_fee_rate(&params, &without_bandwidth, &stake_type).unwrap(), expected);
    }

    #[test]
    fn test_get_chain_parameter_value() {
        let params = vec![chain_parameter(GET_ENERGY_FEE, 420), chain_parameter(GET_TRANSACTION_FEE, 1000)];

        assert_eq!(get_chain_parameter_value(&params, GET_ENERGY_FEE).unwrap(), 420);
        assert_eq!(get_chain_parameter_value(&params, GET_TRANSACTION_FEE).unwrap(), 1000);
        assert!(get_chain_parameter_value(&params, "missing").is_err());
    }

    #[test]
    fn test_bandwidth_fee() {
        assert_eq!(bandwidth_fee(&account_usage(DEFAULT_BANDWIDTH_BYTES, 0, 0), DEFAULT_BANDWIDTH_BYTES, 1000), 0);
        assert_eq!(bandwidth_fee(&account_usage(100, 200, 0), DEFAULT_BANDWIDTH_BYTES, 1000), 0);
        assert_eq!(bandwidth_fee(&account_usage(0, 0, 0), DEFAULT_BANDWIDTH_BYTES, 1000), DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(bandwidth_fee(&account_usage(76, 0, 0), DEFAULT_BANDWIDTH_BYTES, 1000), DEFAULT_BANDWIDTH_BYTES * 1000);
    }

    #[test]
    fn test_calculate_token_transfer_fee_partial_bandwidth() {
        let fee = calculate_token_transfer_fee(&account_usage(76, 0, 0), 64285, 420, 1000);

        assert_eq!(fee.fee, 77142 * 420 + DEFAULT_BANDWIDTH_BYTES * 1000);
        assert_eq!(fee.fee_limit, 77142 * 420);
    }

    #[test]
    fn test_calculate_unfreeze_amounts() {
        let frozen = vec![
            TronFrozen {
                frozen_type: Some("TRON_POWER".to_string()),
                amount: 1_000,
            },
            TronFrozen {
                frozen_type: Some("UNKNOWN".to_string()),
                amount: 1_000,
            },
            TronFrozen { frozen_type: None, amount: 40 },
            TronFrozen {
                frozen_type: Some("ENERGY".to_string()),
                amount: 100,
            },
            TronFrozen {
                frozen_type: Some("BANDWIDTH".to_string()),
                amount: 50,
            },
        ];

        assert_eq!(
            calculate_unfreeze_amounts(Some(&frozen), 120),
            vec![
                TronUnfreeze {
                    resource: Resource::Bandwidth,
                    amount: 40
                },
                TronUnfreeze {
                    resource: Resource::Energy,
                    amount: 80
                },
            ]
        );
        assert_eq!(
            calculate_unfreeze_amounts(Some(&frozen), 50),
            vec![
                TronUnfreeze {
                    resource: Resource::Bandwidth,
                    amount: 40
                },
                TronUnfreeze {
                    resource: Resource::Energy,
                    amount: 10
                },
            ]
        );
        assert!(calculate_unfreeze_amounts(None, 100).is_empty());
        assert!(calculate_unfreeze_amounts(Some(&frozen), 0).is_empty());
    }

    #[test]
    fn test_map_stake_data_unfreeze_uses_raw_amount_when_available() {
        let account = TronAccount::mock_with_staking(
            None,
            Some(vec![TronFrozen {
                frozen_type: Some("BANDWIDTH".to_string()),
                amount: 1_500_000,
            }]),
        );
        let result = map_stake_data(&account, &StakeType::Unfreeze(Resource::Bandwidth), 1_000_000, 1).unwrap();

        assert_eq!(
            result,
            TronStakeData::Unfreeze(vec![TronUnfreeze {
                resource: Resource::Bandwidth,
                amount: 1_000_000,
            }])
        );

        let result = map_stake_data(&account, &StakeType::Unfreeze(Resource::Bandwidth), 1_500_000, 1).unwrap();

        assert_eq!(
            result,
            TronStakeData::Unfreeze(vec![TronUnfreeze {
                resource: Resource::Bandwidth,
                amount: 1_500_000,
            }])
        );
    }

    #[test]
    fn test_map_stake_data_unfreeze_rejects_wrong_resource() {
        let account = TronAccount::mock_with_staking(
            None,
            Some(vec![TronFrozen {
                frozen_type: Some("ENERGY".to_string()),
                amount: 2_000_000,
            }]),
        );

        let result = map_stake_data(&account, &StakeType::Unfreeze(Resource::Bandwidth), 1_000_000, 1);

        assert_eq!(result.unwrap_err().to_string(), "Insufficient frozen bandwidth balance: requested 1000000, available 0");
    }

    #[test]
    fn test_map_stake_data_unfreeze_rejects_excess_amount() {
        let account = TronAccount::mock_with_staking(
            None,
            Some(vec![TronFrozen {
                frozen_type: Some("BANDWIDTH".to_string()),
                amount: 999_999,
            }]),
        );

        let result = map_stake_data(&account, &StakeType::Unfreeze(Resource::Bandwidth), 1_000_000, 1);

        assert_eq!(
            result.unwrap_err().to_string(),
            "Insufficient frozen bandwidth balance: requested 1000000, available 999999"
        );
    }

    #[test]
    fn test_map_stake_data_unstake_keeps_vote_updates() {
        let result = map_stake_data(
            &TronAccount::mock_with_staking(
                Some(vec![AccountVote {
                    vote_address: "validator".to_string(),
                    vote_count: 5,
                }]),
                None,
            ),
            &StakeType::Unstake(Delegation::mock_tron("validator")),
            2_000_000,
            2,
        )
        .unwrap();

        assert_eq!(
            result,
            TronStakeData::Votes(vec![TronVote {
                validator: "validator".to_string(),
                count: 3,
            }])
        );
    }

    #[test]
    fn test_map_stake_data_unstake_keeps_existing_unfreeze_split() {
        let result = map_stake_data(
            &TronAccount::mock_with_staking(
                Some(vec![AccountVote {
                    vote_address: "validator".to_string(),
                    vote_count: 2,
                }]),
                Some(vec![
                    TronFrozen {
                        frozen_type: Some("ENERGY".to_string()),
                        amount: 100,
                    },
                    TronFrozen {
                        frozen_type: Some("BANDWIDTH".to_string()),
                        amount: 50,
                    },
                ]),
            ),
            &StakeType::Unstake(Delegation::mock_tron("validator")),
            120,
            2,
        )
        .unwrap();

        assert_eq!(
            result,
            TronStakeData::Unfreeze(vec![
                TronUnfreeze {
                    resource: Resource::Energy,
                    amount: 100,
                },
                TronUnfreeze {
                    resource: Resource::Bandwidth,
                    amount: 20,
                },
            ])
        );
    }
}
