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

#[uniffi::export]
impl GemAmountType {
    pub fn rules(&self, asset: &Asset) -> GemAmountRules {
        GemAmountRules {
            minimum_value: minimum_value(self, asset),
            reserve_for_fee: reserve_for_fee(self, asset),
            can_change_value: can_change_value(self, asset),
            shows_asset_balance: shows_asset_balance(self, asset),
        }
    }

    pub fn limits(&self, asset: &Asset, balance: &GemTransferBalance) -> Result<GemAmountLimits, GemAmountError> {
        let available = self.available_value(asset, balance)?;
        let reserve = reserve_for_fee(self, asset);
        let max_after_fee = (&available - &reserve).max(BigInt::from(0));
        let reserves_fee = reserves_fee(self, &reserve, &max_after_fee, &minimum_value(self, asset));
        Ok(GemAmountLimits {
            available_value: available.clone(),
            max_value: if reserves_fee { max_after_fee } else { available },
            reserves_fee,
        })
    }

    pub fn validate(&self, asset: &Asset, balance: &GemTransferBalance, value: String) -> Result<(), GemAmountError> {
        validate(&parse_value(&value)?, &self.available_value(asset, balance)?, &minimum_value(self, asset))
    }
}

impl GemAmountType {
    fn available_value(&self, asset: &Asset, balance: &GemTransferBalance) -> Result<BigInt, GemAmountError> {
        Ok(match self {
            Self::Transfer | Self::Deposit => balance.available.clone(),
            Self::Withdraw => balance.withdrawable.clone(),
            Self::Stake { stake_type } => match stake_type {
                GemAmountStakeType::Stake if asset.chain() == Chain::Tron => transfer_rules::tron_stake_available(asset, balance),
                GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. } => balance.available.clone(),
                GemAmountStakeType::Unstake { delegation } | GemAmountStakeType::Redelegate { delegation } | GemAmountStakeType::Withdraw { delegation } => {
                    BigInt::from(delegation.base.balance.clone())
                }
                GemAmountStakeType::Rewards { delegations } => BigInt::from(delegations.iter().map(|delegation| delegation.base.rewards.clone()).sum::<BigUint>()),
                GemAmountStakeType::Unfreeze { resource } => transfer_rules::unfreeze_available(resource, balance),
            },
            Self::Earn { earn_type } => match earn_type {
                GemAmountEarnType::Deposit => balance.available.clone(),
                GemAmountEarnType::Withdraw { delegation } => BigInt::from(delegation.base.balance.clone()),
            },
            Self::Perpetual { position, .. } => match position {
                GemAmountPerpetualPosition::Open | GemAmountPerpetualPosition::Increase => balance.available.clone(),
                GemAmountPerpetualPosition::Reduce { available } => available.clone(),
            },
        })
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
            GemAmountStakeType::Withdraw { .. }
            | GemAmountStakeType::Redelegate { .. }
            | GemAmountStakeType::Unstake { .. }
            | GemAmountStakeType::Unfreeze { .. }
            | GemAmountStakeType::Rewards { .. } => BigInt::from(0),
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

fn usdc_minimum(asset: &Asset, minimum: u64) -> BigInt {
    if asset.symbol == USDC_SYMBOL { BigInt::from(minimum) } else { BigInt::from(0) }
}

fn stake_chain(chain: Chain) -> Option<StakeChain> {
    StakeChain::from_str(chain.as_ref()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::custom_types::GemBigInt;
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
            available: BigInt::from(available),
            frozen: BigInt::from(frozen),
            locked: BigInt::from(locked),
            withdrawable: BigInt::from(7),
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
        let stake_rules = stake(GemAmountStakeType::Stake).rules(&cosmos);
        assert_eq!(stake_rules.minimum_value, BigInt::from(config.min_amount));
        assert_eq!(stake_rules.reserve_for_fee, BigInt::from(config.reserved_for_fees));
        assert!(stake_rules.can_change_value);

        let stake_limits = stake(GemAmountStakeType::Stake)
            .limits(&cosmos, &balance(config.reserved_for_fees * 10 + config.min_amount * 10, 0, 0, 0))
            .unwrap();
        assert!(stake_limits.reserves_fee);
        assert_eq!(stake_limits.max_value, BigInt::from(config.reserved_for_fees * 9 + config.min_amount * 10));

        let tron = asset(Chain::Tron);
        let tron_config = get_stake_config(StakeChain::Tron);
        let tron_rules = stake(GemAmountStakeType::Stake).rules(&tron);
        assert_eq!(tron_rules.reserve_for_fee, BigInt::ZERO);
        assert_eq!(
            stake(GemAmountStakeType::Stake).available_value(&tron, &balance(1, 5_000_000, 3_000_000, 2)).unwrap(),
            BigInt::from(6_000_000)
        );
        let freeze = stake(GemAmountStakeType::Freeze { resource: Resource::Bandwidth });
        assert!(tron_config.reserved_for_fees > 0);
        assert_eq!(freeze.rules(&tron).reserve_for_fee, BigInt::from(tron_config.reserved_for_fees));
        let freeze_limits = freeze
            .limits(&tron, &balance(tron_config.reserved_for_fees + tron_config.min_amount + 1, 99, 98, 0))
            .unwrap();
        assert_eq!(freeze_limits.available_value, BigInt::from(tron_config.reserved_for_fees + tron_config.min_amount + 1));
        assert!(freeze_limits.reserves_fee);
        assert_eq!(freeze_limits.max_value, BigInt::from(tron_config.min_amount + 1));

        let smart_chain = asset(Chain::SmartChain);
        let redelegate = stake(GemAmountStakeType::Redelegate { delegation: delegation(50, 0) });
        let smart_chain_minimum = get_stake_config(StakeChain::SmartChain).min_amount;
        assert!(smart_chain_minimum > 0);
        assert_eq!(redelegate.rules(&smart_chain).minimum_value, BigInt::from(smart_chain_minimum));
        assert_eq!(redelegate.rules(&cosmos).minimum_value, BigInt::ZERO);

        let unstake = stake(GemAmountStakeType::Unstake { delegation: delegation(50, 0) });
        let solana_unstake = unstake.rules(&asset(Chain::Solana));
        assert!(!solana_unstake.can_change_value);
        assert!(!solana_unstake.shows_asset_balance);
        let cosmos_unstake = unstake.rules(&cosmos);
        assert!(cosmos_unstake.can_change_value);
        assert!(cosmos_unstake.shows_asset_balance);

        let rewards = stake(GemAmountStakeType::Rewards { delegations: vec![] }).rules(&cosmos);
        assert!(!rewards.can_change_value);
        assert!(rewards.shows_asset_balance);
        assert_eq!(
            stake(GemAmountStakeType::Rewards {
                delegations: vec![delegation(10, 3), delegation(20, 4)]
            })
            .available_value(&cosmos, &balance(1, 0, 0, 0))
            .unwrap(),
            BigInt::from(7)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unstake { delegation: delegation(50, 0) })
                .available_value(&cosmos, &balance(1, 0, 0, 0))
                .unwrap(),
            BigInt::from(50)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unfreeze { resource: Resource::Energy })
                .available_value(&tron, &balance(1, 2, 3, 0))
                .unwrap(),
            BigInt::from(3)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unfreeze { resource: Resource::Bandwidth })
                .available_value(&tron, &balance(1, 2, 3, 0))
                .unwrap(),
            BigInt::from(2)
        );
    }

    #[test]
    fn test_limits_reserve_boundary() {
        let solana = asset(Chain::Solana);
        let config = get_stake_config(StakeChain::Solana);
        let reserve = config.reserved_for_fees;
        let minimum = config.min_amount;
        assert!(reserve > 0 && minimum > 0);

        let at_boundary = stake(GemAmountStakeType::Stake).limits(&solana, &balance(reserve + minimum, 0, 0, 0)).unwrap();
        assert!(!at_boundary.reserves_fee);
        assert_eq!(at_boundary.max_value, BigInt::from(reserve + minimum));

        let above_boundary = stake(GemAmountStakeType::Stake).limits(&solana, &balance(reserve + minimum + 1, 0, 0, 0)).unwrap();
        assert!(above_boundary.reserves_fee);
        assert_eq!(above_boundary.max_value, BigInt::from(minimum + 1));

        let below_reserve = stake(GemAmountStakeType::Stake).limits(&solana, &balance(reserve - 1, 0, 0, 0)).unwrap();
        assert!(!below_reserve.reserves_fee);
        assert_eq!(below_reserve.max_value, BigInt::from(reserve - 1));
        assert_eq!(below_reserve.available_value, BigInt::from(reserve - 1));
    }

    #[test]
    fn test_earn_perpetual_and_deposit_sources() {
        let ethereum = asset(Chain::Ethereum);
        assert_eq!(
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Deposit
            }
            .available_value(&ethereum, &balance(11, 0, 0, 0))
            .unwrap(),
            BigInt::from(11)
        );
        assert_eq!(
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Withdraw { delegation: delegation(33, 0) }
            }
            .available_value(&ethereum, &balance(11, 0, 0, 0))
            .unwrap(),
            BigInt::from(33)
        );
        assert_eq!(GemAmountType::Deposit.available_value(&usdc(), &balance(5, 0, 0, 0)).unwrap(), BigInt::from(5));
        assert_eq!(GemAmountType::Withdraw.available_value(&usdc(), &balance(5, 0, 0, 0)).unwrap(), BigInt::from(7));

        let perpetual = |leverage: u8, size_decimals: i32| GemAmountType::Perpetual {
            position: GemAmountPerpetualPosition::Open,
            price: 4.0,
            leverage,
            size_decimals,
        };
        assert_eq!(perpetual(1, 0).rules(&usdc()).minimum_value, BigInt::from(12_000_000));
        assert_eq!(perpetual(1, 1).rules(&usdc()).minimum_value, BigInt::from(10_000_000));
        assert_eq!(perpetual(2, 0).rules(&usdc()).minimum_value, BigInt::from(6_000_000));
        assert_eq!(perpetual(1, 0).available_value(&usdc(), &balance(9, 0, 0, 0)).unwrap(), BigInt::from(9));
    }

    #[test]
    fn test_stake_withdraw_has_no_minimum() {
        let withdraw = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Withdraw { delegation: delegation(700, 0) },
        };
        assert_eq!(withdraw.rules(&usdc()).minimum_value, BigInt::ZERO);
        assert!(!withdraw.rules(&usdc()).can_change_value);
    }

    #[test]
    fn test_transfer_deposit_withdraw_rules() {
        assert_eq!(GemAmountType::Transfer.rules(&asset(Chain::Ethereum)).minimum_value, BigInt::ZERO);
        assert_eq!(GemAmountType::Deposit.rules(&usdc()).minimum_value, BigInt::from(MIN_DEPOSIT_AMOUNT));
        assert_eq!(GemAmountType::Withdraw.rules(&usdc()).minimum_value, BigInt::from(MIN_WITHDRAW_AMOUNT));
        assert_eq!(GemAmountType::Deposit.rules(&asset(Chain::Ethereum)).minimum_value, BigInt::ZERO);
        assert_eq!(GemAmountType::Withdraw.available_value(&usdc(), &balance(1, 0, 0, 0)).unwrap(), BigInt::from(7));
        assert_eq!(
            GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Reduce { available: BigInt::from(42) },
                price: 1.0,
                leverage: 1,
                size_decimals: 0
            }
            .available_value(&usdc(), &balance(1, 0, 0, 0))
            .unwrap(),
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
        assert_eq!(validate(&BigInt::from(0), &BigInt::from(10), &BigInt::from(5)), Err(GemAmountError::Zero));
        assert_eq!(validate(&BigInt::from(-1), &BigInt::from(10), &BigInt::from(5)), Err(GemAmountError::Zero));
        assert_eq!(parse_value("abc"), Err(GemAmountError::InvalidValue { value: "abc".into() }));

        let stake = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Stake,
        };
        let bnb = asset(Chain::SmartChain);
        assert_eq!(
            stake.validate(&bnb, &balance(5_000_000_000_000_000_000, 0, 0, 0), "990000000000000000".to_string()),
            Err(GemAmountError::BelowMinimum {
                minimum: "1000000000000000000".into()
            })
        );
        assert_eq!(stake.validate(&bnb, &balance(5_000_000_000_000_000_000, 0, 0, 0), "1500000000000000000".to_string()), Ok(()));
        assert_eq!(
            GemAmountType::Transfer.validate(&bnb, &balance(10, 0, 0, 0), "11".to_string()),
            Err(GemAmountError::InsufficientBalance { available: "10".into() })
        );
    }

    #[test]
    fn test_transfer_balance_carries_big_integers_so_a_malformed_value_cannot_read_as_zero() {
        let _: fn(GemTransferBalance) -> (GemBigInt, GemBigInt, GemBigInt, GemBigInt) = |balance| (balance.available, balance.frozen, balance.locked, balance.withdrawable);
        assert_eq!(
            GemAmountType::Transfer.available_value(&asset(Chain::Ethereum), &balance(500, 0, 0, 0)).unwrap(),
            BigInt::from(500)
        );
    }
}
