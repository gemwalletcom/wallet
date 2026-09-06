use crate::{AssetId, AssetType, Chain, EarnType, StakeType, TransactionInputType};
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
    pub fee_asset: AssetId,
    pub fee_asset_balance: BigInt,
    pub fee: BigInt,
    pub is_max_amount: bool,
    pub minimum_value: Option<BigInt>,
}

impl TransactionInputType {
    pub fn spends_balance(&self) -> bool {
        match self {
            Self::Transfer { .. } | Self::Withdrawal { .. } | Self::Deposit { .. } | Self::Swap { .. } | Self::Generic { .. } => true,
            Self::Stake { stake_type, .. } => match stake_type {
                StakeType::Stake(_) | StakeType::Freeze(_) => true,
                StakeType::Unstake(_) | StakeType::Unfreeze(_) | StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) => false,
            },
            Self::Earn { earn_type, .. } => match earn_type {
                EarnType::Deposit(_) => true,
                EarnType::Withdraw(_) => false,
            },
            Self::Perpetual { .. } | Self::TokenApprove { .. } | Self::Account { .. } | Self::TransferNft { .. } => false,
        }
    }

    pub fn has_fixed_value(&self) -> bool {
        match self {
            Self::Swap { swap_data, .. } => swap_data.data.data_type.has_fixed_value(),
            _ => false,
        }
    }
}

impl TransferAmountInput {
    pub fn calculate(&self) -> Result<TransferAmount, TransferAmountError> {
        let asset = self.input_type.get_asset();
        let spends_balance = self.input_type.spends_balance();
        let should_deduct_fee = spends_balance && asset.id == self.fee_asset;

        let value = match self.is_max_amount && should_deduct_fee && !self.input_type.has_fixed_value() {
            true => self.value.clone().min(&self.available_value - &self.fee),
            false => self.value.clone(),
        };
        let required = match (spends_balance, should_deduct_fee) {
            (false, _) => BigInt::ZERO,
            (true, false) => value.clone(),
            (true, true) => &value + &self.fee,
        };
        let should_skip_fee_check = asset.chain() == Chain::HyperCore && !spends_balance;
        let minimum_account_balance = match asset.chain().minimum_account_balance() {
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
                &self.fee_asset,
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
    use crate::{
        AccountDataType, Asset, Delegation, DelegationValidator, PerpetualConfirmData, PerpetualDirection, PerpetualType, Resource, SwapProvider,
        nft::NFTAsset,
        swap::{ApprovalData, SwapData},
    };

    const SOLANA_MINIMUM_ACCOUNT_BALANCE: u64 = 890_880;
    const FEE: u64 = 5_000;

    fn input(input_type: TransactionInputType, value: u64, available_value: u64, fee_asset_balance: u64) -> TransferAmountInput {
        let fee_asset = AssetId::from_chain(input_type.get_asset().chain());
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
        input(TransactionInputType::Transfer { asset: Asset::mock_sol() }, value, available_value, available_value)
    }

    #[test]
    fn test_spends_balance() {
        let asset = Asset::mock_sol();

        let spending = [
            TransactionInputType::Transfer { asset: asset.clone() },
            TransactionInputType::Deposit { asset: asset.clone() },
            TransactionInputType::Stake {
                asset: asset.clone(),
                stake_type: StakeType::Stake(DelegationValidator::mock()),
            },
            TransactionInputType::Stake {
                asset: asset.clone(),
                stake_type: StakeType::Freeze(Resource::Bandwidth),
            },
        ];
        for input_type in spending {
            assert!(input_type.spends_balance(), "{:?} must spend the sender balance", input_type.transaction_type());
        }

        let non_spending = [
            TransactionInputType::Stake {
                asset: asset.clone(),
                stake_type: StakeType::Unstake(Delegation::mock()),
            },
            TransactionInputType::Stake {
                asset: asset.clone(),
                stake_type: StakeType::Withdraw(Delegation::mock()),
            },
            TransactionInputType::Stake {
                asset: asset.clone(),
                stake_type: StakeType::Rewards(vec![DelegationValidator::mock()]),
            },
            TransactionInputType::Stake {
                asset: asset.clone(),
                stake_type: StakeType::Unfreeze(Resource::Bandwidth),
            },
            TransactionInputType::TokenApprove {
                asset: Asset::mock_spl_token(),
                approval_data: ApprovalData::mock(),
            },
            TransactionInputType::Account {
                asset: asset.clone(),
                account_type: AccountDataType::Activate,
            },
            TransactionInputType::TransferNft {
                asset: asset.clone(),
                nft_asset: NFTAsset::mock(),
            },
            TransactionInputType::Perpetual {
                asset: asset.clone(),
                perpetual_type: PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)),
            },
            TransactionInputType::Perpetual {
                asset,
                perpetual_type: PerpetualType::Close(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)),
            },
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

        let full_balance_payment = input(TransactionInputType::Transfer { asset: Asset::mock_btc() }, 100_000_000, 100_000_000, 100_000_000);
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
        let mut zero_transfer = input(TransactionInputType::Transfer { asset: hypercore.clone() }, 0, 1_000_000, 0);
        zero_transfer.fee_asset = hypercore.id;
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
            input(TransactionInputType::Transfer { asset: Asset::mock() }, 999_000, 1_000_000, 1_000_000)
                .calculate()
                .unwrap_err(),
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
            TransactionInputType::Stake {
                asset: Asset::mock_sol(),
                stake_type: StakeType::Stake(DelegationValidator::mock()),
            },
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

        let token = input(TransactionInputType::Transfer { asset: Asset::mock_spl_token() }, 10_000_000, 10_000_000, 10_000);
        assert!(token.calculate().is_ok());
    }

    #[test]
    fn test_calculate_max_never_trims_a_contract_swap_below_the_quoted_amount() {
        const TON_FEE_WITH_ATTACHMENT: u64 = 320_000_000;
        let contract_swap = |from_value: &str, message_value: &str| TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Ton),
            to_asset: Asset::mock_ton_usdt(),
            swap_data: SwapData::mock_contract(SwapProvider::StonfiV2, from_value, "1000000", message_value),
        };
        let max = |input_type: TransactionInputType, value: u64| {
            let mut input = input(input_type, value, 1_215_893_271, 1_215_893_271);
            input.fee = BigInt::from(TON_FEE_WITH_ATTACHMENT);
            input.is_max_amount = true;
            input
        };

        let fits = max(contract_swap("885893271", "1195893271"), 885_893_271).calculate().unwrap();
        assert_eq!(fits.value, BigInt::from(885_893_271u64));
        assert!(fits.is_max_amount);

        assert_eq!(
            max(contract_swap("1195893271", "1505893271"), 1_195_893_271).calculate().unwrap_err(),
            TransferAmountError::InsufficientBalance {
                asset_id: AssetId::from_chain(Chain::Ton),
                required: BigInt::from(1_515_893_271u64),
                available: BigInt::from(1_215_893_271u64),
            },
            "a contract swap sends the quoted amount, so confirm must refuse it rather than show a trimmed one"
        );

        let transfer_swap = TransactionInputType::Swap {
            from_asset: Asset::from_chain(Chain::Ton),
            to_asset: Asset::mock_sol(),
            swap_data: SwapData::mock_transfer(SwapProvider::NearIntents, "1215893271", "1000000", "deposit"),
        };
        let trimmed = max(transfer_swap, 1_215_893_271).calculate().unwrap();
        assert_eq!(trimmed.value, BigInt::from(1_215_893_271u64 - TON_FEE_WITH_ATTACHMENT));
    }

    #[test]
    fn test_calculate_non_spending() {
        let unstake = input(
            TransactionInputType::Stake {
                asset: Asset::mock_sol(),
                stake_type: StakeType::Unstake(Delegation::mock()),
            },
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

        let below_reserve_unstake = input(
            TransactionInputType::Stake {
                asset: Asset::mock_sol(),
                stake_type: StakeType::Unstake(Delegation::mock()),
            },
            100,
            100,
            5_000_000,
        );
        assert!(
            below_reserve_unstake.calculate().is_ok(),
            "unstaking never spends the balance, so a delegation below the reserve is still allowed"
        );

        let approve = input(
            TransactionInputType::TokenApprove {
                asset: Asset::mock_spl_token(),
                approval_data: ApprovalData::mock(),
            },
            0,
            0,
            5_000_000,
        );
        assert!(approve.calculate().is_ok());

        let activate = input(
            TransactionInputType::Account {
                asset: Asset::mock_spl_token(),
                account_type: AccountDataType::Activate,
            },
            0,
            0,
            5_000_000,
        );
        assert!(activate.calculate().is_ok());
    }

    #[test]
    fn test_calculate_perpetual_never_spends_wallet_balance() {
        let asset = Asset::from_chain(Chain::HyperCore);

        let open = input(
            TransactionInputType::Perpetual {
                asset: asset.clone(),
                perpetual_type: PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)),
            },
            1_010_000_000,
            0,
            0,
        );
        assert!(
            open.calculate().is_ok(),
            "a perpetual is margined from the perpetual account, so an empty wallet must not gate opening it"
        );

        let close = input(
            TransactionInputType::Perpetual {
                asset,
                perpetual_type: PerpetualType::Close(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None)),
            },
            100,
            0,
            0,
        );
        assert!(close.calculate().is_ok());
    }
}
