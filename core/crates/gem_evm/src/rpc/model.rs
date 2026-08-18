use num_bigint::{BigInt, BigUint};
use primitives::{TransactionState, contract_constants::EVM_ZERO_BLOCK_HASH};
use serde::{Deserialize, Serialize};
use serde_serializers::{
    bigint_from_hex_str, deserialize_biguint_from_hex_str, deserialize_biguint_from_option_hex_str, deserialize_u64_from_str, deserialize_u64_from_str_or_int,
};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub transactions: Vec<Transaction>,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str")]
    pub timestamp: BigUint,
}

#[derive(Debug, Deserialize)]
pub struct BlockHeader {
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
    #[serde(default)]
    pub input: String,
    pub to: Option<String>,
    #[serde(default, deserialize_with = "deserialize_biguint_from_hex_str")]
    pub value: BigUint,
    #[serde(default)]
    pub calls: Option<Vec<TransactionCall>>,
}

impl Transaction {
    pub(crate) fn with_primary_call(&self) -> Transaction {
        match self.calls.as_ref().and_then(|calls| calls.last()) {
            Some(call) => Transaction {
                to: call.to.clone(),
                value: BigUint::from(0u8),
                input: call.input.clone(),
                ..self.clone()
            },
            None => self.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCall {
    pub to: Option<String>,
    pub input: String,
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
    #[serde(default, deserialize_with = "deserialize_u64_from_str")]
    pub block_number: u64,
    #[serde(default)]
    pub fee_token: Option<String>,
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
        self.block_number != 0 && self.block_hash != EVM_ZERO_BLOCK_HASH
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
    use primitives::asset_constants::TEMPO_USDC_TOKEN_ID;
    use primitives::testkit::json_rpc::load_json_rpc_result;

    use super::*;
    use crate::address::ethereum_address_checksum;

    const BATCH_PRIMARY_CALL_TARGET: &str = "0xA2Dc7d0266f0CC50b3eEaF36c9BFCeCFF1BEea91";

    #[test]
    fn test_decode_batched_call_transaction() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/tempo_swap_batched_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/tempo_swap_batched_tx_receipt.json"));
        let block = load_json_rpc_result::<Block>(include_str!("../../testdata/tempo_swap_batched_block.json"));

        assert_eq!(transaction.to, None);
        assert_eq!(transaction.value, BigUint::from(0u8));
        assert_eq!(transaction.input, "");
        assert_eq!(transaction.calls.as_ref().unwrap().len(), 2);
        assert_eq!(ethereum_address_checksum(receipt.fee_token.as_deref().unwrap()).unwrap(), TEMPO_USDC_TOKEN_ID);
        assert_eq!(receipt.get_fee(), BigUint::from(471_789u64 * 1_260_212_000u64));
        assert_eq!(block.transactions.len(), 1);
        assert_eq!(block.transactions[0].hash, transaction.hash);

        let swap_transaction = transaction.with_primary_call();
        assert_eq!(ethereum_address_checksum(swap_transaction.to.as_deref().unwrap()).unwrap(), BATCH_PRIMARY_CALL_TARGET);
        assert_eq!(swap_transaction.value, BigUint::from(0u8));
        assert_eq!(&swap_transaction.input[..10], "0x3593564c");

        let mut single_call = transaction.clone();
        single_call.calls.as_mut().unwrap().truncate(1);
        let approval_transaction = single_call.with_primary_call();
        assert_eq!(ethereum_address_checksum(approval_transaction.to.as_deref().unwrap()).unwrap(), TEMPO_USDC_TOKEN_ID);
        assert_eq!(&approval_transaction.input[..10], "0x095ea7b3");

        let type2 = load_json_rpc_result::<Transaction>(include_str!("../../testdata/transfer_erc20.json"));
        let unchanged = type2.with_primary_call();
        assert_eq!(type2.calls, None);
        assert_eq!(unchanged.to, type2.to);
        assert_eq!(unchanged.input, type2.input);
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
