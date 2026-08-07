use std::collections::HashMap;

use num_bigint::BigInt;
use primitives::{AssetId, Chain, SimulationBalanceChange, SimulationResult, SimulationWarning};

use crate::{
    Address,
    models::simulation::{TonEmulationActionType, TonEmulationJettonSwap, TonEmulationJettonTransfer, TonEmulationResponse},
};

pub(crate) fn map_simulation_result(signer: &Address, response: TonEmulationResponse) -> SimulationResult {
    if let Some(warning) = execution_warning(&response) {
        return SimulationResult::new(vec![warning], vec![]);
    }

    SimulationResult {
        balance_changes: map_balance_changes(signer, &response),
        ..Default::default()
    }
}

fn map_balance_changes(signer: &Address, response: &TonEmulationResponse) -> Vec<SimulationBalanceChange> {
    let mut deltas = HashMap::new();
    let native_change = response
        .transactions
        .values()
        .filter(|transaction| transaction.account == *signer)
        .fold(BigInt::ZERO, |total, transaction| {
            total + &transaction.account_state_after.balance - &transaction.account_state_before.balance
        });
    add_balance_change(&mut deltas, AssetId::from_chain(Chain::Ton), native_change);

    for action in &response.actions {
        if action.success != Some(true) {
            continue;
        }
        match &action.action {
            TonEmulationActionType::JettonSwap(details) => map_swap_balance_changes(signer, details, &mut deltas),
            TonEmulationActionType::JettonTransfer(details) => map_jetton_transfer_balance_change(signer, details, &mut deltas),
            TonEmulationActionType::Unsupported => {}
        }
    }

    let mut balance_changes: Vec<SimulationBalanceChange> = deltas
        .into_iter()
        .filter(|(_, value)| value != &BigInt::ZERO)
        .map(|(asset_id, value)| SimulationBalanceChange::new(asset_id, value))
        .collect();
    balance_changes.sort_by_key(|change| change.asset_id.to_string());
    balance_changes
}

fn execution_warning(response: &TonEmulationResponse) -> Option<SimulationWarning> {
    for transaction in response.transactions.values() {
        let Some(description) = &transaction.description else {
            continue;
        };
        if description.aborted {
            return Some(SimulationWarning::execution_error("TON transaction aborted"));
        }
        if let Some(compute) = &description.compute_ph
            && (!compute.success.unwrap_or(false) || compute.exit_code.is_some_and(|code| code != 0 && code != 1))
        {
            return Some(SimulationWarning::execution_error("TON compute phase failed"));
        }
        if let Some(action) = &description.action
            && !action.success.unwrap_or(false)
        {
            return Some(SimulationWarning::execution_error("TON action phase failed"));
        }
    }
    if response.actions.iter().any(|action| action.success == Some(false)) {
        return Some(SimulationWarning::execution_error("TON action failed"));
    }
    None
}

fn map_swap_balance_changes(signer: &Address, details: &TonEmulationJettonSwap, deltas: &mut HashMap<AssetId, BigInt>) {
    if details.sender != *signer {
        return;
    }
    if let Some(asset) = &details.asset_in {
        add_balance_change(deltas, jetton_asset_id(asset), -&details.dex_incoming_transfer.amount);
    }
    if let Some(asset) = &details.asset_out {
        add_balance_change(deltas, jetton_asset_id(asset), details.dex_outgoing_transfer.amount.clone());
    }
}

fn map_jetton_transfer_balance_change(signer: &Address, details: &TonEmulationJettonTransfer, deltas: &mut HashMap<AssetId, BigInt>) {
    let value = match (details.sender == *signer, details.receiver == *signer) {
        (true, false) => -&details.amount,
        (false, true) => details.amount.clone(),
        _ => return,
    };
    add_balance_change(deltas, jetton_asset_id(&details.asset), value);
}

fn jetton_asset_id(address: &Address) -> AssetId {
    AssetId::from_token(Chain::Ton, &address.encode_bounceable())
}

fn add_balance_change(deltas: &mut HashMap<AssetId, BigInt>, asset_id: AssetId, value: BigInt) {
    *deltas.entry(asset_id).or_default() += value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn signer() -> Address {
        Address::parse("0:33a14a5a9406979d59b9328898591660b8b1736342b11632efdcc911ab9057cf").unwrap()
    }

    #[test]
    fn test_map_simulation_result_surfaces_execution_failure() {
        let response = serde_json::from_str(include_str!("../../testdata/emulate_ton_connect_failed_response.json")).unwrap();

        let result = map_simulation_result(&signer(), response);

        assert_eq!(result.warnings, vec![SimulationWarning::execution_error("TON transaction aborted")]);
        assert!(result.balance_changes.is_empty());
    }

    #[test]
    fn test_map_simulation_result_rejects_malformed_balance() {
        let mut response: Value = serde_json::from_str(include_str!("../../testdata/emulate_ton_connect_dedust_response.json")).unwrap();
        response["transactions"]["root"]["account_state_after"]["balance"] = Value::String("invalid".to_string());

        assert!(serde_json::from_value::<TonEmulationResponse>(response).is_err());
    }
}
