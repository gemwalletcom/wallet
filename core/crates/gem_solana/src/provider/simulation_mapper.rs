use std::collections::{HashMap, HashSet};

use num_bigint::BigInt;
use primitives::{AssetId, Chain, SimulationBalanceChange, SimulationResult, SimulationSeverity, SimulationWarning, SimulationWarningType};
use serde_json::Value;

use crate::models::{SimulateTransactionValue, TokenBalance};

pub fn map_simulation_result(account_keys: &[String], signer_addresses: &HashSet<String>, simulation: SimulateTransactionValue) -> SimulationResult {
    if let Some(err) = simulation.err {
        return SimulationResult::new(vec![simulation_error_warning(err)], vec![]);
    }

    SimulationResult {
        balance_changes: map_balance_changes(
            account_keys,
            signer_addresses,
            &simulation.pre_balances,
            &simulation.post_balances,
            &simulation.pre_token_balances.unwrap_or_default(),
            &simulation.post_token_balances.unwrap_or_default(),
        ),
        ..Default::default()
    }
}

fn simulation_error_warning(error: Value) -> SimulationWarning {
    let message = match error {
        Value::String(message) => message,
        error => error.to_string(),
    };
    SimulationWarning::new(SimulationSeverity::Critical, SimulationWarningType::ValidationError, Some(message))
}

fn map_balance_changes(
    account_keys: &[String],
    signer_addresses: &HashSet<String>,
    pre_balances: &[u64],
    post_balances: &[u64],
    pre_token_balances: &[TokenBalance],
    post_token_balances: &[TokenBalance],
) -> Vec<SimulationBalanceChange> {
    let mut deltas: HashMap<AssetId, BigInt> = HashMap::new();
    for (asset_id, value) in signer_asset_values(account_keys, signer_addresses, post_balances, post_token_balances) {
        *deltas.entry(asset_id).or_default() += value;
    }
    for (asset_id, value) in signer_asset_values(account_keys, signer_addresses, pre_balances, pre_token_balances) {
        *deltas.entry(asset_id).or_default() -= value;
    }

    let mut balance_changes: Vec<SimulationBalanceChange> = deltas
        .into_iter()
        .filter(|(_, value)| *value != BigInt::from(0))
        .map(|(asset_id, value)| SimulationBalanceChange {
            asset_id,
            value: value.to_string(),
        })
        .collect();
    balance_changes.sort_by_key(|change| change.asset_id.to_string());
    balance_changes
}

fn signer_asset_values(account_keys: &[String], signer_addresses: &HashSet<String>, balances: &[u64], token_balances: &[TokenBalance]) -> Vec<(AssetId, BigInt)> {
    let mut values: Vec<(AssetId, BigInt)> = token_balances
        .iter()
        .filter(|token_balance| signer_addresses.contains(&token_balance.owner))
        .map(|token_balance| (AssetId::from_token(Chain::Solana, &token_balance.mint), BigInt::from(token_balance.get_amount())))
        .collect();

    for (index, address) in account_keys.iter().enumerate() {
        if !signer_addresses.contains(address) {
            continue;
        }
        if let Some(balance) = balances.get(index) {
            values.push((AssetId::from_chain(Chain::Solana), BigInt::from(*balance)));
        }
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::{SOLANA_USDC_ASSET_ID, SOLANA_USDC_TOKEN_ID};

    fn signers(addresses: &[&str]) -> HashSet<String> {
        addresses.iter().map(|address| address.to_string()).collect()
    }

    #[test]
    fn test_map_balance_changes() {
        let account_keys = vec!["wallet".to_string(), "recipient".to_string()];
        let pre_balances = vec![1_000_000_000, 0];
        let post_balances = vec![899_995_000, 100_000_000];
        let pre_tokens = vec![TokenBalance::mock(SOLANA_USDC_TOKEN_ID, "wallet", 1_000_000)];
        let post_tokens = vec![TokenBalance::mock(SOLANA_USDC_TOKEN_ID, "wallet", 250_000)];

        let changes = map_balance_changes(&account_keys, &signers(&["wallet"]), &pre_balances, &post_balances, &pre_tokens, &post_tokens);

        assert_eq!(
            changes,
            vec![
                SimulationBalanceChange {
                    asset_id: AssetId::from_chain(Chain::Solana),
                    value: "-100005000".to_string(),
                },
                SimulationBalanceChange {
                    asset_id: SOLANA_USDC_ASSET_ID.clone(),
                    value: "-750000".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_map_balance_changes_ignores_non_signers() {
        let account_keys = vec!["wallet".to_string(), "recipient".to_string()];
        let pre_balances = vec![1_000_000_000, 0];
        let post_balances = vec![999_995_000, 5_000_000];
        let pre_tokens = vec![TokenBalance::mock(SOLANA_USDC_TOKEN_ID, "recipient", 1_000_000)];
        let post_tokens = vec![TokenBalance::mock(SOLANA_USDC_TOKEN_ID, "recipient", 2_000_000)];

        let changes = map_balance_changes(&account_keys, &signers(&["wallet"]), &pre_balances, &post_balances, &pre_tokens, &post_tokens);

        assert_eq!(
            changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_chain(Chain::Solana),
                value: "-5000".to_string(),
            }]
        );
    }

    #[test]
    fn test_map_balance_changes_token_account_close() {
        let account_keys = vec!["wallet".to_string()];
        let pre_tokens = vec![TokenBalance::mock(SOLANA_USDC_TOKEN_ID, "wallet", 1_000_000)];

        let changes = map_balance_changes(&account_keys, &signers(&["wallet"]), &[1_000_000_000], &[1_000_000_000], &pre_tokens, &[]);

        assert_eq!(
            changes,
            vec![SimulationBalanceChange {
                asset_id: SOLANA_USDC_ASSET_ID.clone(),
                value: "-1000000".to_string(),
            }]
        );
    }

    #[test]
    fn test_map_balance_changes_nets_multiple_signers() {
        let account_keys = vec!["wallet".to_string(), "cosigner".to_string()];

        let changes = map_balance_changes(&account_keys, &signers(&["wallet", "cosigner"]), &[1_000_000_000, 50_000], &[999_995_000, 65_000], &[], &[]);

        assert_eq!(
            changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_chain(Chain::Solana),
                value: "10000".to_string(),
            }]
        );
    }

    #[test]
    fn test_map_balance_changes_ignores_loaded_account_balances_without_keys() {
        let account_keys = vec!["wallet".to_string()];

        let changes = map_balance_changes(&account_keys, &signers(&["wallet"]), &[1_000_000_000, 10, 20], &[999_995_000, 1_000, 2_000], &[], &[]);

        assert_eq!(
            changes,
            vec![SimulationBalanceChange {
                asset_id: AssetId::from_chain(Chain::Solana),
                value: "-5000".to_string(),
            }]
        );
    }

    #[test]
    fn test_map_simulation_result_returns_validation_warning_for_failed_simulation() {
        let result = map_simulation_result(
            &[],
            &HashSet::new(),
            SimulateTransactionValue {
                err: Some(serde_json::json!({"InstructionError":[1, "InvalidArgument"]})),
                pre_balances: vec![],
                post_balances: vec![],
                pre_token_balances: None,
                post_token_balances: None,
            },
        );

        assert_eq!(
            result.warnings,
            vec![SimulationWarning::new(
                SimulationSeverity::Critical,
                SimulationWarningType::ValidationError,
                Some("{\"InstructionError\":[1,\"InvalidArgument\"]}".to_string()),
            )]
        );
        assert_eq!(result.balance_changes, vec![]);
    }
}
