use std::str::FromStr;

use num_bigint::BigInt;
use num_traits::Zero;
use primitives::{SimulationBalanceChange, SimulationResult, SimulationWarning};

use crate::provider::transactions_mapper::map_asset_id;
use crate::rpc::proto::{BalanceChange, ExecutedTransaction};

pub fn map_simulation_result(sender: &str, transaction: &ExecutedTransaction) -> SimulationResult {
    if !transaction.execution_success() {
        let message = transaction.execution_error().unwrap_or_else(|| "execution failed".to_string());
        return SimulationResult::new(vec![SimulationWarning::validation_error(message)], vec![]);
    }

    match map_balance_changes(sender, &transaction.balance_changes) {
        Ok(balance_changes) => SimulationResult {
            balance_changes,
            ..Default::default()
        },
        Err(message) => SimulationResult::new(vec![SimulationWarning::validation_error(message)], vec![]),
    }
}

fn map_balance_changes(sender: &str, balance_changes: &[BalanceChange]) -> Result<Vec<SimulationBalanceChange>, &'static str> {
    let mut changes = Vec::new();
    for change in balance_changes.iter().filter(|change| change.address.as_deref() == Some(sender)) {
        let amount = change
            .amount
            .as_deref()
            .and_then(|amount| BigInt::from_str(amount).ok())
            .ok_or("missing or malformed balance change amount")?;
        if amount.is_zero() {
            continue;
        }
        let coin_type = change.coin_type.as_deref().ok_or("missing balance change coin type")?;
        changes.push(SimulationBalanceChange::new(map_asset_id(coin_type), amount));
    }
    changes.sort_by_key(|change| change.asset_id.to_string());
    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SUI_COIN_TYPE_FULL;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_ADDRESS_EMPTY};
    use crate::rpc::proto::TransactionEffects;
    use primitives::asset_constants::SUI_USDC_TOKEN_ID;
    use primitives::{AssetId, Chain};

    #[test]
    fn test_map_simulation_result() {
        let transaction = ExecutedTransaction {
            effects: Some(TransactionEffects::mock(true, None)),
            balance_changes: vec![
                BalanceChange::mock(TEST_ADDRESS, SUI_COIN_TYPE_FULL, "-101744880"),
                BalanceChange::mock(TEST_ADDRESS, SUI_USDC_TOKEN_ID, "250000"),
                BalanceChange::mock(TEST_ADDRESS, SUI_USDC_TOKEN_ID, "0"),
                BalanceChange::mock(TEST_ADDRESS_EMPTY, SUI_COIN_TYPE_FULL, "100000000"),
            ],
            ..Default::default()
        };

        let result = map_simulation_result(TEST_ADDRESS, &transaction);

        assert!(result.warnings.is_empty());
        assert_eq!(
            result.balance_changes,
            vec![
                SimulationBalanceChange::new(AssetId::from_chain(Chain::Sui), BigInt::from(-101_744_880)),
                SimulationBalanceChange::new(AssetId::from_token(Chain::Sui, SUI_USDC_TOKEN_ID), BigInt::from(250_000)),
            ]
        );
    }

    #[test]
    fn test_map_simulation_result_malformed_signer_change_returns_validation_warning() {
        let cases = [
            (
                BalanceChange {
                    address: Some(TEST_ADDRESS.to_string()),
                    coin_type: Some(SUI_COIN_TYPE_FULL.to_string()),
                    amount: None,
                },
                "missing or malformed balance change amount",
            ),
            (
                BalanceChange {
                    address: Some(TEST_ADDRESS.to_string()),
                    coin_type: Some(SUI_COIN_TYPE_FULL.to_string()),
                    amount: Some("not-a-number".to_string()),
                },
                "missing or malformed balance change amount",
            ),
            (
                BalanceChange {
                    address: Some(TEST_ADDRESS.to_string()),
                    coin_type: None,
                    amount: Some("100".to_string()),
                },
                "missing balance change coin type",
            ),
        ];

        for (change, message) in cases {
            let transaction = ExecutedTransaction {
                effects: Some(TransactionEffects::mock(true, None)),
                balance_changes: vec![BalanceChange::mock(TEST_ADDRESS, SUI_COIN_TYPE_FULL, "-100"), change],
                ..Default::default()
            };

            let result = map_simulation_result(TEST_ADDRESS, &transaction);

            assert_eq!(result.warnings, vec![SimulationWarning::validation_error(message)]);
            assert!(result.balance_changes.is_empty());
        }

        // zero amounts stay skipped even without a coin type, and malformed changes of other addresses stay ignored
        let transaction = ExecutedTransaction {
            effects: Some(TransactionEffects::mock(true, None)),
            balance_changes: vec![
                BalanceChange {
                    address: Some(TEST_ADDRESS.to_string()),
                    coin_type: None,
                    amount: Some("0".to_string()),
                },
                BalanceChange {
                    address: Some(TEST_ADDRESS_EMPTY.to_string()),
                    coin_type: None,
                    amount: None,
                },
                BalanceChange::mock(TEST_ADDRESS, SUI_COIN_TYPE_FULL, "-100"),
            ],
            ..Default::default()
        };

        let result = map_simulation_result(TEST_ADDRESS, &transaction);

        assert!(result.warnings.is_empty());
        assert_eq!(
            result.balance_changes,
            vec![SimulationBalanceChange::new(AssetId::from_chain(Chain::Sui), BigInt::from(-100))]
        );
    }

    #[test]
    fn test_map_simulation_result_failed_execution_returns_validation_warning() {
        let transaction = ExecutedTransaction {
            effects: Some(TransactionEffects::mock(false, Some("InsufficientGas"))),
            balance_changes: vec![BalanceChange::mock(TEST_ADDRESS, SUI_COIN_TYPE_FULL, "-101744880")],
            ..Default::default()
        };

        let result = map_simulation_result(TEST_ADDRESS, &transaction);

        assert_eq!(result.warnings, vec![SimulationWarning::validation_error("InsufficientGas")]);
        assert!(result.balance_changes.is_empty());

        let missing_status = ExecutedTransaction::default();
        assert_eq!(
            map_simulation_result(TEST_ADDRESS, &missing_status).warnings,
            vec![SimulationWarning::validation_error("execution failed")]
        );
    }
}
