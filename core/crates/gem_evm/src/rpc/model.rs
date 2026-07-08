use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use primitives::{TransactionState, contract_constants::EVM_ZERO_BLOCK_HASH};
use serde::{Deserialize, Serialize};
use serde_serializers::{bigint_from_hex_str, deserialize_biguint_from_hex_str, deserialize_biguint_from_option_hex_str, deserialize_u64_from_str_or_int};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transactions: Vec<Transaction>,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub timestamp: BigUint,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockTransactionsIds {
    pub transactions: Vec<String>,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub timestamp: BigUint,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub from: String,
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub gas: u64,
    pub hash: String,
    pub input: String,
    pub to: Option<String>,
    #[serde(rename = "blockNumber", default, deserialize_with = "deserialize_biguint_from_hex_str")]
    pub block_number: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub value: BigUint,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub gas_used: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub effective_gas_price: BigUint,
    #[serde(default, deserialize_with = "deserialize_biguint_from_option_hex_str")]
    pub l1_fee: Option<BigUint>,
    pub logs: Vec<Log>,
    pub status: String,
    pub block_hash: String,
    #[serde(default, deserialize_with = "deserialize_biguint_from_hex_str")]
    pub block_number: BigUint,
}

impl TransactionReceipt {
    pub fn get_fee(&self) -> BigUint {
        let fee = self.gas_used.clone() * self.effective_gas_price.clone();
        if let Some(l1_fee) = self.l1_fee.clone() {
            return fee + l1_fee;
        }
        fee
    }

    pub fn has_valid_block_reference(&self) -> bool {
        !self.block_number.is_zero() && self.block_hash != EVM_ZERO_BLOCK_HASH
    }

    pub fn get_state(&self) -> TransactionState {
        if !self.has_valid_block_reference() {
            return TransactionState::Pending;
        }

        match self.status.as_str() {
            "0x1" => TransactionState::Confirmed,
            "0x0" => TransactionState::Reverted,
            _ => TransactionState::Pending,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    #[serde(default)]
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReplayTrace {
    #[serde(default)]
    pub state_diff: HashMap<String, StateChange>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallResult {
    #[serde(default)]
    pub state_diff: HashMap<String, StateChange>,
    #[serde(default)]
    pub trace: Vec<TraceCallEntry>,
}

impl TraceCallResult {
    pub fn root_call_error(&self) -> Option<&str> {
        self.trace.iter().find(|entry| entry.trace_address.is_empty())?.error.as_deref()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallEntry {
    pub action: TraceCallAction,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub trace_address: Vec<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallAction {
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: Option<String>,
    #[serde(default)]
    pub input: String,
    #[serde(default)]
    pub call_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChange {
    pub balance: Diff<String>,
    pub storage: HashMap<String, Diff<String>>,
}

impl StateChange {
    pub fn balance_change(&self) -> Option<(BigInt, BigInt)> {
        let Diff::Change(change) = &self.balance else { return None };
        let from = bigint_from_hex_str(&change.from_to.from).ok()?;
        let to = bigint_from_hex_str(&change.from_to.to).ok()?;
        Some((from, to))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Diff<T> {
    Change(Change<T>),
    Add(Add<T>),
    Delete(Delete<T>),
    Keep(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Change<T> {
    #[serde(rename = "*")]
    pub from_to: FromTo<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Add<T> {
    #[serde(rename = "+")]
    pub value: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Delete<T> {
    #[serde(rename = "-")]
    pub value: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FromTo<T> {
    pub from: T,
    pub to: T,
}

#[cfg(test)]
mod tests {
    use primitives::testkit::json_rpc::load_json_rpc_result;

    use super::*;

    #[test]
    fn test_decode_trace_replay_transaction() {
        let trace_replay_transaction = load_json_rpc_result::<TransactionReplayTrace>(include_str!("../../testdata/trace_replay_tx_trace.json"));

        assert!(trace_replay_transaction.state_diff.len() > 1);
    }

    #[test]
    fn test_root_call_error_detects_top_level_revert_only() {
        let reverted_root: TraceCallResult = load_json_rpc_result(include_str!("../../testdata/trace_call_reverted_root.json"));
        assert_eq!(reverted_root.root_call_error(), Some("Reverted"));

        let reverted_subcall_only: TraceCallResult = load_json_rpc_result(include_str!("../../testdata/trace_call_subcall_reverted.json"));
        assert_eq!(reverted_subcall_only.root_call_error(), None);
    }

    #[test]
    fn test_trace_call_result_tolerates_selfdestruct_action() {
        let trace: TraceCallResult = load_json_rpc_result(include_str!("../../testdata/trace_call_selfdestruct_action.json"));

        let entry = &trace.trace[0];
        assert_eq!(entry.action.from, "");
        assert_eq!(entry.action.call_type, None);
    }
}
