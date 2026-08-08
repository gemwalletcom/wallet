use std::error::Error;

use chrono::{DateTime, Utc};
use num_bigint::{BigInt, BigUint};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::deserialize_bigint_from_str;

use crate::models::transaction::{STATUS_FAILURE, STATUS_SUCCESS};
use crate::models::{BalanceChange, Digest, Effect, Event, GasObject, GasUsed, Owner, OwnerObject, Status};

#[derive(Debug, Deserialize)]
pub(super) struct GraphqlTransaction {
    digest: String,
    effects: GraphqlEffects,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlEffects {
    status: GraphqlExecutionStatus,
    timestamp: DateTime<Utc>,
    gas_effects: GraphqlGasEffects,
    balance_changes: GraphqlConnection<GraphqlBalanceChange>,
    events: GraphqlConnection<GraphqlEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GraphqlExecutionStatus {
    Success,
    Failure,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlGasEffects {
    gas_object: GraphqlGasObject,
    gas_summary: GraphqlGasSummary,
}

#[derive(Debug, Deserialize)]
struct GraphqlGasObject {
    owner: GraphqlOwner,
}

#[derive(Debug, Deserialize)]
struct GraphqlOwner {
    address: GraphqlAddress,
}

#[derive(Debug, Deserialize)]
struct GraphqlAddress {
    address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlGasSummary {
    computation_cost: u64,
    storage_cost: u64,
    storage_rebate: u64,
    non_refundable_storage_fee: u64,
}

#[derive(Debug, Deserialize)]
struct GraphqlConnection<T> {
    nodes: Vec<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlBalanceChange {
    owner: GraphqlAddress,
    coin_type: GraphqlMoveType,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    amount: BigInt,
}

#[derive(Debug, Deserialize)]
struct GraphqlMoveType {
    repr: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlEvent {
    contents: GraphqlEventContents,
    transaction_module: GraphqlTransactionModule,
}

#[derive(Debug, Deserialize)]
struct GraphqlEventContents {
    #[serde(rename = "type")]
    event_type: GraphqlMoveType,
    json: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct GraphqlTransactionModule {
    package: GraphqlPackage,
}

#[derive(Debug, Deserialize)]
struct GraphqlPackage {
    address: String,
}

pub(super) fn map_transaction(transaction: GraphqlTransaction) -> Result<Digest, Box<dyn Error + Send + Sync>> {
    let status = match transaction.effects.status {
        GraphqlExecutionStatus::Success => STATUS_SUCCESS,
        GraphqlExecutionStatus::Failure => STATUS_FAILURE,
    };
    let gas_summary = transaction.effects.gas_effects.gas_summary;
    let balance_changes = transaction
        .effects
        .balance_changes
        .nodes
        .into_iter()
        .map(|change| BalanceChange {
            owner: address_owner(change.owner.address),
            coin_type: change.coin_type.repr,
            amount: change.amount,
        })
        .collect();
    let events = transaction
        .effects
        .events
        .nodes
        .into_iter()
        .map(|event| Event {
            event_type: event.contents.event_type.repr,
            parsed_json: event.contents.json,
            package_id: event.transaction_module.package.address,
        })
        .collect();

    Ok(Digest {
        digest: transaction.digest,
        effects: Effect {
            gas_used: GasUsed {
                computation_cost: BigUint::from(gas_summary.computation_cost),
                storage_cost: BigUint::from(gas_summary.storage_cost),
                storage_rebate: BigUint::from(gas_summary.storage_rebate),
                non_refundable_storage_fee: BigUint::from(gas_summary.non_refundable_storage_fee),
            },
            status: Status { status: status.to_string() },
            gas_object: GasObject {
                owner: address_owner(transaction.effects.gas_effects.gas_object.owner.address.address),
            },
        },
        move_call_packages: Vec::new(),
        balance_changes: Some(balance_changes),
        events,
        timestamp_ms: transaction.effects.timestamp.timestamp_millis() as u64,
    })
}

fn address_owner(address: String) -> Owner {
    Owner::OwnerObject(OwnerObject { address_owner: Some(address) })
}
