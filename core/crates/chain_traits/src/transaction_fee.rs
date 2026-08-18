use primitives::{Asset, Chain, FeePriority, FeeRate, FeeUnitType, GasPriceType, SOLANA_PRIORITY_FEE_SCALE, TransactionFee};

pub enum TransactionFeeOperation {
    Transfer,
    TokenTransfer,
    Swap,
}

pub struct TransactionFeeEstimate {
    pub priority: FeePriority,
    pub fee: TransactionFee,
}

pub struct TransactionFeeEstimates {
    pub transfer: Vec<TransactionFeeEstimate>,
    pub token_transfer: Option<Vec<TransactionFeeEstimate>>,
    pub swap: Option<Vec<TransactionFeeEstimate>>,
}

impl TransactionFeeEstimate {
    pub(crate) fn new(chain: Chain, rate: &FeeRate, units: Option<u64>, fee_unit_type: FeeUnitType) -> Self {
        let (gas_price_type, fee) = match (&rate.gas_price_type, units) {
            (GasPriceType::Solana { gas_price, unit_price, .. }, Some(units)) => {
                let priority_fee = unit_price * units / SOLANA_PRIORITY_FEE_SCALE;
                let gas_price_type = GasPriceType::solana(gas_price.clone(), priority_fee, unit_price.clone());
                let fee = gas_price_type.total_fee();
                (gas_price_type, fee)
            }
            (gas_price_type, Some(units)) => {
                let fee = gas_price_type.total_fee() * units;
                let fee = match fee_unit_type {
                    FeeUnitType::SatVb => {
                        let scale = fee_unit_type.scale_factor();
                        (fee + scale - 1) / scale
                    }
                    FeeUnitType::Gwei | FeeUnitType::Native => fee,
                };
                (gas_price_type.clone(), fee)
            }
            (gas_price_type, None) => (gas_price_type.clone(), gas_price_type.total_fee()),
        };
        Self {
            priority: rate.priority,
            fee: TransactionFee::new_gas_price_type(gas_price_type, fee, units.unwrap_or(1).into(), Default::default(), Asset::from_chain(chain)),
        }
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, FeePriority, FeeRate, FeeUnitType, GasPriceType};

    use super::TransactionFeeEstimate;

    #[test]
    fn test_transaction_fee_estimate() {
        let evm = TransactionFeeEstimate::new(
            Chain::Ethereum,
            &FeeRate::new(FeePriority::Normal, GasPriceType::eip1559(51_000_000u64, 1_000_000u64)),
            Some(21_000),
            FeeUnitType::Gwei,
        );
        assert_eq!(evm.fee.fee.to_string(), "1092000000000");

        let bitcoin = TransactionFeeEstimate::new(
            Chain::Bitcoin,
            &FeeRate::new(FeePriority::Fast, GasPriceType::regular(20u64)),
            Some(141),
            FeeUnitType::SatVb,
        );
        assert_eq!(bitcoin.fee.fee.to_string(), "282");

        let fixed = TransactionFeeEstimate::new(
            Chain::Bitcoin,
            &FeeRate::new(FeePriority::Normal, GasPriceType::regular(10_000u64)),
            None,
            FeeUnitType::SatVb,
        );
        assert_eq!(fixed.fee.fee.to_string(), "10000");

        let solana = TransactionFeeEstimate::new(
            Chain::Solana,
            &FeeRate::new(FeePriority::Normal, GasPriceType::solana(5_000u64, 0u64, 100_000u64)),
            Some(100_000),
            FeeUnitType::Native,
        );
        assert_eq!(solana.fee.gas_price_type.priority_fee().to_string(), "10000");
        assert_eq!(solana.fee.fee.to_string(), "15000");
    }
}
