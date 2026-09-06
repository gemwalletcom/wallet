use std::str::FromStr;

use num_bigint::{BigInt, BigUint};
use primitives::{Asset, AutocloseEstimator, Chain, Delegation, EarnType, PerpetualDirection, StakeChain, StakeType, TpslType};

use super::model::{GemAmountEarnType, GemAmountError, GemAmountInput, GemAmountPerpetualPosition, GemAmountStakeType, GemAmountTransfer, GemAmountType, GemPerpetualAutoclose};
use crate::config::perpetual_config::{MIN_DEPOSIT_AMOUNT, MIN_WITHDRAW_AMOUNT};
use crate::config::stake::get_stake_config;
use crate::models::custom_types::GemBigInt;
use crate::perpetual::GemPerpetual;
use crate::services::balance::{GemAssetBalance, GemBalanceRequirement};
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualPositionAction;
use crate::services::transfer::rules as transfer_rules;
use crate::services::transfer::{GemRecipient, GemTransferData};
use gem_hypercore::perpetual_formatter::PerpetualFormatter;
use primitives::PerpetualProvider;
use primitives::TransactionInputType;

const USDC_SYMBOL: &str = "USDC";

#[uniffi::export]
impl GemAmountType {
    pub fn input(&self, asset: &Asset, balance: &GemAssetBalance) -> GemAmountInput {
        let available = self.available_value(asset, balance);
        let reserve = reserve_for_fee(self, asset);
        let max_after_fee = (&available - &reserve).max(BigInt::from(0));
        let reserved_fee = reserves_fee(self, &reserve, &max_after_fee, &minimum_value(self, asset)).then_some(reserve);
        GemAmountInput {
            available_value: available.clone(),
            max_value: if reserved_fee.is_some() { max_after_fee } else { available },
            reserved_fee,
            can_change_value: can_change_value(self, asset),
            shows_asset_balance: shows_asset_balance(self, asset),
        }
    }

    pub fn validate(&self, asset: &Asset, balance: &GemAssetBalance, value: GemBigInt) -> Result<(), GemAmountError> {
        validate(&value, &self.available_value(asset, balance), &minimum_value(self, asset))
    }

    pub fn can_switch_input_type(&self) -> bool {
        matches!(self, Self::Transfer)
    }
}

pub fn perpetual_amount_type(action: &GemPerpetualPositionAction, leverage: u8) -> GemAmountType {
    let data = action.data();
    GemAmountType::Perpetual {
        position: match action {
            GemPerpetualPositionAction::Open { .. } => GemAmountPerpetualPosition::Open,
            GemPerpetualPositionAction::Increase { .. } => GemAmountPerpetualPosition::Increase,
            GemPerpetualPositionAction::Reduce { available, .. } => GemAmountPerpetualPosition::Reduce { available: available.clone() },
        },
        direction: data.direction.clone(),
        price: data.price,
        leverage,
        size_decimals: data.asset.decimals,
    }
}

pub fn stake_amount_type(stake_type: StakeType, delegations: Vec<Delegation>) -> GemAmountType {
    let stake_type = match stake_type {
        StakeType::Stake(_) => GemAmountStakeType::Stake,
        StakeType::Unstake(delegation) => GemAmountStakeType::Unstake { delegation },
        StakeType::Redelegate(data) => GemAmountStakeType::Redelegate { delegation: data.delegation },
        StakeType::Withdraw(delegation) => GemAmountStakeType::Withdraw { delegation },
        StakeType::Rewards(validators) => GemAmountStakeType::Rewards {
            delegations: delegations
                .into_iter()
                .filter(|delegation| validators.iter().any(|validator| validator.id == delegation.validator.id))
                .collect(),
        },
        StakeType::Freeze(resource) => GemAmountStakeType::Freeze { resource },
        StakeType::Unfreeze(resource) => GemAmountStakeType::Unfreeze { resource },
    };
    GemAmountType::Stake { stake_type }
}

pub fn earn_amount_type(earn_type: EarnType) -> GemAmountType {
    GemAmountType::Earn {
        earn_type: match earn_type {
            EarnType::Deposit(_) => GemAmountEarnType::Deposit,
            EarnType::Withdraw(delegation) => GemAmountEarnType::Withdraw { delegation },
        },
    }
}

pub fn transfer_data(asset: Asset, transfer: GemAmountTransfer, owner: Option<GemRecipient>, value: GemBigInt, use_max_amount: bool) -> Result<GemTransferData, GemServiceError> {
    let (input_type, recipient) = match transfer {
        GemAmountTransfer::Send { recipient } => (TransactionInputType::Transfer { asset }, recipient),
        GemAmountTransfer::Deposit => (TransactionInputType::Deposit { asset }, GemPerpetual::new(PerpetualProvider::Hypercore).deposit_recipient()),
        GemAmountTransfer::Withdraw => {
            let owner = owner.ok_or_else(|| GemServiceError::NotFound {
                msg: format!("no {} account to withdraw to", asset.chain()),
            })?;
            (TransactionInputType::Withdrawal { asset }, owner)
        }
    };
    Ok(GemTransferData {
        input_type,
        recipient,
        value,
        use_max_amount,
        minimum_value: None,
    })
}

impl GemAmountType {
    fn available_value(&self, asset: &Asset, balance: &GemAssetBalance) -> BigInt {
        match self {
            Self::Transfer | Self::Deposit => BigInt::from(balance.available.clone()),
            Self::Withdraw => BigInt::from(balance.withdrawable.clone()),
            Self::Stake { stake_type } => match stake_type {
                GemAmountStakeType::Stake if asset.chain() == Chain::Tron => transfer_rules::tron_stake_available(asset, balance),
                GemAmountStakeType::Stake | GemAmountStakeType::Freeze { .. } => BigInt::from(balance.available.clone()),
                GemAmountStakeType::Unstake { delegation } | GemAmountStakeType::Redelegate { delegation } | GemAmountStakeType::Withdraw { delegation } => {
                    BigInt::from(delegation.base.balance.clone())
                }
                GemAmountStakeType::Rewards { delegations } => BigInt::from(delegations.iter().map(|delegation| delegation.base.rewards.clone()).sum::<BigUint>()),
                GemAmountStakeType::Unfreeze { resource } => transfer_rules::unfreeze_available(resource, balance),
            },
            Self::Earn { earn_type } => match earn_type {
                GemAmountEarnType::Deposit => BigInt::from(balance.available.clone()),
                GemAmountEarnType::Withdraw { delegation } => BigInt::from(delegation.base.balance.clone()),
            },
            Self::Perpetual { position, .. } => match position {
                GemAmountPerpetualPosition::Open | GemAmountPerpetualPosition::Increase => BigInt::from(balance.available.clone()),
                GemAmountPerpetualPosition::Reduce { available } => BigInt::from(available.clone()),
            },
        }
    }
}

pub fn perpetual_autoclose(price: f64, direction: PerpetualDirection, leverage: u8, take_profit_percent: u8, stop_loss_percent: u8) -> GemPerpetualAutoclose {
    let estimator = AutocloseEstimator::for_open(price, 0.0, leverage, direction);
    let target = |percent: u8, trigger_type: TpslType| (percent > 0).then(|| estimator.target_price_from_roe(i32::from(percent), trigger_type));
    GemPerpetualAutoclose {
        take_profit: target(take_profit_percent, TpslType::TakeProfit),
        stop_loss: target(stop_loss_percent, TpslType::StopLoss),
    }
}

pub fn validate(value: &BigInt, available: &BigInt, minimum: &BigInt) -> Result<(), GemAmountError> {
    if value <= &BigInt::from(0) {
        return Err(GemAmountError::Zero);
    }
    if value < minimum {
        return Err(GemAmountError::BelowMinimum { minimum: minimum.clone() });
    }
    if value > available {
        return Err(GemAmountError::InsufficientBalance {
            requirement: GemBalanceRequirement::new(value.clone(), available.clone()),
        });
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
    use crate::config::perpetual_config::HYPERLIQUID_DEPOSIT_ADDRESS;
    use crate::models::custom_types::GemBigUint;
    use primitives::Resource;
    use primitives::asset_balance::BalanceMetadata;
    use primitives::{AssetId, AssetType, Delegation, DelegationBase, DelegationState, DelegationValidator, StakeProviderType};

    fn asset(chain: Chain) -> Asset {
        Asset::from_chain(chain)
    }

    fn usdc() -> Asset {
        Asset::new(AssetId::from(Chain::HyperCore, Some("usdc".into())), "USDC".into(), "USDC".into(), 6, AssetType::TOKEN)
    }

    fn balance(available: u64, frozen: u64, locked: u64, votes: u32) -> GemAssetBalance {
        GemAssetBalance {
            available: BigUint::from(available),
            frozen: BigUint::from(frozen),
            locked: BigUint::from(locked),
            withdrawable: BigUint::from(7u32),
            metadata: Some(BalanceMetadata {
                votes,
                ..BalanceMetadata::default()
            }),
            ..GemAssetBalance::mock()
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
        assert_eq!(minimum_value(&stake(GemAmountStakeType::Stake), &cosmos), BigInt::from(config.min_amount));

        let stake_input = stake(GemAmountStakeType::Stake).input(&cosmos, &balance(config.reserved_for_fees * 10 + config.min_amount * 10, 0, 0, 0));
        assert_eq!(stake_input.reserved_fee, Some(BigInt::from(config.reserved_for_fees)));
        assert!(stake_input.can_change_value);
        assert_eq!(stake_input.max_value, BigInt::from(config.reserved_for_fees * 9 + config.min_amount * 10));

        let tron = asset(Chain::Tron);
        let tron_config = get_stake_config(StakeChain::Tron);
        assert_eq!(reserve_for_fee(&stake(GemAmountStakeType::Stake), &tron), BigInt::ZERO);
        assert_eq!(
            stake(GemAmountStakeType::Stake).available_value(&tron, &balance(1, 5_000_000, 3_000_000, 2)),
            BigInt::from(6_000_000)
        );
        let freeze = stake(GemAmountStakeType::Freeze { resource: Resource::Bandwidth });
        assert!(tron_config.reserved_for_fees > 0);
        let freeze_input = freeze.input(&tron, &balance(tron_config.reserved_for_fees + tron_config.min_amount + 1, 99, 98, 0));
        assert_eq!(freeze_input.available_value, BigInt::from(tron_config.reserved_for_fees + tron_config.min_amount + 1));
        assert_eq!(freeze_input.reserved_fee, Some(BigInt::from(tron_config.reserved_for_fees)));
        assert_eq!(freeze_input.max_value, BigInt::from(tron_config.min_amount + 1));

        let smart_chain = asset(Chain::SmartChain);
        let redelegate = stake(GemAmountStakeType::Redelegate { delegation: delegation(50, 0) });
        let smart_chain_minimum = get_stake_config(StakeChain::SmartChain).min_amount;
        assert!(smart_chain_minimum > 0);
        assert_eq!(minimum_value(&redelegate, &smart_chain), BigInt::from(smart_chain_minimum));
        assert_eq!(minimum_value(&redelegate, &cosmos), BigInt::ZERO);

        let unstake = stake(GemAmountStakeType::Unstake { delegation: delegation(50, 0) });
        let solana_unstake = unstake.input(&asset(Chain::Solana), &balance(1, 0, 0, 0));
        assert!(!solana_unstake.can_change_value);
        assert!(!solana_unstake.shows_asset_balance);
        let cosmos_unstake = unstake.input(&cosmos, &balance(1, 0, 0, 0));
        assert!(cosmos_unstake.can_change_value);
        assert!(cosmos_unstake.shows_asset_balance);

        let rewards = stake(GemAmountStakeType::Rewards { delegations: vec![] }).input(&cosmos, &balance(1, 0, 0, 0));
        assert!(!rewards.can_change_value);
        assert!(rewards.shows_asset_balance);
        assert_eq!(
            stake(GemAmountStakeType::Rewards {
                delegations: vec![delegation(10, 3), delegation(20, 4)]
            })
            .available_value(&cosmos, &balance(1, 0, 0, 0)),
            BigInt::from(7)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unstake { delegation: delegation(50, 0) }).available_value(&cosmos, &balance(1, 0, 0, 0)),
            BigInt::from(50)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unfreeze { resource: Resource::Energy }).available_value(&tron, &balance(1, 2, 3, 0)),
            BigInt::from(3)
        );
        assert_eq!(
            stake(GemAmountStakeType::Unfreeze { resource: Resource::Bandwidth }).available_value(&tron, &balance(1, 2, 3, 0)),
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

        let at_boundary = stake(GemAmountStakeType::Stake).input(&solana, &balance(reserve + minimum, 0, 0, 0));
        assert_eq!(at_boundary.reserved_fee, None);
        assert_eq!(at_boundary.max_value, BigInt::from(reserve + minimum));

        let above_boundary = stake(GemAmountStakeType::Stake).input(&solana, &balance(reserve + minimum + 1, 0, 0, 0));
        assert_eq!(above_boundary.reserved_fee, Some(BigInt::from(reserve)));
        assert_eq!(above_boundary.max_value, BigInt::from(minimum + 1));

        let below_reserve = stake(GemAmountStakeType::Stake).input(&solana, &balance(reserve - 1, 0, 0, 0));
        assert_eq!(below_reserve.reserved_fee, None);
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
            .available_value(&ethereum, &balance(11, 0, 0, 0)),
            BigInt::from(11)
        );
        assert_eq!(
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Withdraw { delegation: delegation(33, 0) }
            }
            .available_value(&ethereum, &balance(11, 0, 0, 0)),
            BigInt::from(33)
        );
        assert_eq!(GemAmountType::Deposit.available_value(&usdc(), &balance(5, 0, 0, 0)), BigInt::from(5));
        assert_eq!(GemAmountType::Withdraw.available_value(&usdc(), &balance(5, 0, 0, 0)), BigInt::from(7));

        let perpetual = |leverage: u8, size_decimals: i32| GemAmountType::Perpetual {
            position: GemAmountPerpetualPosition::Open,
            direction: PerpetualDirection::Long,
            price: 4.0,
            leverage,
            size_decimals,
        };
        assert_eq!(minimum_value(&perpetual(1, 0), &usdc()), BigInt::from(12_000_000));
        assert_eq!(minimum_value(&perpetual(1, 1), &usdc()), BigInt::from(10_000_000));
        assert_eq!(minimum_value(&perpetual(2, 0), &usdc()), BigInt::from(6_000_000));
        assert_eq!(perpetual(1, 0).available_value(&usdc(), &balance(9, 0, 0, 0)), BigInt::from(9));
    }

    #[test]
    fn test_stake_withdraw_has_no_minimum() {
        let withdraw = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Withdraw { delegation: delegation(700, 0) },
        };
        assert_eq!(minimum_value(&withdraw, &usdc()), BigInt::ZERO);
        assert!(!withdraw.input(&usdc(), &balance(1, 0, 0, 0)).can_change_value);
    }

    #[test]
    fn test_transfer_deposit_withdraw_rules() {
        assert_eq!(minimum_value(&GemAmountType::Transfer, &asset(Chain::Ethereum)), BigInt::ZERO);
        assert_eq!(minimum_value(&GemAmountType::Deposit, &usdc()), BigInt::from(MIN_DEPOSIT_AMOUNT));
        assert_eq!(minimum_value(&GemAmountType::Withdraw, &usdc()), BigInt::from(MIN_WITHDRAW_AMOUNT));
        assert_eq!(minimum_value(&GemAmountType::Deposit, &asset(Chain::Ethereum)), BigInt::ZERO);
        assert_eq!(GemAmountType::Withdraw.available_value(&usdc(), &balance(1, 0, 0, 0)), BigInt::from(7));
        assert_eq!(
            GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Reduce { available: BigUint::from(42u32) },
                direction: PerpetualDirection::Long,
                price: 1.0,
                leverage: 1,
                size_decimals: 0
            }
            .available_value(&usdc(), &balance(1, 0, 0, 0)),
            BigInt::from(42)
        );
    }

    #[test]
    fn test_perpetual_autoclose_follows_the_preference_percents() {
        let long = perpetual_autoclose(100.0, PerpetualDirection::Long, 10, 50, 20);
        assert_eq!(long.take_profit, Some(105.0));
        assert_eq!(long.stop_loss, Some(98.0));

        let short = perpetual_autoclose(100.0, PerpetualDirection::Short, 10, 50, 20);
        assert_eq!(short.take_profit, Some(95.0));
        assert_eq!(short.stop_loss, Some(102.0));

        let off = perpetual_autoclose(100.0, PerpetualDirection::Long, 10, 0, 20);
        assert_eq!(off.take_profit, None);
        assert_eq!(off.stop_loss, Some(98.0));
        assert_eq!(
            perpetual_autoclose(100.0, PerpetualDirection::Long, 10, 0, 0),
            GemPerpetualAutoclose {
                take_profit: None,
                stop_loss: None
            }
        );
    }

    #[test]
    fn test_validate() {
        assert_eq!(validate(&BigInt::from(0), &BigInt::from(10), &BigInt::from(0)), Err(GemAmountError::Zero));
        assert_eq!(
            validate(&BigInt::from(1), &BigInt::from(10), &BigInt::from(2)),
            Err(GemAmountError::BelowMinimum { minimum: BigInt::from(2) })
        );
        assert_eq!(
            validate(&BigInt::from(11), &BigInt::from(10), &BigInt::from(0)),
            Err(GemAmountError::InsufficientBalance {
                requirement: GemBalanceRequirement::new(BigInt::from(11), BigInt::from(10))
            })
        );
        assert_eq!(validate(&BigInt::from(5), &BigInt::from(10), &BigInt::from(2)), Ok(()));
        assert_eq!(validate(&BigInt::from(0), &BigInt::from(10), &BigInt::from(5)), Err(GemAmountError::Zero));
        assert_eq!(validate(&BigInt::from(-1), &BigInt::from(10), &BigInt::from(5)), Err(GemAmountError::Zero));

        let stake = GemAmountType::Stake {
            stake_type: GemAmountStakeType::Stake,
        };
        let bnb = asset(Chain::SmartChain);
        assert_eq!(
            stake.validate(&bnb, &balance(5_000_000_000_000_000_000, 0, 0, 0), BigInt::from(990_000_000_000_000_000u64)),
            Err(GemAmountError::BelowMinimum {
                minimum: BigInt::from(1_000_000_000_000_000_000u64)
            })
        );
        assert_eq!(
            stake.validate(&bnb, &balance(5_000_000_000_000_000_000, 0, 0, 0), BigInt::from(1_500_000_000_000_000_000u64)),
            Ok(())
        );
        assert_eq!(
            GemAmountType::Transfer.validate(&bnb, &balance(10, 0, 0, 0), BigInt::from(11)),
            Err(GemAmountError::InsufficientBalance {
                requirement: GemBalanceRequirement::new(BigInt::from(11), BigInt::from(10))
            })
        );
    }

    #[test]
    fn test_transfer_balance_carries_big_integers_so_a_malformed_value_cannot_read_as_zero() {
        let _: fn(GemAssetBalance) -> (GemBigUint, GemBigUint, GemBigUint, GemBigUint) = |balance| (balance.available, balance.frozen, balance.locked, balance.withdrawable);
        assert_eq!(GemAmountType::Transfer.available_value(&asset(Chain::Ethereum), &balance(500, 0, 0, 0)), BigInt::from(500));
    }

    #[test]
    fn test_only_a_transfer_switches_the_input_type() {
        assert!(GemAmountType::Transfer.can_switch_input_type());
        assert!(!GemAmountType::Deposit.can_switch_input_type());
        assert!(!GemAmountType::Withdraw.can_switch_input_type());
        assert!(!stake(GemAmountStakeType::Stake).can_switch_input_type());
    }

    #[test]
    fn test_perpetual_amount_type_reads_the_position_from_the_action() {
        let data = crate::services::perpetual::GemPerpetualTransferData {
            provider: PerpetualProvider::Hypercore,
            direction: PerpetualDirection::Short,
            asset: usdc(),
            base_asset: usdc(),
            asset_index: 1,
            price: 120.5,
            leverage: 3,
            margin_type: primitives::PerpetualMarginType::Cross,
        };

        let open = perpetual_amount_type(&GemPerpetualPositionAction::Open { data: data.clone() }, 10);
        assert_eq!(
            open,
            GemAmountType::Perpetual {
                position: GemAmountPerpetualPosition::Open,
                direction: PerpetualDirection::Short,
                price: 120.5,
                leverage: 10,
                size_decimals: 6,
            }
        );
        let reduce = perpetual_amount_type(
            &GemPerpetualPositionAction::Reduce {
                data,
                available: GemBigUint::from(1_000u32),
            },
            3,
        );
        assert!(
            matches!(reduce, GemAmountType::Perpetual { position: GemAmountPerpetualPosition::Reduce { available }, leverage: 3, .. } if available == GemBigUint::from(1_000u32))
        );
    }

    #[test]
    fn test_stake_amount_type_mirrors_the_stake_type_and_keeps_only_the_rewarded_delegations() {
        let delegation = delegation(100, 5);
        let other = Delegation {
            base: DelegationBase {
                validator_id: "other".into(),
                ..delegation.base.clone()
            },
            validator: DelegationValidator {
                id: "other".into(),
                ..delegation.validator.clone()
            },
            price: None,
        };

        assert_eq!(stake_amount_type(StakeType::Stake(delegation.validator.clone()), vec![]), stake(GemAmountStakeType::Stake));
        assert_eq!(
            stake_amount_type(StakeType::Unstake(delegation.clone()), vec![]),
            stake(GemAmountStakeType::Unstake { delegation: delegation.clone() })
        );
        assert_eq!(
            stake_amount_type(
                StakeType::Redelegate(primitives::RedelegateData {
                    delegation: delegation.clone(),
                    to_validator: other.validator.clone(),
                }),
                vec![]
            ),
            stake(GemAmountStakeType::Redelegate { delegation: delegation.clone() })
        );
        assert_eq!(
            stake_amount_type(StakeType::Rewards(vec![delegation.validator.clone()]), vec![other, delegation.clone()]),
            stake(GemAmountStakeType::Rewards { delegations: vec![delegation] })
        );
        assert_eq!(
            stake_amount_type(StakeType::Freeze(Resource::Energy), vec![]),
            stake(GemAmountStakeType::Freeze { resource: Resource::Energy })
        );
    }

    #[test]
    fn test_earn_amount_type_keeps_the_withdrawn_delegation() {
        let delegation = delegation(100, 0);

        assert_eq!(
            earn_amount_type(EarnType::Deposit(delegation.validator.clone())),
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Deposit
            }
        );
        assert_eq!(
            earn_amount_type(EarnType::Withdraw(delegation.clone())),
            GemAmountType::Earn {
                earn_type: GemAmountEarnType::Withdraw { delegation }
            }
        );
    }

    #[test]
    fn test_transfer_data_addresses_a_send_a_deposit_and_a_withdrawal() {
        let recipient = GemRecipient::named("to".into(), "friend".into());
        let owner = GemRecipient::named("owner".into(), "wallet".into());

        let send = transfer_data(usdc(), GemAmountTransfer::Send { recipient: recipient.clone() }, None, GemBigInt::from(1), false).unwrap();
        assert!(matches!(send.input_type, TransactionInputType::Transfer { .. }));
        assert_eq!(send.recipient, recipient);

        let deposit = transfer_data(usdc(), GemAmountTransfer::Deposit, None, GemBigInt::from(2), true).unwrap();
        assert!(matches!(deposit.input_type, TransactionInputType::Deposit { .. }));
        assert_eq!(deposit.recipient.address, HYPERLIQUID_DEPOSIT_ADDRESS);
        assert!(deposit.use_max_amount);

        let withdraw = transfer_data(usdc(), GemAmountTransfer::Withdraw, Some(owner.clone()), GemBigInt::from(3), false).unwrap();
        assert!(matches!(withdraw.input_type, TransactionInputType::Withdrawal { .. }));
        assert_eq!(withdraw.recipient, owner);

        assert!(transfer_data(usdc(), GemAmountTransfer::Withdraw, None, GemBigInt::from(3), false).is_err());
    }
}
