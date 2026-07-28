use crate::{Asset, AssetId, AssetType, Chain, EarnType, PerpetualType, StakeType, TransactionInputType};
use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferAmount {
    pub value: BigInt,
    pub network_fee: BigInt,
    pub is_max_amount: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransferAmountError {
    InsufficientBalance { asset_id: AssetId, required: BigInt, available: BigInt },
    InsufficientNetworkFee { asset_id: AssetId, required: BigInt, available: BigInt },
    MinimumAccountBalanceTooLow { asset_id: AssetId, required: BigInt, available: BigInt },
}

impl std::fmt::Display for TransferAmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientBalance { asset_id, required, available } => {
                write!(f, "insufficient {} balance: required {}, available {}", asset_id, required, available)
            }
            Self::InsufficientNetworkFee { asset_id, required, available } => {
                write!(f, "insufficient {} network fee balance: required {}, available {}", asset_id, required, available)
            }
            Self::MinimumAccountBalanceTooLow { asset_id, required, available } => {
                write!(f, "{} account balance below minimum: required {}, remaining {}", asset_id, required, available)
            }
        }
    }
}

impl std::error::Error for TransferAmountError {}

impl TransferAmountError {
    fn insufficient_balance(asset_id: &AssetId, required: BigInt, available: BigInt) -> Self {
        Self::InsufficientBalance {
            asset_id: asset_id.clone(),
            required,
            available,
        }
    }

    fn insufficient_network_fee(asset_id: &AssetId, required: BigInt, available: BigInt) -> Self {
        Self::InsufficientNetworkFee {
            asset_id: asset_id.clone(),
            required,
            available,
        }
    }

    fn minimum_account_balance_too_low(asset_id: &AssetId, required: BigInt, available: BigInt) -> Self {
        Self::MinimumAccountBalanceTooLow {
            asset_id: asset_id.clone(),
            required,
            available,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransferAmountInput {
    pub input_type: TransactionInputType,
    pub value: BigInt,
    pub available_value: BigInt,
    pub fee_asset: Asset,
    pub fee_asset_balance: BigInt,
    pub fee: BigInt,
    pub is_max_amount: bool,
    pub minimum_value: Option<BigInt>,
}

impl TransactionInputType {
    pub fn spends_balance(&self) -> bool {
        match self {
            Self::Transfer(_) | Self::Deposit(_) | Self::Swap(_, _, _) | Self::Generic(_, _, _) => true,
            Self::Stake(_, stake_type) => match stake_type {
                StakeType::Stake(_) | StakeType::Freeze(_) => true,
                StakeType::Unstake(_) | StakeType::Unfreeze(_) | StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) => false,
            },
            Self::Earn(_, earn_type, _) => match earn_type {
                EarnType::Deposit(_) => true,
                EarnType::Withdraw(_) => false,
            },
            Self::Perpetual(_, perpetual_type) => match perpetual_type {
                PerpetualType::Open(_) | PerpetualType::Increase(_) => true,
                PerpetualType::Close(_) | PerpetualType::Modify(_) | PerpetualType::Reduce(_) => false,
            },
            Self::TokenApprove(_, _) | Self::Account(_, _) | Self::TransferNft(_, _) => false,
        }
    }
}

impl TransferAmountInput {
    pub fn calculate(&self) -> Result<TransferAmount, TransferAmountError> {
        let asset = self.input_type.get_asset();
        let spends_balance = self.input_type.spends_balance();
        let should_deduct_fee = spends_balance && asset.id == self.fee_asset.id;

        let value = match self.is_max_amount && should_deduct_fee {
            true => self.value.clone().min(&self.available_value - &self.fee),
            false => self.value.clone(),
        };
        let required = match (spends_balance, should_deduct_fee) {
            (false, _) => BigInt::ZERO,
            (true, false) => value.clone(),
            (true, true) => &value + &self.fee,
        };
        let should_skip_fee_check = asset.chain == Chain::HyperCore && !spends_balance;
        let minimum_account_balance = match asset.chain.minimum_account_balance() {
            Some(minimum) if asset.asset_type == AssetType::NATIVE && !self.is_max_amount && spends_balance => Some(BigInt::from(minimum)),
            _ => None,
        };
        let remaining_balance = &self.available_value - &required;

        if required > self.available_value {
            return match &minimum_account_balance {
                Some(minimum) if value <= self.available_value => Err(TransferAmountError::minimum_account_balance_too_low(&asset.id, minimum.clone(), remaining_balance)),
                _ => Err(TransferAmountError::insufficient_balance(&asset.id, required, self.available_value.clone())),
            };
        }

        if self.fee_asset_balance < self.fee && !should_skip_fee_check {
            return Err(TransferAmountError::insufficient_network_fee(
                &self.fee_asset.id,
                self.fee.clone(),
                self.fee_asset_balance.clone(),
            ));
        }

        if let Some(minimum) = &minimum_account_balance
            && remaining_balance < *minimum
        {
            return Err(TransferAmountError::minimum_account_balance_too_low(&asset.id, minimum.clone(), remaining_balance));
        }

        if let Some(minimum_value) = &self.minimum_value
            && value < *minimum_value
        {
            return Err(TransferAmountError::insufficient_balance(&asset.id, minimum_value.clone(), value));
        }

        Ok(TransferAmount {
            value,
            network_fee: self.fee.clone(),
            is_max_amount: self.is_max_amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountDataType, Delegation, DelegationValidator, PerpetualConfirmData, PerpetualDirection, Resource, nft::NFTAsset, swap::ApprovalData};

    const SOLANA_MINIMUM_ACCOUNT_BALANCE: u64 = 890_880;
    const FEE: u64 = 5_000;

    fn input(input_type: TransactionInputType, value: u64, available_value: u64, fee_asset_balance: u64) -> TransferAmountInput {
        let fee_asset = Asset::from_chain(input_type.get_asset().chain);
        TransferAmountInput {
            input_type,
            value: BigInt::from(value),
            available_value: BigInt::from(available_value),
            fee_asset,
            fee_asset_balance: BigInt::from(fee_asset_balance),
            fee: BigInt::from(FEE),
            is_max_amount: false,
            minimum_value: None,
        }
    }

    fn solana_transfer(value: u64, available_value: u64) -> TransferAmountInput {
        input(TransactionInputType::Transfer(Asset::mock_sol()), value, available_value, available_value)
    }

    #[test]
    fn test_spends_balance() {
        let asset = Asset::mock_sol();

        let spending = [
            TransactionInputType::Transfer(asset.clone()),
            TransactionInputType::Deposit(asset.clone()),
            TransactionInputType::Stake(asset.clone(), StakeType::Stake(DelegationValidator::mock())),
            TransactionInputType::Stake(asset.clone(), StakeType::Freeze(Resource::Bandwidth)),
            TransactionInputType::Perpetual(asset.clone(), PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None))),
        ];
        for input_type in spending {
            assert!(input_type.spends_balance(), "{:?} must spend the sender balance", input_type.transaction_type());
        }

        let non_spending = [
            TransactionInputType::Stake(asset.clone(), StakeType::Unstake(Delegation::mock())),
            TransactionInputType::Stake(asset.clone(), StakeType::Withdraw(Delegation::mock())),
            TransactionInputType::Stake(asset.clone(), StakeType::Rewards(vec![DelegationValidator::mock()])),
            TransactionInputType::Stake(asset.clone(), StakeType::Unfreeze(Resource::Bandwidth)),
            TransactionInputType::TokenApprove(Asset::mock_spl_token(), ApprovalData::mock()),
            TransactionInputType::Account(asset.clone(), AccountDataType::Activate),
            TransactionInputType::TransferNft(asset.clone(), NFTAsset::mock()),
            TransactionInputType::Perpetual(asset, PerpetualType::Close(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None))),
        ];
        for input_type in non_spending {
            assert!(!input_type.spends_balance(), "{:?} must not spend the sender balance", input_type.transaction_type());
        }
    }

    #[test]
    fn test_calculate_spending() {
        let ok = solana_transfer(10_000_000, 100_000_000).calculate().unwrap();
        assert_eq!(ok.value, BigInt::from(10_000_000));
        assert_eq!(ok.network_fee, BigInt::from(FEE));
        assert!(!ok.is_max_amount);

        assert_eq!(
            solana_transfer(20_000_000, 10_000_000).calculate().unwrap_err(),
            TransferAmountError::InsufficientBalance {
                asset_id: Asset::mock_sol().id,
                required: BigInt::from(20_005_000u64),
                available: BigInt::from(10_000_000u64),
            },
            "sending more than the balance is insufficient balance, not a reserve problem"
        );

        let full_balance_payment = input(TransactionInputType::Transfer(Asset::mock_btc()), 100_000_000, 100_000_000, 100_000_000);
        assert_eq!(
            full_balance_payment.calculate().unwrap_err(),
            TransferAmountError::InsufficientBalance {
                asset_id: Asset::mock_btc().id,
                required: BigInt::from(100_005_000u64),
                available: BigInt::from(100_000_000u64),
            },
            "a fixed amount equal to the whole balance is not treated as max, so the fee on top makes it insufficient"
        );

        let mut insufficient_fee = solana_transfer(10_000_000, 100_000_000);
        insufficient_fee.fee_asset_balance = BigInt::from(1_000);
        assert_eq!(
            insufficient_fee.calculate().unwrap_err(),
            TransferAmountError::InsufficientNetworkFee {
                asset_id: Asset::mock_sol().id,
                required: BigInt::from(FEE),
                available: BigInt::from(1_000),
            }
        );

        let mut below_minimum = solana_transfer(100, 100_000_000);
        below_minimum.minimum_value = Some(BigInt::from(200));
        assert_eq!(
            below_minimum.calculate().unwrap_err(),
            TransferAmountError::InsufficientBalance {
                asset_id: Asset::mock_sol().id,
                required: BigInt::from(200),
                available: BigInt::from(100),
            }
        );
    }

    #[test]
    fn test_calculate_zero_value_still_spends() {
        let below_minimum = solana_transfer(0, SOLANA_MINIMUM_ACCOUNT_BALANCE);
        assert_eq!(
            below_minimum.calculate().unwrap_err(),
            TransferAmountError::MinimumAccountBalanceTooLow {
                asset_id: Asset::mock_sol().id,
                required: BigInt::from(SOLANA_MINIMUM_ACCOUNT_BALANCE),
                available: BigInt::from(SOLANA_MINIMUM_ACCOUNT_BALANCE - FEE),
            },
            "a zero value transfer still spends the fee, so the minimum balance rule applies"
        );

        let hypercore = Asset::from_chain(Chain::HyperCore);
        let mut zero_transfer = input(TransactionInputType::Transfer(hypercore.clone()), 0, 1_000_000, 0);
        zero_transfer.fee_asset = hypercore;
        assert_eq!(
            zero_transfer.calculate().unwrap_err(),
            TransferAmountError::InsufficientNetworkFee {
                asset_id: Asset::from_chain(Chain::HyperCore).id,
                required: BigInt::from(FEE),
                available: BigInt::ZERO,
            },
            "the HyperCore fee exemption is for non-spending types, not for zero amounts"
        );
    }

    #[test]
    fn test_calculate_minimum_account_balance() {
        assert_eq!(
            solana_transfer(999_000, 1_000_000).calculate().unwrap_err(),
            TransferAmountError::MinimumAccountBalanceTooLow {
                asset_id: Asset::mock_sol().id,
                required: BigInt::from(SOLANA_MINIMUM_ACCOUNT_BALANCE),
                available: BigInt::from(-4_000),
            },
            "sending almost the whole balance is a reserve problem, not insufficient balance"
        );

        assert_eq!(
            input(TransactionInputType::Transfer(Asset::mock()), 999_000, 1_000_000, 1_000_000).calculate().unwrap_err(),
            TransferAmountError::InsufficientBalance {
                asset_id: Asset::mock().id,
                required: BigInt::from(1_004_000u64),
                available: BigInt::from(1_000_000u64),
            },
            "a chain without a rent-exempt minimum reports insufficient balance, not a reserve problem"
        );

        let exactly_minimum = 10_000_000 + FEE + SOLANA_MINIMUM_ACCOUNT_BALANCE;
        assert!(
            solana_transfer(10_000_000, exactly_minimum).calculate().is_ok(),
            "leaving exactly the rent-exempt minimum keeps the account, so it is allowed"
        );

        assert_eq!(
            solana_transfer(10_000_000, exactly_minimum - 1).calculate().unwrap_err(),
            TransferAmountError::MinimumAccountBalanceTooLow {
                asset_id: Asset::mock_sol().id,
                required: BigInt::from(SOLANA_MINIMUM_ACCOUNT_BALANCE),
                available: BigInt::from(SOLANA_MINIMUM_ACCOUNT_BALANCE - 1),
            }
        );

        let mut max = solana_transfer(1_000_000_000, 1_000_000_000);
        max.is_max_amount = true;
        let result = max.calculate().unwrap();
        assert_eq!(result.value, BigInt::from(1_000_000_000 - FEE));
        assert!(result.is_max_amount);

        const RESERVED_FOR_FEES: u64 = 5_000_000;
        let mut max_with_reserve = input(
            TransactionInputType::Stake(Asset::mock_sol(), StakeType::Stake(DelegationValidator::mock())),
            1_000_000_000 - RESERVED_FOR_FEES,
            1_000_000_000,
            1_000_000_000,
        );
        max_with_reserve.is_max_amount = true;
        assert_eq!(
            max_with_reserve.calculate().unwrap().value,
            BigInt::from(1_000_000_000 - RESERVED_FOR_FEES),
            "the amount screen already reserved for fees, core must not spend that reserve"
        );

        let token = input(TransactionInputType::Transfer(Asset::mock_spl_token()), 10_000_000, 10_000_000, 10_000);
        assert!(token.calculate().is_ok());
    }

    #[test]
    fn test_calculate_non_spending() {
        let unstake = input(
            TransactionInputType::Stake(Asset::mock_sol(), StakeType::Unstake(Delegation::mock())),
            649_953_059,
            649_953_059,
            5_000_000,
        );
        let result = unstake.calculate().unwrap();
        assert_eq!(result.value, BigInt::from(649_953_059u64));

        let mut unstake_without_fee = unstake;
        unstake_without_fee.fee_asset_balance = BigInt::from(1_000);
        assert_eq!(
            unstake_without_fee.calculate().unwrap_err(),
            TransferAmountError::InsufficientNetworkFee {
                asset_id: Asset::mock_sol().id,
                required: BigInt::from(FEE),
                available: BigInt::from(1_000),
            }
        );

        let below_reserve_unstake = input(TransactionInputType::Stake(Asset::mock_sol(), StakeType::Unstake(Delegation::mock())), 100, 100, 5_000_000);
        assert!(
            below_reserve_unstake.calculate().is_ok(),
            "unstaking never spends the balance, so a delegation below the reserve is still allowed"
        );

        let approve = input(TransactionInputType::TokenApprove(Asset::mock_spl_token(), ApprovalData::mock()), 0, 0, 5_000_000);
        assert!(approve.calculate().is_ok());

        let activate = input(TransactionInputType::Account(Asset::mock_spl_token(), AccountDataType::Activate), 0, 0, 5_000_000);
        assert!(activate.calculate().is_ok());
    }

    #[test]
    fn test_calculate_hypercore_fee_check() {
        let asset = Asset::from_chain(Chain::HyperCore);
        let close = input(
            TransactionInputType::Perpetual(asset.clone(), PerpetualType::Close(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None))),
            100,
            100,
            0,
        );
        assert!(close.calculate().is_ok());

        let open = input(
            TransactionInputType::Perpetual(asset, PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None))),
            100,
            1_000_000,
            0,
        );
        assert_eq!(
            open.calculate().unwrap_err(),
            TransferAmountError::InsufficientNetworkFee {
                asset_id: Asset::from_chain(Chain::HyperCore).id,
                required: BigInt::from(FEE),
                available: BigInt::ZERO,
            }
        );
    }
}
