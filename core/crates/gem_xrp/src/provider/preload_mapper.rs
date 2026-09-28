use crate::models::rpc::{AccountInfoResult, Fee};
use num_bigint::BigInt;
use primitives::{FeePriority, FeeRate, GasPriceType, TransactionLoadMetadata};
use std::error::Error;

const NORMAL_FEE_PERCENT: u64 = 120;
const FAST_FEE_PERCENT: u64 = 200;

pub fn map_fee_rates(fee: &Fee) -> Vec<FeeRate> {
    let base_fee = fee.open_ledger_fee.max(fee.minimum_fee);
    let rate = |percent: u64| GasPriceType::regular(BigInt::from((base_fee * percent).div_ceil(100)));
    vec![FeeRate::new(FeePriority::Normal, rate(NORMAL_FEE_PERCENT)), FeeRate::new(FeePriority::Fast, rate(FAST_FEE_PERCENT))]
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

    #[test]
    fn test_map_fee_rates() {
        assert_eq!(
            map_fee_rates(&Fee { minimum_fee: 10, open_ledger_fee: 10 }),
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(12))),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(20)))
            ]
        );
        assert_eq!(
            map_fee_rates(&Fee { minimum_fee: 10, open_ledger_fee: 5000 }),
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(6000))),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(10000)))
            ]
        );
        assert_eq!(
            map_fee_rates(&Fee { minimum_fee: 10, open_ledger_fee: 5 }),
            vec![
                FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(12))),
                FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(20)))
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
