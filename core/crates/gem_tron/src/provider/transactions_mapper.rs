use chain_primitives::{BalanceDiff, SwapMapper};
use chrono::{DateTime, Utc};
use num_bigint::{BigInt, BigUint};
use primitives::{
    Address as _, AssetId, Transaction, TransactionResourceTypeMetadata, TransactionState, TransactionSwapMetadata, TransactionType, chain::Chain, decode_hex,
    hex::decode_hex_utf8, stake_type::Resource,
};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

use crate::address::TronAddress;
use crate::models::{
    BlockTransactions, ContractParameterValue, InternalTransaction, Transaction as TronTransaction, TransactionReceiptData, TronContractType, TronLog, TronTransactionBroadcast,
};
use crate::provider::balance_diff::{internal_transaction_deltas, token_balance_deltas};
use crate::rpc::constants::ERC20_TRANSFER_EVENT_SIGNATURE;
use crate::trc20;

fn decode_hex_message(hex_str: &str) -> String {
    decode_hex_utf8(hex_str).unwrap_or_else(|| hex_str.to_string())
}

fn resource_type_metadata(resource: Option<&str>) -> Option<Value> {
    let resource_type = resource.and_then(|resource| resource.parse::<Resource>().ok()).unwrap_or(Resource::Bandwidth);
    serde_json::to_value(TransactionResourceTypeMetadata::new(resource_type)).ok()
}

fn tron_swap_metadata(
    chain: Chain,
    owner: &TronAddress,
    call_value: Option<u64>,
    logs: &[TronLog],
    internal_transactions: &[InternalTransaction],
) -> Option<TransactionSwapMetadata> {
    // Native TRX keys as `None` so its call_value and internal transfer legs merge into a single diff.
    let mut deltas: HashMap<Option<String>, BigInt> = HashMap::new();

    for (token, delta) in token_balance_deltas(logs, owner) {
        *deltas.entry(Some(token)).or_default() += delta;
    }

    for (token_id, delta) in internal_transaction_deltas(internal_transactions, owner) {
        *deltas.entry(token_id).or_default() += delta;
    }

    if let Some(call_value) = call_value.filter(|value| *value > 0) {
        *deltas.entry(None).or_default() -= BigInt::from(call_value);
    }

    let balance_diffs: Vec<BalanceDiff> = deltas
        .into_iter()
        .map(|(token, diff)| BalanceDiff {
            asset_id: match token {
                Some(token) => AssetId { chain, token_id: Some(token) },
                None => chain.as_asset_id(),
            },
            diff,
        })
        .collect();

    SwapMapper::map_swap(&balance_diffs, &BigUint::from(0u8), &chain.as_asset_id(), None)
}

pub fn map_transaction_broadcast(response: &TronTransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(message) = &response.message {
        Err(decode_hex_message(message).into())
    } else if let Some(txid) = &response.txid {
        Ok(txid.clone())
    } else {
        Err("Transaction broadcast failed with unknown error".into())
    }
}

pub fn map_transactions_by_block(chain: Chain, block: BlockTransactions, receipts: Vec<TransactionReceiptData>) -> Vec<Transaction> {
    block
        .transactions
        .into_iter()
        .zip(receipts)
        .filter_map(|(transaction, receipt)| map_transaction(chain, transaction, receipt))
        .collect()
}

pub fn map_transaction(chain: Chain, transaction: TronTransaction, receipt: TransactionReceiptData) -> Option<Transaction> {
    let contract = transaction.raw_data.contract.first()?.clone();
    let contract_result = transaction.ret.first()?;
    let context = TransactionContext::new(
        chain,
        transaction.transaction_id,
        contract.parameter.value.owner_address.clone(),
        &contract_result.contract_ret,
        receipt.fee,
        receipt.block_time_stamp,
        transaction.raw_data.data.as_deref(),
    )?;

    if let Some(transaction) = context.map_native_contract(contract.contract_type, &contract.parameter.value) {
        return Some(transaction);
    }

    if contract.contract_type == Some(TronContractType::TriggerSmart) {
        return context.map_trigger_smart_contract(
            &contract.parameter.value,
            &receipt.log.unwrap_or_default(),
            &receipt.internal_transactions.unwrap_or_default(),
        );
    }

    None
}

struct TransactionContext {
    chain: Chain,
    hash: String,
    from: String,
    state: TransactionState,
    fee: String,
    fee_asset_id: AssetId,
    memo: Option<String>,
    created_at: DateTime<Utc>,
}

impl TransactionContext {
    fn new(chain: Chain, hash: String, owner_address: Option<String>, contract_ret: &str, fee: Option<i64>, block_time_stamp: i64, data: Option<&str>) -> Option<Self> {
        Some(Self {
            chain,
            hash,
            from: owner_address.unwrap_or_default(),
            state: map_transaction_state(contract_ret),
            fee: fee.unwrap_or_default().to_string(),
            fee_asset_id: chain.as_asset_id(),
            memo: data.map(decode_hex_message),
            created_at: DateTime::from_timestamp_millis(block_time_stamp)?,
        })
    }

    fn map_native_contract(&self, contract_type: Option<TronContractType>, contract_value: &ContractParameterValue) -> Option<Transaction> {
        let (transaction_type, to, value, metadata) = match contract_type? {
            TronContractType::Transfer => (
                TransactionType::Transfer,
                contract_value.to_address.clone().unwrap_or_default(),
                contract_value.amount.unwrap_or_default().to_string(),
                None,
            ),
            TronContractType::FreezeBalanceV2 => (
                TransactionType::StakeFreeze,
                self.from.clone(),
                contract_value.frozen_balance.unwrap_or_default().to_string(),
                resource_type_metadata(contract_value.resource.as_deref()),
            ),
            TronContractType::UnfreezeBalanceV2 => (
                TransactionType::StakeUnfreeze,
                self.from.clone(),
                contract_value.unfreeze_balance.unwrap_or_default().to_string(),
                resource_type_metadata(contract_value.resource.as_deref()),
            ),
            TronContractType::VoteWitness => self.vote_witness_transaction_data(contract_value)?,
            _ => return None,
        };

        Some(self.build_transaction(self.fee_asset_id.clone(), self.from.clone(), to, transaction_type, value, metadata))
    }

    fn vote_witness_transaction_data(&self, contract_value: &ContractParameterValue) -> Option<(TransactionType, String, String, Option<Value>)> {
        let vote = contract_value.votes.as_ref()?.first()?;
        let to = TronAddress::from_hex(vote.vote_address.as_str())?.encode();
        let value = (vote.vote_count * 1_000_000).to_string();
        Some((TransactionType::StakeDelegate, to, value, None))
    }

    fn map_trigger_smart_contract(&self, contract_value: &ContractParameterValue, logs: &[TronLog], internal_transactions: &[InternalTransaction]) -> Option<Transaction> {
        self.map_token_approval(contract_value)
            .or_else(|| self.map_swap(contract_value, logs, internal_transactions))
            .or_else(|| self.map_token_transfer(contract_value, logs))
    }

    fn map_token_approval(&self, contract_value: &ContractParameterValue) -> Option<Transaction> {
        let token_id = contract_value.contract_address.as_ref()?;
        let approval = trc20::decode_approval_hex(contract_value.data.as_deref()?)?;

        Some(self.build_transaction(
            AssetId::from_token(self.chain, token_id),
            self.from.clone(),
            approval.spender.encode(),
            TransactionType::TokenApproval,
            approval.value.to_string(),
            None,
        ))
    }

    fn map_swap(&self, contract_value: &ContractParameterValue, logs: &[TronLog], internal_transactions: &[InternalTransaction]) -> Option<Transaction> {
        let owner = TronAddress::from_hex_or_base58(&self.from)?;
        let swap = tron_swap_metadata(self.chain, &owner, contract_value.call_value, logs, internal_transactions)?;

        Some(self.build_transaction(
            swap.from_asset.clone(),
            self.from.clone(),
            self.from.clone(),
            TransactionType::Swap,
            swap.from_value.clone(),
            serde_json::to_value(&swap).ok(),
        ))
    }

    fn map_token_transfer(&self, contract_value: &ContractParameterValue, logs: &[TronLog]) -> Option<Transaction> {
        if logs.len() != 1 {
            return None;
        }

        let log = logs.first()?;
        let topics = log.topics.as_ref()?;
        if topics.len() != 3 || topics.first()?.as_str() != ERC20_TRANSFER_EVENT_SIGNATURE {
            return None;
        }

        let from = TronAddress::from_topic(&topics[1])?.encode();
        let to = TronAddress::from_topic(&topics[2])?.encode();
        let value = BigUint::from_bytes_be(&decode_hex(log.data.as_deref()?).ok()?).to_string();
        let asset_id = AssetId::from_token(self.chain, contract_value.contract_address.as_ref()?);

        Some(self.build_transaction(asset_id, from, to, TransactionType::Transfer, value, None))
    }

    fn build_transaction(&self, asset_id: AssetId, from: String, to: String, transaction_type: TransactionType, value: String, metadata: Option<Value>) -> Transaction {
        Transaction::new(
            self.hash.clone(),
            asset_id,
            from,
            to,
            None,
            transaction_type,
            self.state,
            self.fee.clone(),
            self.fee_asset_id.clone(),
            value,
            self.memo.clone(),
            metadata,
            self.created_at,
        )
    }
}

fn map_transaction_state(contract_ret: &str) -> TransactionState {
    if contract_ret == "SUCCESS" {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BlockTransactions, TransactionReceipt, TransactionReceiptData, TriggerConstantContractResponse, TronContractType, TronTransactionBroadcast};
    use crate::provider::testkit::{TEST_TOKEN_APPROVAL_TRANSACTION_ID, TEST_TRANSACTION_ID};
    use primitives::asset_constants::TRON_USDT_TOKEN_ID;

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response: TronTransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_error.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Contract validate error : Cannot transfer TRX to yourself.");
    }

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response: TronTransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_success.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "7f60ccd0594b5c3e0264cca9a6e6e64cb96ee66ce3a796b4356cb8ccc548f62b");
    }

    #[test]
    fn test_map_transaction_broadcast_unknown_error() {
        let response = TronTransactionBroadcast {
            txid: None,
            code: None,
            message: None,
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction broadcast failed with unknown error");
    }

    #[test]
    fn test_map_transaction_freeze_bandwidth() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_freeze.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1758589896000,
            result: None,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeFreeze);
        assert_eq!(transaction.value, "100000000");
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.metadata, serde_json::to_value(TransactionResourceTypeMetadata::new(Resource::Bandwidth)).ok());
    }

    #[test]
    fn test_map_transaction_freeze_energy() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_freeze_energy.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1760552376000,
            result: None,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeFreeze);
        assert_eq!(transaction.value, "10000000");
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.metadata, serde_json::to_value(TransactionResourceTypeMetadata::new(Resource::Energy)).ok());
    }

    #[test]
    fn test_map_transaction_stake() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_stake.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1758225849000,
            result: None,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(transaction.value, "2125000000");
        assert_eq!(transaction.from, "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb");
        assert_eq!(transaction.to, "TJvaAeFb8Lykt9RQcVyyTFN2iDvGMuyD4M");
    }

    #[test]
    fn test_map_transaction_unfreeze() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_unfreeze.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1758596982000,
            result: None,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeUnfreeze);
        assert_eq!(transaction.value, "100000000");
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.metadata, serde_json::to_value(TransactionResourceTypeMetadata::new(Resource::Bandwidth)).ok());
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_coin_transfer.json")).unwrap();
        let receipt: TransactionReceiptData = serde_json::from_str(include_str!("../../testdata/transaction_coin_transfer_receipt.json")).unwrap();

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, "25000000");
        assert_ne!(transaction.from, transaction.to);
    }

    #[test]
    fn test_map_transaction_token_transfer() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_token_transfer.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1727747910000,
            result: None,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: Some(vec![crate::models::TronLog {
                address: None,
                topics: Some(vec![
                    "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".to_string(),
                    "0000000000000000000000002e1d447fa4169390cf5f5b3d12d380decfbfe20f".to_string(),
                    "0000000000000000000000006e2cf2878020b966786f01ab45ea1fcef6880092".to_string(),
                ]),
                data: Some("00000000000000000000000000000000000000000000000000000000017d7840".to_string()),
            }]),
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_ne!(transaction.from, transaction.to);
    }

    #[test]
    fn test_map_transaction_token_approval() {
        let failed = map_transaction(
            Chain::Tron,
            TronTransaction::mock_token_approval("OUT_OF_ENERGY"),
            TransactionReceiptData::mock_transaction_receipt_with_result("OUT_OF_ENERGY"),
        )
        .unwrap();

        assert_eq!(failed.hash, TEST_TOKEN_APPROVAL_TRANSACTION_ID);
        assert_eq!(failed.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(failed.from, "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7");
        assert_eq!(failed.to, "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7");
        assert_eq!(failed.value, "0");
        assert_eq!(failed.transaction_type, TransactionType::TokenApproval);
        assert_eq!(failed.state, TransactionState::Failed);

        let confirmed = map_transaction(
            Chain::Tron,
            TronTransaction::mock_token_approval("SUCCESS"),
            TransactionReceiptData::mock_transaction_receipt_with_result("SUCCESS"),
        )
        .unwrap();

        assert_eq!(confirmed.transaction_type, TransactionType::TokenApproval);
        assert_eq!(confirmed.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_trigger_smart_swap() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_trigger_smart_swap.json")).unwrap();
        let swap_logs: TriggerConstantContractResponse = serde_json::from_str(include_str!("../../testdata/trigger_constant_contract_swap_with_logs.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1758589896000,
            result: None,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: swap_logs.logs,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.asset_id, Chain::Tron.as_asset_id());
        assert_eq!(transaction.value, "1000000");
        let output_token = TronAddress::from_hex("4e4bee11cea0070f957b98fd8cf4138ef3295e0e").unwrap().encode();
        let metadata: TransactionSwapMetadata = serde_json::from_value(transaction.metadata.unwrap()).unwrap();
        assert_eq!(metadata.from_asset, Chain::Tron.as_asset_id());
        assert_eq!(metadata.from_value, "1000000");
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Tron,
                token_id: Some(output_token)
            }
        );
        assert_eq!(metadata.to_value, "329114");
    }

    #[test]
    fn test_map_transaction_trigger_smart_swap_credits_native_leg_from_internal_transaction() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_token_transfer.json")).unwrap();
        let usdt_transfer_out = TronLog::mock_transfer(
            "41a614f803b6fd780986a42c78ec9c7f77e6ded13c",
            "0000000000000000000000002e1d447fa4169390cf5f5b3d12d380decfbfe20f",
            "0000000000000000000000006e2cf2878020b966786f01ab45ea1fcef6880092",
            "00000000000000000000000000000000000000000000000000000000017d7840",
        );
        let trx_unwrap_in = InternalTransaction::mock(
            "416e2cf2878020b966786f01ab45ea1fcef6880092",
            "412e1d447fa4169390cf5f5b3d12d380decfbfe20f",
            900_000,
            None,
            false,
        );
        let receipt = TransactionReceiptData::mock_transaction_receipt_with_logs(vec![usdt_transfer_out], vec![trx_unwrap_in]);

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Swap);

        let metadata: TransactionSwapMetadata = serde_json::from_value(transaction.metadata.unwrap()).unwrap();
        let usdt = TronAddress::from_hex("41a614f803b6fd780986a42c78ec9c7f77e6ded13c").unwrap().encode();
        assert_eq!(
            metadata.from_asset,
            AssetId {
                chain: Chain::Tron,
                token_id: Some(usdt)
            }
        );
        assert_eq!(metadata.from_value, "25000000");
        assert_eq!(metadata.to_asset, Chain::Tron.as_asset_id());
        assert_eq!(metadata.to_value, "900000");
    }

    #[test]
    fn test_map_transactions_by_block_ignores_unsupported_contract_types() {
        let block: BlockTransactions = serde_json::from_str(include_str!("../../testdata/block_mixed_contract_types.json")).unwrap();
        let receipts: Vec<TransactionReceiptData> = serde_json::from_str(include_str!("../../testdata/block_mixed_contract_types_receipts.json")).unwrap();

        assert_eq!(block.transactions.len(), 5);
        assert_eq!(block.transactions[0].raw_data.contract[0].contract_type, Some(TronContractType::DelegateResource));
        assert_eq!(block.transactions[1].raw_data.contract[0].contract_type, Some(TronContractType::UnDelegateResource));
        assert_eq!(block.transactions[2].raw_data.contract[0].contract_type, Some(TronContractType::TransferAsset));
        assert_eq!(block.transactions[3].raw_data.contract[0].contract_type, None);

        let transactions = map_transactions_by_block(Chain::Tron, block, receipts);

        assert_eq!(transactions.len(), 1);
        let transaction = transactions.first().unwrap();
        assert_eq!(transaction.hash, "10f1e5b04c0dd39f14d4b5ca270899b36ae9c52ac1b9b64b76360c7373cc0893");
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(transaction.from, "TWBPGLwQw2EbqYLLw1DJnTDt2ZQ9yJW1JJ");
        assert_eq!(transaction.to, "TViSMURdt2dda6Pf163UBZoSfbV9hECvvc");
        assert_eq!(transaction.value, "249000000");
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_thorchain_swap() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_thorchain_swap.json")).unwrap();
        let receipt = TransactionReceiptData {
            id: "test_id".to_string(),
            fee: Some(1000),
            block_number: 12345,
            block_time_stamp: 1771951038000,
            result: None,
            receipt: TransactionReceipt {
                result: Some("SUCCESS".to_string()),
            },
            log: None,
            internal_transactions: None,
        };

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, "200000000");
        assert_eq!(transaction.memo.as_deref(), Some("=:TRON.USDT:TNAwd1WFe7GHTxovGU9MeT6mi3J4KAZMvP:0/1/0:g1:50"));
    }
}
