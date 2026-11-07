use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use primitives::Address as _;
use primitives::{
    Asset, AssetId, Chain, Resource, SimulationBalanceChange, SimulationHeader, SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldKind, SimulationPayloadFieldType, SimulationResult, SimulationSeverity,
    SimulationWarning, SimulationWarningApproval, SimulationWarningType, TransactionType,
};

use crate::TransactionApproval;
use crate::address::TronAddress;
use crate::models::TriggerConstantContractResponse;
use crate::provider::balance_diff::token_balance_deltas;
use crate::transaction::TronContract;

pub fn map_simulation_result(owner: &TronAddress, response: &TriggerConstantContractResponse, call_value: Option<u64>) -> SimulationResult {
    SimulationResult {
        balance_changes: map_balance_changes(owner, response, call_value),
        ..Default::default()
    }
}

pub(crate) fn map_approval_simulation(decoded: TransactionApproval) -> SimulationResult {
    let TransactionApproval { approval, contract, expiration } = decoded;
    let asset_id = AssetId::from_token(Chain::Tron, &approval.token);
    let value = (!approval.is_unlimited).then_some(approval.value);
    let mut result = SimulationResult {
        header: Some(SimulationHeader {
            asset_id: asset_id.clone(),
            value: value.clone(),
            is_unlimited: approval.is_unlimited,
        }),
        warnings: vec![SimulationWarning::new(
            SimulationSeverity::Low,
            SimulationWarningType::TokenApproval(SimulationWarningApproval { asset_id, value: value.map(BigInt::from) }),
            None,
        )],
        payload: vec![
            SimulationPayloadField::standard(SimulationPayloadFieldKind::Method, "approve", SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary),
            SimulationPayloadField::standard(SimulationPayloadFieldKind::Contract, contract, SimulationPayloadFieldType::Address, SimulationPayloadFieldDisplay::Primary),
            SimulationPayloadField::standard(SimulationPayloadFieldKind::Spender, approval.spender, SimulationPayloadFieldType::Address, SimulationPayloadFieldDisplay::Primary),
        ],
        balance_changes: vec![],
    };
    if let Some(expiration) = expiration {
        result.payload.push(SimulationPayloadField::standard(
            SimulationPayloadFieldKind::Expiration,
            expiration.to_string(),
            SimulationPayloadFieldType::Timestamp,
            SimulationPayloadFieldDisplay::Primary,
        ));
    }
    result
}

const DEFAULT_DELEGATE_LOCK_BLOCKS: u64 = 86_400;

pub(crate) fn map_stake_simulation(contract: &TronContract, withdrawal: Option<BigUint>) -> Option<SimulationResult> {
    let (value, fields) = match contract {
        TronContract::Transfer { .. } | TronContract::TriggerSmart { .. } => return None,
        TronContract::VoteWitness { votes, .. } => (
            Some(votes.iter().map(|vote| BigUint::from(vote.count)).sum::<BigUint>() * BigUint::from(10u64).pow(Asset::from_chain(Chain::Tron).decimals)),
            votes.iter().map(|vote| address_field(SimulationPayloadFieldKind::Validator, vote.address.encode())).collect(),
        ),
        TronContract::FreezeBalanceV2 { frozen_balance, resource, .. } => (Some(BigUint::from(*frozen_balance)), vec![resource_field((*resource).into())]),
        TronContract::UnfreezeBalanceV2 { unfreeze_balance, resource, .. } => (Some(BigUint::from(*unfreeze_balance)), vec![resource_field((*resource).into())]),
        TronContract::DelegateResource {
            receiver,
            balance,
            resource,
            lock,
            lock_period,
            ..
        } => (
            Some(BigUint::from(*balance)),
            [
                Some(address_field(SimulationPayloadFieldKind::Recipient, receiver.encode())),
                Some(resource_field((*resource).into())),
                lock.then(|| lock_time_field(*lock_period)),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        TronContract::UnDelegateResource { receiver, balance, resource, .. } => (Some(BigUint::from(*balance)), vec![address_field(SimulationPayloadFieldKind::Recipient, receiver.encode()), resource_field((*resource).into())]),
        TronContract::WithdrawBalance { .. } | TronContract::WithdrawExpireUnfreeze { .. } => (withdrawal, vec![]),
    };
    Some(SimulationResult {
        payload: [action_field(contract)].into_iter().chain(fields).collect(),
        header: value.map(|value| SimulationHeader {
            asset_id: AssetId::from_chain(Chain::Tron),
            value: Some(value),
            is_unlimited: false,
        }),
        ..SimulationResult::default()
    })
}

fn action_field(contract: &TronContract) -> SimulationPayloadField {
    match stake_transaction_type(contract) {
        Some(transaction_type) => SimulationPayloadField::standard(SimulationPayloadFieldKind::Action, transaction_type.as_ref(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary),
        None => SimulationPayloadField::standard(SimulationPayloadFieldKind::Method, contract.kind().to_string(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary),
    }
}

fn stake_transaction_type(contract: &TronContract) -> Option<TransactionType> {
    match contract {
        TronContract::VoteWitness { .. } => Some(TransactionType::StakeDelegate),
        TronContract::FreezeBalanceV2 { .. } => Some(TransactionType::StakeFreeze),
        TronContract::UnfreezeBalanceV2 { .. } => Some(TransactionType::StakeUnfreeze),
        TronContract::WithdrawBalance { .. } => Some(TransactionType::StakeRewards),
        TronContract::WithdrawExpireUnfreeze { .. } => Some(TransactionType::StakeWithdraw),
        TronContract::Transfer { .. } | TronContract::TriggerSmart { .. } | TronContract::DelegateResource { .. } | TronContract::UnDelegateResource { .. } => None,
    }
}

fn address_field(kind: SimulationPayloadFieldKind, address: String) -> SimulationPayloadField {
    SimulationPayloadField::standard(kind, address, SimulationPayloadFieldType::Address, SimulationPayloadFieldDisplay::Primary)
}

fn resource_field(resource: Resource) -> SimulationPayloadField {
    SimulationPayloadField::standard(SimulationPayloadFieldKind::Resource, resource.as_ref(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary)
}

fn lock_time_field(lock_period: u64) -> SimulationPayloadField {
    let blocks = if lock_period == 0 { DEFAULT_DELEGATE_LOCK_BLOCKS } else { lock_period };
    let seconds = blocks.saturating_mul(u64::from(Chain::Tron.block_time()) / 1_000);
    SimulationPayloadField::standard(SimulationPayloadFieldKind::LockTime, seconds.to_string(), SimulationPayloadFieldType::Text, SimulationPayloadFieldDisplay::Primary)
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
    use crate::transaction::decode_wallet_connect_single_contract;
    use primitives::ApprovalData;

    #[test]
    fn test_map_approval_simulation() {
        for (value, is_unlimited) in [(BigUint::from(100u32), false), (BigUint::from_bytes_be(&[0xff; 32]), true)] {
            let result = map_approval_simulation(TransactionApproval {
                contract: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(),
                expiration: None,
                approval: ApprovalData {
                    token: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(),
                    spender: "TJoSEwEqt7cT3TUwmEoUYnYs5cZR3xSukM".to_string(),
                    value: value.clone(),
                    is_unlimited,
                },
            });
            assert_eq!(
                result.header,
                Some(SimulationHeader {
                    asset_id: AssetId::from_token(Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"),
                    value: if is_unlimited { None } else { Some(value) },
                    is_unlimited
                })
            );
            assert_eq!(
                result.payload.iter().map(|field| field.display.clone()).collect::<Vec<_>>(),
                vec![SimulationPayloadFieldDisplay::Primary, SimulationPayloadFieldDisplay::Primary, SimulationPayloadFieldDisplay::Primary]
            );
        }
    }

    fn fixture_contract(transaction: &serde_json::Value) -> TronContract {
        decode_wallet_connect_single_contract(&serde_json::json!({ "transaction": transaction }).to_string()).unwrap().unwrap()
    }

    fn delegation_contract(name: &str) -> TronContract {
        let fixtures: Vec<serde_json::Value> = serde_json::from_str(include_str!("../../testdata/wallet_connect_resource_delegation.json")).unwrap();
        fixture_contract(&fixtures.into_iter().find(|fixture| fixture["name"] == name).unwrap()["transaction"])
    }

    #[test]
    fn test_map_stake_simulation() {
        const RECEIVER: &str = "THTR75o8xXAgCTQqpiot2AFRAjvW1tSbVV";
        let primary = SimulationPayloadFieldDisplay::Primary;
        let cases = [
            (
                delegation_contract("delegate_energy"),
                None,
                Some(2_000_000u64),
                vec![
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Method, "DelegateResourceContract", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Recipient, RECEIVER, SimulationPayloadFieldType::Address, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Resource, "energy", SimulationPayloadFieldType::Text, primary.clone()),
                ],
            ),
            (
                delegation_contract("delegate_energy_locked"),
                None,
                Some(2_000_000),
                vec![
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Method, "DelegateResourceContract", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Recipient, RECEIVER, SimulationPayloadFieldType::Address, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Resource, "energy", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::LockTime, "86400", SimulationPayloadFieldType::Text, primary.clone()),
                ],
            ),
            (
                delegation_contract("delegate_energy_default_lock"),
                None,
                Some(2_000_000),
                vec![
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Method, "DelegateResourceContract", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Recipient, RECEIVER, SimulationPayloadFieldType::Address, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Resource, "energy", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::LockTime, "259200", SimulationPayloadFieldType::Text, primary.clone()),
                ],
            ),
            (
                delegation_contract("undelegate_bandwidth"),
                None,
                Some(2_000_000),
                vec![
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Method, "UnDelegateResourceContract", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Recipient, RECEIVER, SimulationPayloadFieldType::Address, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Resource, "bandwidth", SimulationPayloadFieldType::Text, primary.clone()),
                ],
            ),
            (
                fixture_contract(&serde_json::from_str(include_str!("../../testdata/transaction_stake.json")).unwrap()),
                None,
                Some(2_125_000_000),
                vec![
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Action, "stakeDelegate", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Validator, "TJvaAeFb8Lykt9RQcVyyTFN2iDvGMuyD4M", SimulationPayloadFieldType::Address, primary.clone()),
                ],
            ),
            (
                fixture_contract(&serde_json::from_str(include_str!("../../testdata/transaction_freeze_energy.json")).unwrap()),
                None,
                Some(10_000_000),
                vec![
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Action, "stakeFreeze", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Resource, "energy", SimulationPayloadFieldType::Text, primary.clone()),
                ],
            ),
            (
                fixture_contract(&serde_json::from_str(include_str!("../../testdata/transaction_unfreeze.json")).unwrap()),
                None,
                Some(100_000_000),
                vec![
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Action, "stakeUnfreeze", SimulationPayloadFieldType::Text, primary.clone()),
                    SimulationPayloadField::standard(SimulationPayloadFieldKind::Resource, "bandwidth", SimulationPayloadFieldType::Text, primary.clone()),
                ],
            ),
            (
                TronContract::WithdrawBalance { owner: TronAddress::mock() },
                Some(7_000_000u64),
                Some(7_000_000),
                vec![SimulationPayloadField::standard(SimulationPayloadFieldKind::Action, "stakeRewards", SimulationPayloadFieldType::Text, primary.clone())],
            ),
            (
                TronContract::WithdrawBalance { owner: TronAddress::mock() },
                None,
                None,
                vec![SimulationPayloadField::standard(SimulationPayloadFieldKind::Action, "stakeRewards", SimulationPayloadFieldType::Text, primary.clone())],
            ),
            (
                TronContract::WithdrawExpireUnfreeze { owner: TronAddress::mock() },
                Some(8_000_000),
                Some(8_000_000),
                vec![SimulationPayloadField::standard(SimulationPayloadFieldKind::Action, "stakeWithdraw", SimulationPayloadFieldType::Text, primary.clone())],
            ),
        ];
        for (contract, withdrawal, value, payload) in cases {
            assert_eq!(
                map_stake_simulation(&contract, withdrawal.map(BigUint::from)),
                Some(SimulationResult {
                    payload,
                    header: value.map(|value| SimulationHeader {
                        asset_id: AssetId::from_chain(Chain::Tron),
                        value: Some(BigUint::from(value)),
                        is_unlimited: false,
                    }),
                    ..SimulationResult::default()
                }),
                "{:?}",
                contract.kind()
            );
        }

        assert_eq!(
            map_stake_simulation(
                &TronContract::Transfer {
                    owner: TronAddress::mock(),
                    to: TronAddress::mock(),
                    amount: 1
                },
                None
            ),
            None
        );
        assert_eq!(
            map_stake_simulation(
                &TronContract::TriggerSmart {
                    owner: TronAddress::mock(),
                    contract: TronAddress::mock(),
                    data: vec![],
                    call_value: None,
                    call_token_value: None,
                    token_id: None,
                },
                None
            ),
            None
        );
    }

    #[test]
    fn test_map_simulation_result_swap_with_logs() {
        let response: TriggerConstantContractResponse = serde_json::from_str(include_str!("../../testdata/trigger_constant_contract_swap_with_logs.json")).unwrap();

        let result = map_simulation_result(&TronAddress::mock(), &response, Some(1_000_000));

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
}
