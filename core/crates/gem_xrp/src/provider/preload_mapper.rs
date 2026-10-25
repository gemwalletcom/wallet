use crate::models::{
    XrpSwap,
    rpc::{AccountInfoResult, Fee},
};
use num_bigint::BigInt;
use primitives::{AssetId, Chain, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionLoadInput, TransactionLoadMetadata, swap::SwapQuoteDataType};
use std::error::Error;

const NORMAL_FEE_MULTIPLIER: u64 = 2;
const FAST_FEE_MULTIPLIER: u64 = 4;

pub fn map_fee_rates(fee: &Fee) -> Vec<FeeRate> {
    let base_fee = fee.open_ledger_fee.max(fee.minimum_fee);
    vec![
        FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(base_fee * NORMAL_FEE_MULTIPLIER))),
        FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(base_fee * FAST_FEE_MULTIPLIER))),
    ]
}

pub fn map_transaction_fee(input: &TransactionLoadInput) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
    let Ok(swap_data) = input.input_type.get_swap_data() else {
        return Ok(input.default_fee());
    };
    match swap_data.data.data_type {
        SwapQuoteDataType::Contract => {
            let swap: XrpSwap = serde_json::from_str(&swap_data.data.data)?;
            Ok(TransactionFee::calculate(swap.fee_multiplier(), &input.gas_price, AssetId::from_chain(Chain::Xrp)))
        }
        SwapQuoteDataType::Transfer => Ok(input.default_fee()),
    }
}

pub fn map_transaction_preload(account_result: AccountInfoResult, destination_exists: bool) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
    if let Some(account_data) = account_result.account_data {
        Ok(TransactionLoadMetadata::Xrp {
            sequence: account_data.sequence,
            block_number: account_result.ledger_current_index,
            is_destination_address_exist: destination_exists,
        })
    } else {
        Err("Account not found".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rpc::{AccountInfo, AccountInfoResult};
    use num_bigint::BigUint;
    use primitives::{
        Asset, TransactionInputType,
        swap::{SwapData, SwapQuoteData},
    };

    #[test]
    fn test_map_transaction_fee() {
        let batch = |swap: XrpSwap| SwapQuoteData {
            data: serde_json::to_string(&swap).unwrap(),
            ..SwapQuoteData::mock()
        };
        for (data, fee) in [
            (batch(XrpSwap { trust_line: true, ..XrpSwap::mock() }), 100),
            (batch(XrpSwap::mock()), 80),
            (batch(XrpSwap { fee: None, ..XrpSwap::mock() }), 20),
            (SwapQuoteData::new_transfer("rM27yzkCw6WA3T4g1sPaeC1kpxHUhuxRWn".to_string(), BigUint::from(10u64), None), 20),
        ] {
            let input = TransactionLoadInput {
                gas_price: GasPriceType::regular(BigInt::from(20)),
                ..TransactionLoadInput::mock_with_input_type(TransactionInputType::Swap {
                    from_asset: Asset::from_chain(Chain::Xrp),
                    to_asset: Asset::mock_xrp_rlusd(),
                    swap_data: SwapData { data, ..SwapData::mock() },
                })
            };

            assert_eq!(map_transaction_fee(&input).unwrap().fee, BigInt::from(fee));
        }
    }

    #[test]
    fn test_map_fee_rates() {
        assert_eq!(
            map_fee_rates(&Fee { minimum_fee: 10, open_ledger_fee: 10 }),
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(20))),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(40)))
            ]
        );
        assert_eq!(
            map_fee_rates(&Fee { minimum_fee: 10, open_ledger_fee: 5000 }),
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(10000))),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(20000)))
            ]
        );
        assert_eq!(
            map_fee_rates(&Fee { minimum_fee: 10, open_ledger_fee: 5 }),
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(20))),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(40)))
            ]
        );
    }

    #[test]
    fn test_map_transaction_preload_with_account_data() {
        let account_result = AccountInfoResult {
            account_data: Some(AccountInfo {
                balance: 1000000,
                sequence: 12345,
                owner_count: 0,
                account: Some("rAccount123".to_string()),
                flags: Some(0),
                ledger_entry_type: Some("AccountRoot".to_string()),
            }),
            ledger_current_index: 67890,
        };

        let result = map_transaction_preload(account_result, false).unwrap();

        if let TransactionLoadMetadata::Xrp {
            sequence,
            block_number,
            is_destination_address_exist,
        } = result
        {
            assert_eq!(sequence, 12345);
            assert_eq!(block_number, 67890);
            assert!(!is_destination_address_exist);
        } else {
            panic!("Expected XRP metadata");
        }
    }

    #[test]
    fn test_map_transaction_preload_without_account_data() {
        let account_result = AccountInfoResult {
            account_data: None,
            ledger_current_index: 67890,
        };

        let result = map_transaction_preload(account_result, true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Account not found");
    }
}
