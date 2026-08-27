use std::str::FromStr;

use num_bigint::{BigInt, BigUint};
use primitives::{Asset, Chain, StakeChain};

use super::model::{GemAmountEarnType, GemAmountError, GemAmountLimits, GemAmountPerpetualPosition, GemAmountRules, GemAmountStakeType, GemAmountType};
use crate::config::perpetual_config::{MIN_DEPOSIT_AMOUNT, MIN_WITHDRAW_AMOUNT};
use crate::config::stake::get_stake_config;
use crate::services::transfer::GemTransferBalance;
use crate::services::transfer::rules as transfer_rules;
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

pub fn limits(amount_type: &GemAmountType, asset: &Asset, balance: &GemTransferBalance) -> GemAmountLimits {
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

pub fn parse_value(value: &str) -> Result<BigInt, GemAmountError> {
    value.parse::<BigInt>().map_err(|_| GemAmountError::InvalidValue { value: value.to_string() })
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
        GemAmountType::Perpetual {
            price, leverage, size_decimals, ..
        } => BigInt::from(PerpetualFormatter::minimum_order_usd_amount(*price, *size_decimals, *leverage)),
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

pub fn available_value(amount_type: &GemAmountType, asset: &Asset, balance: &GemTransferBalance) -> BigInt {
    let parse = |value: &str| value.parse::<BigInt>().unwrap_or_default();
    match amount_type {
        GemAmountType::Transfer | GemAmountType::Deposit => parse(&balance.available),
        GemAmountType::Withdraw => parse(&balance.withdrawable),
        GemAmountType::Stake { stake_type } => match stake_type {
            GemAmountStakeType::Stake if asset.chain() == Chain::Tron => transfer_rules::tron_stake_available(asset, balance),
            GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. } => parse(&balance.available),
            GemAmountStakeType::Unstake { delegation } | GemAmountStakeType::Redelegate { delegation } | GemAmountStakeType::Withdraw { delegation } => {
                BigInt::from(delegation.base.balance.clone())
            }
            GemAmountStakeType::Rewards { delegations } => BigInt::from(delegations.iter().map(|delegation| delegation.base.rewards.clone()).sum::<BigUint>()),
            GemAmountStakeType::Unfreeze { resource } => transfer_rules::unfreeze_available(resource, balance),
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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Resource;
    use primitives::{AssetId, AssetType, Delegation, DelegationBase, DelegationState, DelegationValidator, StakeProviderType};

    fn asset(chain: Chain) -> Asset {
        Asset::from_chain(chain)
    }

    fn usdc() -> Asset {
        Asset::new(AssetId::from(Chain::HyperCore, Some("usdc".into())), "USDC".into(), "USDC".into(), 6, AssetType::TOKEN)
    }

    fn balance(available: u64, frozen: u64, locked: u64, votes: u32) -> GemTransferBalance {
        GemTransferBalance {
            available: available.to_string(),
            frozen: frozen.to_string(),
            locked: locked.to_string(),
            withdrawable: "7".to_string(),
            votes,
        }
    }

    fn delegation(balance: u64, rewards: u64) -> Delegation {
        Delegation {
            base: DelegationBase {
                asset_id: AssetId::from_chain(Chain::Cosmos),
                state: DelegationState::Active,
                balance: BigUint::from(balance),
                shares: BigUint::default(),
                rewards: BigUint::from(rewards),
                completion_date: None,
                delegation_id: "delegation".into(),
                validator_id: "validator".into(),
            },
            validator: DelegationValidator {
                chain: Chain::Cosmos,
                id: "validator".into(),
                name: "validator".into(),
                is_active: true,
                commission: 0.0,
                apr: 0.0,
                provider_type: StakeProviderType::Stake,
            },
            price: None,
        }
    }

    fn stake(stake_type: GemAmountStakeType) -> GemAmountType {
        GemAmountType::Stake { stake_type }
    }

    #[test]
    fn test_stake_rules_reserve_fees_and_minimums() {
        let cosmos = asset(Chain::Cosmos);
        let config = get_stake_config(StakeChain::Cosmos);
        let stake_rules = rules(&stake(GemAmountStakeType::Stake), &cosmos);
        assert_eq!(stake_rules.minimum_value, config.min_amount.to_string());
        assert_eq!(stake_rules.reserve_for_fee, config.reserved_for_fees.to_string());
        assert!(stake_rules.can_change_value);

        let limits = limits(
            &stake(GemAmountStakeType::Stake),
            &cosmos,
            &balance(config.reserved_for_fees * 10 + config.min_amount * 10, 0, 0, 0),
        );
        assert!(limits.reserves_fee);
        assert_eq!(limits.max_value, (config.reserved_for_fees * 9 + config.min_amount * 10).to_string());

        let tron = asset(Chain::Tron);
        let tron_rules = rules(&stake(GemAmountStakeType::Stake), &tron);
        assert_eq!(tron_rules.reserve_for_fee, "0");
        assert_eq!(
            available_value(&stake(GemAmountStakeType::Stake), &tron, &balance(1, 5_000_000, 3_000_000, 2)),
            BigInt::from(6_000_000)
        );

        let rewards = rules(&stake(GemAmountStakeType::Rewards { delegations: vec![] }), &cosmos);
        assert!(!rewards.can_change_value);
        assert!(rewards.shows_asset_balance);
        assert_eq!(
            available_value(
                &stake(GemAmountStakeType::Rewards {
                    delegations: vec![delegation(10, 3), delegation(20, 4)]
                }),
                &cosmos,
                &balance(1, 0, 0, 0)
            ),
            BigInt::from(7)
        );
        assert_eq!(
            available_value(&stake(GemAmountStakeType::Unstake { delegation: delegation(50, 0) }), &cosmos, &balance(1, 0, 0, 0)),
            BigInt::from(50)
        );
        assert_eq!(
            available_value(&stake(GemAmountStakeType::Unfreeze { resource: Resource::Energy }), &tron, &balance(1, 2, 3, 0)),
            BigInt::from(3)
        );
    }

    #[test]
    fn test_transfer_deposit_withdraw_rules() {
        assert_eq!(rules(&GemAmountType::Transfer, &asset(Chain::Ethereum)).minimum_value, "0");
        assert_eq!(rules(&GemAmountType::Deposit, &usdc()).minimum_value, MIN_DEPOSIT_AMOUNT.to_string());
        assert_eq!(rules(&GemAmountType::Withdraw, &usdc()).minimum_value, MIN_WITHDRAW_AMOUNT.to_string());
        assert_eq!(rules(&GemAmountType::Deposit, &asset(Chain::Ethereum)).minimum_value, "0");
        assert_eq!(available_value(&GemAmountType::Withdraw, &usdc(), &balance(1, 0, 0, 0)), BigInt::from(7));
        assert_eq!(
            available_value(
                &GemAmountType::Perpetual {
                    position: GemAmountPerpetualPosition::Reduce { available: "42".into() },
                    price: 1.0,
                    leverage: 1,
                    size_decimals: 0
                },
                &usdc(),
                &balance(1, 0, 0, 0)
            ),
            BigInt::from(42)
        );
    }

    #[test]
    fn test_validate() {
        assert_eq!(validate(&BigInt::from(0), &BigInt::from(10), &BigInt::from(0)), Err(GemAmountError::Zero));
        assert_eq!(
            validate(&BigInt::from(1), &BigInt::from(10), &BigInt::from(2)),
            Err(GemAmountError::BelowMinimum { minimum: "2".into() })
        );
        assert_eq!(
            validate(&BigInt::from(11), &BigInt::from(10), &BigInt::from(0)),
            Err(GemAmountError::InsufficientBalance { available: "10".into() })
        );
        assert_eq!(validate(&BigInt::from(5), &BigInt::from(10), &BigInt::from(2)), Ok(()));
    }
}
