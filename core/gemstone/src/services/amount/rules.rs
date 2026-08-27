use std::str::FromStr;

use num_bigint::{BigInt, BigUint};
use primitives::{Asset, Chain, Resource, StakeChain};

use super::model::{GemAmountBalance, GemAmountEarnType, GemAmountError, GemAmountLimits, GemAmountPerpetualPosition, GemAmountRules, GemAmountStakeType, GemAmountType};
use crate::config::perpetual_config::{MIN_DEPOSIT_AMOUNT, MIN_WITHDRAW_AMOUNT};
use crate::config::stake::get_stake_config;
use gem_hypercore::perpetual_formatter::PerpetualFormatter;

const USDC_SYMBOL: &str = "USDC";

pub fn rules(amount_type: &GemAmountType, asset: &Asset) -> GemAmountRules {
    let minimum_value = minimum_value(amount_type, asset);
    GemAmountRules {
        minimum_value: minimum_value.to_string(),
        reserve_for_fee: reserve_for_fee(amount_type, asset).to_string(),
        can_change_value: can_change_value(amount_type, asset),
        shows_asset_balance: shows_asset_balance(amount_type, asset),
    }
}

pub fn limits(amount_type: &GemAmountType, asset: &Asset, balance: &GemAmountBalance) -> GemAmountLimits {
    let available = available_value(amount_type, asset, balance);
    let reserve = reserve_for_fee(amount_type, asset);
    let max_after_fee = (&available - &reserve).max(BigInt::from(0));
    let reserves_fee = reserves_fee(amount_type, &reserve, &max_after_fee, &minimum_value(amount_type, asset));
    GemAmountLimits {
        available_value: available.to_string(),
        max_value: if reserves_fee { max_after_fee.to_string() } else { available.to_string() },
        reserves_fee,
    }
}

pub fn validate(value: &BigInt, available: &BigInt, minimum: &BigInt) -> Result<(), GemAmountError> {
    if value <= &BigInt::from(0) {
        return Err(GemAmountError::Zero);
    }
    if value < minimum {
        return Err(GemAmountError::BelowMinimum { minimum: minimum.to_string() });
    }
    if value > available {
        return Err(GemAmountError::InsufficientBalance { available: available.to_string() });
    }
    Ok(())
}

fn minimum_value(amount_type: &GemAmountType, asset: &Asset) -> BigInt {
    let stake_config = stake_chain(asset.chain()).map(get_stake_config);
    match amount_type {
        GemAmountType::Transfer | GemAmountType::Earn { .. } => BigInt::from(0),
        GemAmountType::Deposit => usdc_minimum(asset, MIN_DEPOSIT_AMOUNT),
        GemAmountType::Withdraw => usdc_minimum(asset, MIN_WITHDRAW_AMOUNT),
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. } => stake_config.map(|config| BigInt::from(config.min_amount)).unwrap_or_default(),
            GemAmountStakeType::Redelegate { .. } if asset.chain() == Chain::SmartChain => stake_config.map(|config| BigInt::from(config.min_amount)).unwrap_or_default(),
            GemAmountStakeType::Withdraw { .. } => usdc_minimum(asset, MIN_DEPOSIT_AMOUNT),
            GemAmountStakeType::Redelegate { .. } | GemAmountStakeType::Unstake { .. } | GemAmountStakeType::Unfreeze { .. } | GemAmountStakeType::Rewards { .. } => {
                BigInt::from(0)
            }
        },
        GemAmountType::Perpetual { price, leverage, .. } => BigInt::from(PerpetualFormatter::minimum_order_usd_amount(*price, asset.decimals, *leverage)),
    }
}

fn reserve_for_fee(amount_type: &GemAmountType, asset: &Asset) -> BigInt {
    let reserved = stake_chain(asset.chain())
        .map(|chain| BigInt::from(get_stake_config(chain).reserved_for_fees))
        .unwrap_or_default();
    match amount_type {
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Stake if asset.chain() != Chain::Tron => reserved,
            GemAmountStakeType::Freeze { .. } => reserved,
            _ => BigInt::from(0),
        },
        _ => BigInt::from(0),
    }
}

fn can_change_value(amount_type: &GemAmountType, asset: &Asset) -> bool {
    match amount_type {
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Unstake { .. } => stake_chain(asset.chain()).map(|chain| get_stake_config(chain).change_amount_on_unstake).unwrap_or(true),
            GemAmountStakeType::Withdraw { .. } | GemAmountStakeType::Rewards { .. } => false,
            GemAmountStakeType::Stake | GemAmountStakeType::Redelegate { .. } | GemAmountStakeType::Freeze { .. } | GemAmountStakeType::Unfreeze { .. } => true,
        },
        _ => true,
    }
}

fn shows_asset_balance(amount_type: &GemAmountType, asset: &Asset) -> bool {
    match amount_type {
        GemAmountType::Stake {
            stake_type: GemAmountStakeType::Rewards { .. },
        } => true,
        _ => can_change_value(amount_type, asset),
    }
}

fn reserves_fee(amount_type: &GemAmountType, reserve: &BigInt, max_after_fee: &BigInt, minimum: &BigInt) -> bool {
    match amount_type {
        GemAmountType::Stake {
            stake_type: GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. },
        } => reserve > &BigInt::from(0) && max_after_fee > minimum,
        _ => false,
    }
}

pub fn available_value(amount_type: &GemAmountType, asset: &Asset, balance: &GemAmountBalance) -> BigInt {
    let parse = |value: &str| value.parse::<BigInt>().unwrap_or_default();
    match amount_type {
        GemAmountType::Transfer | GemAmountType::Deposit => parse(&balance.available),
        GemAmountType::Withdraw => parse(&balance.withdrawable),
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Stake if asset.chain() == Chain::Tron => {
                let staked = BigInt::from(balance.votes) * BigInt::from(10u32).pow(asset.decimals.max(0) as u32);
                parse(&balance.frozen) + parse(&balance.locked) - staked
            }
            GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. } => parse(&balance.available),
            GemAmountStakeType::Unstake { delegation } | GemAmountStakeType::Redelegate { delegation } | GemAmountStakeType::Withdraw { delegation } => {
                BigInt::from(delegation.base.balance.clone())
            }
            GemAmountStakeType::Rewards { delegations } => BigInt::from(delegations.iter().map(|delegation| delegation.base.rewards.clone()).sum::<BigUint>()),
            GemAmountStakeType::Unfreeze { resource } => match resource {
                Resource::Bandwidth => parse(&balance.frozen),
                Resource::Energy => parse(&balance.locked),
            },
        },
        GemAmountType::Earn { earn_type } => match earn_type {
            GemAmountEarnType::Deposit => parse(&balance.available),
            GemAmountEarnType::Withdraw { delegation } => BigInt::from(delegation.base.balance.clone()),
        },
        GemAmountType::Perpetual { position, .. } => match position {
            GemAmountPerpetualPosition::Open | GemAmountPerpetualPosition::Increase => parse(&balance.available),
            GemAmountPerpetualPosition::Reduce { available } => parse(available),
        },
    }
}

fn usdc_minimum(asset: &Asset, minimum: u64) -> BigInt {
    if asset.symbol == USDC_SYMBOL { BigInt::from(minimum) } else { BigInt::from(0) }
}

fn stake_chain(chain: Chain) -> Option<StakeChain> {
    StakeChain::from_str(chain.as_ref()).ok()
}
