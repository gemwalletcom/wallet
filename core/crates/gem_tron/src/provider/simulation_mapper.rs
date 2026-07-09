use std::collections::HashMap;

use num_bigint::BigInt;
use num_traits::Zero;
use primitives::{Address as _, AssetId, Chain, SimulationBalanceChange, SimulationResult, SimulationWarning};

use crate::address::TronAddress;
use crate::models::{TriggerConstantContractResponse, TronLog};
use crate::rpc::constants::ERC20_TRANSFER_EVENT_SIGNATURE;

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

    let mut token_deltas: HashMap<String, BigInt> = HashMap::new();
    for log in response.logs.iter().flatten() {
        let Some((token, delta)) = decode_transfer_delta(log, owner) else { continue };
        *token_deltas.entry(token).or_default() += delta;
    }

    changes.extend(
        token_deltas
            .into_iter()
            .filter(|(_, delta)| !delta.is_zero())
            .map(|(token, delta)| SimulationBalanceChange::new(AssetId::from_token(Chain::Tron, &token), delta)),
    );

    changes.sort_by_key(|change| change.asset_id.to_string());
    changes
}

fn decode_transfer_delta(log: &TronLog, owner: &TronAddress) -> Option<(String, BigInt)> {
    let topics = log.topics.as_ref()?;
    if topics.len() != 3 || topics[0] != ERC20_TRANSFER_EVENT_SIGNATURE {
        return None;
    }

    let from = TronAddress::from_topic(&topics[1])?;
    let to = TronAddress::from_topic(&topics[2])?;
    let amount = BigInt::parse_bytes(log.data.as_deref()?.as_bytes(), 16)?;

    let delta = match (from == *owner, to == *owner) {
        (false, false) => return None,
        (true, false) => -amount,
        (false, true) => amount,
        (true, true) => BigInt::default(),
    };

    Some((log.address?.encode(), delta))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> TronAddress {
        TronAddress::from_hex_or_base58("TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM").unwrap()
    }

    #[test]
    fn test_map_simulation_result_swap_with_logs() {
        let response: TriggerConstantContractResponse = serde_json::from_str(include_str!("../../testdata/trigger_constant_contract_swap_with_logs.json")).unwrap();

        let result = map_simulation_result(&owner(), &response, Some(1_000_000));

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

        let result = map_simulation_result(&owner(), &response, Some(1_000_000));

        assert_eq!(result.warnings, vec![SimulationWarning::execution_error("REVERT opcode executed")]);
        assert!(result.balance_changes.is_empty());
    }

    #[test]
    fn test_decode_transfer_delta_ignores_uninvolved_transfer() {
        let log = TronLog {
            address: TronAddress::from_hex_or_base58("DVz9MDHhhhUv2XskVieSNVc4U4fN1Rbss"),
            topics: Some(vec![
                ERC20_TRANSFER_EVENT_SIGNATURE.to_string(),
                "0000000000000000000000000344a87b2c5bc1cd9407fb9bd0c325a4403af30b".to_string(),
                "0000000000000000000000004e4bee11cea0070f957b98fd8cf4138ef3295e0e".to_string(),
            ]),
            data: Some("00000000000000000000000000000000000000000000000000000000000f4240".to_string()),
        };

        assert!(decode_transfer_delta(&log, &owner()).is_none());
    }

    #[test]
    fn test_decode_transfer_delta_ignores_non_transfer_topic() {
        let log = TronLog {
            address: TronAddress::from_hex_or_base58("DVz9MDHhhhUv2XskVieSNVc4U4fN1Rbss"),
            topics: Some(vec!["e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c".to_string()]),
            data: Some("00".to_string()),
        };

        assert!(decode_transfer_delta(&log, &owner()).is_none());
    }
}
