use num_bigint::BigInt;
use num_traits::Zero;
use primitives::{AssetId, Chain, SimulationBalanceChange, SimulationResult, SimulationWarning};

use crate::address::TronAddress;
use crate::models::TriggerConstantContractResponse;
use crate::provider::balance_diff::token_balance_deltas;

pub fn map_simulation_result(owner: &TronAddress, response: &TriggerConstantContractResponse, call_value: Option<u64>) -> SimulationResult {
    if let Err(error) = response.get_energy() {
        let message = error.message.clone().unwrap_or_else(|| error.to_string());
        return SimulationResult::new(vec![SimulationWarning::execution_error(message)], vec![]);
    }

    SimulationResult {
        balance_changes: map_balance_changes(owner, response, call_value),
        ..Default::default()
    }
}

fn map_balance_changes(owner: &TronAddress, response: &TriggerConstantContractResponse, call_value: Option<u64>) -> Vec<SimulationBalanceChange> {
    let mut changes = Vec::new();
    if let Some(value) = call_value.filter(|value| *value > 0) {
        changes.push(SimulationBalanceChange::new(AssetId::from_chain(Chain::Tron), -BigInt::from(value)));
    }

    changes.extend(
        token_balance_deltas(response.logs.as_deref().unwrap_or_default(), owner)
            .into_iter()
            .filter(|(_, delta)| !delta.is_zero())
            .map(|(token, delta)| SimulationBalanceChange::new(AssetId::from_token(Chain::Tron, &token), delta)),
    );

    changes.sort_by_key(|change| change.asset_id.to_string());
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Address as _;

    fn mock_owner() -> TronAddress {
        TronAddress::from_hex_or_base58("TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM").unwrap()
    }

    #[test]
    fn test_map_simulation_result_swap_with_logs() {
        let response: TriggerConstantContractResponse = serde_json::from_str(include_str!("../../testdata/trigger_constant_contract_swap_with_logs.json")).unwrap();

        let result = map_simulation_result(&mock_owner(), &response, Some(1_000_000));

        assert!(result.warnings.is_empty());
        let output_token = TronAddress::from_hex("4e4bee11cea0070f957b98fd8cf4138ef3295e0e").unwrap().encode();
        assert_eq!(
            result.balance_changes,
            vec![
                SimulationBalanceChange::new(AssetId::from_chain(Chain::Tron), BigInt::from(-1_000_000)),
                SimulationBalanceChange::new(AssetId::from_token(Chain::Tron, &output_token), BigInt::from(329_114)),
            ]
        );
    }

    #[test]
    fn test_map_simulation_result_reverted_returns_validation_warning() {
        let response: TriggerConstantContractResponse = serde_json::from_str(include_str!("../../testdata/trigger_constant_contract_reverted.json")).unwrap();

        let result = map_simulation_result(&mock_owner(), &response, Some(1_000_000));

        assert_eq!(result.warnings, vec![SimulationWarning::execution_error("REVERT opcode executed")]);
        assert!(result.balance_changes.is_empty());
    }
}
