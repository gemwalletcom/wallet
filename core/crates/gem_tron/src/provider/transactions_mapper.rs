use alloy_primitives::{Address, U256};
use chain_primitives::{BalanceDiff, SwapMapper};
use chrono::{DateTime, Utc};
use gem_evm::contracts::{IOkxDexRouter, ISunSwapReferralVault, OkxCommission};
use num_bigint::{BigInt, BigUint};
use primitives::{
    Address as _, AssetId, SwapProvider, Transaction, TransactionResourceTypeMetadata, TransactionState, TransactionSwapMetadata, TransactionSwapReferralFee, TransactionType,
    chain::Chain,
    contract_constants::{EVM_NATIVE_TOKEN_ADDRESS, TRON_SUNSWAP_REFERRAL_VAULT_CONTRACT, TRON_SUNSWAP_UNIVERSAL_ROUTER_CONTRACT},
    hex::decode_hex_utf8,
    stake_type::Resource,
    swap::TRON_REFERRAL_ADDRESS,
};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use crate::address::TronAddress;
use crate::models::{BlockTransactions, ContractParameterValue, InternalTransaction, Transaction as TronTransaction, TransactionReceiptData, TronContractType, TronLog, TronTransactionBroadcast};
use crate::provider::balance_diff::{decode_token_transfer, internal_transaction_deltas, token_balance_deltas};
use crate::trc20;

fn decode_hex_message(hex_str: &str) -> String {
    decode_hex_utf8(hex_str).unwrap_or_else(|| hex_str.to_string())
}

fn resource_type_metadata(resource: Option<&str>) -> Option<Value> {
    let resource_type = resource.and_then(|resource| resource.parse::<Resource>().ok()).unwrap_or(Resource::Bandwidth);
    serde_json::to_value(TransactionResourceTypeMetadata::new(resource_type)).ok()
}

fn tron_swap_metadata(chain: Chain, owner: &TronAddress, contract: Option<&str>, call_value: Option<u64>, logs: &[TronLog], internal_transactions: &[InternalTransaction]) -> Option<TransactionSwapMetadata> {
    let provider = swap_provider(contract, logs);
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

    SwapMapper::map_swap(&balance_diffs, &BigUint::from(0u8), &chain.as_asset_id(), provider).map(|swap| swap.with_referral_fee(referral_fee(chain, owner, logs)))
}

fn swap_provider(contract: Option<&str>, logs: &[TronLog]) -> Option<SwapProvider> {
    let router = TronAddress::from_hex_or_base58(TRON_SUNSWAP_UNIVERSAL_ROUTER_CONTRACT);
    if contract.and_then(TronAddress::from_hex_or_base58).is_some_and(|contract| Some(contract) == router) {
        return Some(SwapProvider::SunswapV3);
    }
    logs.iter().any(|log| log.decode_event::<IOkxDexRouter::OrderRecord>().is_some()).then_some(SwapProvider::Okx)
}

fn referral_fee(chain: Chain, owner: &TronAddress, logs: &[TronLog]) -> Option<TransactionSwapReferralFee> {
    let referral = TronAddress::from_hex_or_base58(TRON_REFERRAL_ADDRESS)?;
    if *owner == referral {
        return None;
    }
    okx_referral_fee(chain, &referral, logs).or_else(|| sunswap_referral_fee(chain, &referral, logs))
}

fn okx_referral_fee(chain: Chain, referral: &TronAddress, logs: &[TronLog]) -> Option<TransactionSwapReferralFee> {
    let native_token = Address::from_str(EVM_NATIVE_TOKEN_ADDRESS).ok()?;
    logs.iter().find_map(|log| {
        let commission = log
            .decode_event::<IOkxDexRouter::CommissionFromTokenRecord>()
            .map(OkxCommission::from)
            .or_else(|| log.decode_event::<IOkxDexRouter::CommissionToTokenRecord>().map(OkxCommission::from))?;
        (TronAddress::from(commission.referrer) == *referral).then(|| referral_fee_value(chain, commission.token, native_token, commission.amount))
    })
}

fn sunswap_referral_fee(chain: Chain, referral: &TronAddress, logs: &[TronLog]) -> Option<TransactionSwapReferralFee> {
    let vault = TronAddress::from_hex_or_base58(TRON_SUNSWAP_REFERRAL_VAULT_CONTRACT)?;
    logs.iter().filter(|log| log.address == Some(vault)).find_map(|log| {
        let allocation = log.decode_event::<ISunSwapReferralVault::ReferralAllocated>()?;
        (TronAddress::from(allocation.rebateRecipient) == *referral).then(|| referral_fee_value(chain, allocation.token, Address::ZERO, allocation.rebateAmount))
    })
}

fn referral_fee_value(chain: Chain, token: Address, native_token: Address, amount: U256) -> TransactionSwapReferralFee {
    TransactionSwapReferralFee {
        asset_id: match token == native_token {
            true => chain.as_asset_id(),
            false => AssetId::from_token(chain, &TronAddress::from(token).encode()),
        },
        value: BigUint::from_bytes_be(&amount.to_be_bytes::<32>()),
    }
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
    block.transactions.into_iter().zip(receipts).filter_map(|(transaction, receipt)| map_transaction(chain, transaction, receipt)).collect()
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
        return context.map_trigger_smart_contract(&contract.parameter.value, &receipt.log.unwrap_or_default(), &receipt.internal_transactions.unwrap_or_default());
    }

    None
}

struct TransactionContext {
    chain: Chain,
    hash: String,
    from: String,
    state: TransactionState,
    fee: BigUint,
    fee_asset_id: AssetId,
    memo: Option<String>,
    created_at: DateTime<Utc>,
}

impl TransactionContext {
    fn new(chain: Chain, hash: String, owner_address: Option<String>, contract_ret: &str, fee: Option<u64>, block_time_stamp: i64, data: Option<&str>) -> Option<Self> {
        Some(Self {
            chain,
            hash,
            from: owner_address.unwrap_or_default(),
            state: map_transaction_state(contract_ret),
            fee: BigUint::from(fee.unwrap_or_default()),
            fee_asset_id: chain.as_asset_id(),
            memo: data.map(decode_hex_message),
            created_at: DateTime::from_timestamp_millis(block_time_stamp)?,
        })
    }

    fn map_native_contract(&self, contract_type: Option<TronContractType>, contract_value: &ContractParameterValue) -> Option<Transaction> {
        let (transaction_type, to, value, metadata): (TransactionType, String, BigUint, Option<Value>) = match contract_type? {
            TronContractType::Transfer => (TransactionType::Transfer, contract_value.to_address.clone().unwrap_or_default(), BigUint::from(contract_value.amount.unwrap_or_default()), None),
            TronContractType::FreezeBalanceV2 => (
                TransactionType::StakeFreeze,
                self.from.clone(),
                BigUint::from(contract_value.frozen_balance.unwrap_or_default()),
                resource_type_metadata(contract_value.resource.as_deref()),
            ),
            TronContractType::UnfreezeBalanceV2 => (
                TransactionType::StakeUnfreeze,
                self.from.clone(),
                BigUint::from(contract_value.unfreeze_balance.unwrap_or_default()),
                resource_type_metadata(contract_value.resource.as_deref()),
            ),
            TronContractType::VoteWitness => self.vote_witness_transaction_data(contract_value)?,
            _ => return None,
        };

        Some(self.build_transaction(self.fee_asset_id.clone(), self.from.clone(), to, transaction_type, value, metadata))
    }

    fn vote_witness_transaction_data(&self, contract_value: &ContractParameterValue) -> Option<(TransactionType, String, BigUint, Option<Value>)> {
        let vote = contract_value.votes.as_ref()?.first()?;
        let to = TronAddress::from_hex(vote.vote_address.as_str())?.encode();
        let value = BigUint::from(vote.vote_count.checked_mul(1_000_000)?);
        Some((TransactionType::StakeDelegate, to, value, None))
    }

    fn map_trigger_smart_contract(&self, contract_value: &ContractParameterValue, logs: &[TronLog], internal_transactions: &[InternalTransaction]) -> Option<Transaction> {
        self.map_token_approval(contract_value)
            .or_else(|| self.map_gasfree_transfer(contract_value, logs))
            .or_else(|| self.map_swap(contract_value, logs, internal_transactions))
            .or_else(|| self.map_token_transfer(contract_value, logs))
            .or_else(|| self.map_contract_call(contract_value))
    }

    fn map_token_approval(&self, contract_value: &ContractParameterValue) -> Option<Transaction> {
        let token_id = contract_value.contract_address.as_ref()?;
        let approval = trc20::decode_approval_hex(contract_value.data.as_deref()?)?;

        Some(self.build_transaction(AssetId::from_token(self.chain, token_id), self.from.clone(), approval.spender.encode(), TransactionType::TokenApproval, approval.value, None))
    }

    fn map_swap(&self, contract_value: &ContractParameterValue, logs: &[TronLog], internal_transactions: &[InternalTransaction]) -> Option<Transaction> {
        let owner = TronAddress::from_hex_or_base58(&self.from)?;
        let swap = tron_swap_metadata(self.chain, &owner, contract_value.contract_address.as_deref(), contract_value.call_value, logs, internal_transactions)?;

        Some(Transaction {
            contract: contract_value.contract_address.clone(),
            ..self.build_transaction(swap.from_asset.clone(), self.from.clone(), self.from.clone(), TransactionType::Swap, swap.from_value.clone(), serde_json::to_value(&swap).ok())
        })
    }

    fn map_token_transfer(&self, contract_value: &ContractParameterValue, logs: &[TronLog]) -> Option<Transaction> {
        if logs.len() != 1 {
            return None;
        }

        let (_, from, to, value) = decode_token_transfer(logs.first()?)?;
        let asset_id = AssetId::from_token(self.chain, contract_value.contract_address.as_ref()?);

        Some(self.build_transaction(asset_id, from.encode(), to.encode(), TransactionType::Transfer, value, None))
    }

    fn map_contract_call(&self, contract_value: &ContractParameterValue) -> Option<Transaction> {
        let contract = contract_value.contract_address.clone()?;
        let data = contract_value.data.as_ref()?;
        let value = BigUint::from(contract_value.call_value.unwrap_or_default());

        Some(
            self.build_transaction(self.fee_asset_id.clone(), self.from.clone(), contract, TransactionType::SmartContractCall, value, None)
                .with_data(Some(format!("0x{data}"))),
        )
    }

    fn map_gasfree_transfer(&self, contract_value: &ContractParameterValue, logs: &[TronLog]) -> Option<Transaction> {
        let (token, receiver, value) = trc20::decode_gasfree_permit_transfer_hex(contract_value.data.as_deref()?)?;
        let (transfer_index, from) = logs.iter().enumerate().find_map(|(index, log)| {
            let (transferred_token, from, to, transferred_value) = decode_token_transfer(log)?;
            (transferred_token == Some(token) && to == receiver && transferred_value == value).then_some((index, from))
        })?;
        let fee = logs
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != transfer_index)
            .filter_map(|(_, log)| decode_token_transfer(log))
            .filter(|(fee_token, fee_from, _, _)| *fee_token == Some(token) && *fee_from == from)
            .map(|(_, _, _, value)| value)
            .sum::<BigUint>();
        let asset_id = AssetId::from_token(self.chain, &token.encode());

        Some(Transaction {
            fee,
            fee_asset_id: asset_id.clone(),
            ..self.build_transaction(asset_id, from.encode(), receiver.encode(), TransactionType::Transfer, value, None)
        })
    }

    fn build_transaction(&self, asset_id: AssetId, from: String, to: String, transaction_type: TransactionType, value: BigUint, metadata: Option<Value>) -> Transaction {
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
    if contract_ret == "SUCCESS" { TransactionState::Confirmed } else { TransactionState::Failed }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BlockTransactions, TransactionReceipt, TransactionReceiptData, TriggerConstantContractResponse, TronContractType, TronLog, TronTransactionBroadcast};
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
        let response = TronTransactionBroadcast { txid: None, code: None, message: None };

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
            receipt: TransactionReceipt { result: Some("SUCCESS".to_string()) },
            log: None,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeFreeze);
        assert_eq!(transaction.value, BigUint::from(100000000u64));
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
            receipt: TransactionReceipt { result: Some("SUCCESS".to_string()) },
            log: None,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeFreeze);
        assert_eq!(transaction.value, BigUint::from(10000000u64));
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
            receipt: TransactionReceipt { result: Some("SUCCESS".to_string()) },
            log: None,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(transaction.value, BigUint::from(2125000000u64));
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
            receipt: TransactionReceipt { result: Some("SUCCESS".to_string()) },
            log: None,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::StakeUnfreeze);
        assert_eq!(transaction.value, BigUint::from(100000000u64));
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
        assert_eq!(transaction.hash(), TEST_TRANSACTION_ID);
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, BigUint::from(25000000u64));
        assert_ne!(transaction.from, transaction.to);
    }

    #[test]
    fn test_map_transaction_contract_call() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_relay_deposit.json")).unwrap();
        let receipt: TransactionReceiptData = serde_json::from_str(include_str!("../../testdata/transaction_relay_deposit_receipt.json")).unwrap();

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(transaction.asset_id, AssetId::from_chain(Chain::Tron));
        assert_eq!(transaction.from, "THagQ7ZgwYwms18ytALb2F5TvqywGVNic2");
        assert_eq!(transaction.to, "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN");
        assert_eq!(transaction.value, BigUint::from(1534064u64));
        assert_eq!(
            transaction.data.as_deref(),
            Some("0x49290c1c000000000000000000000000537e4f36cf498913f26942a71b59c8db188b880b57e85acead26d5fd307f5074d2d53e05311137cd85a6fc2b20a49d8b7f037757")
        );
    }

    #[test]
    fn test_map_transaction_okx_swap_referral_fee() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_okx_swap.json")).unwrap();
        let receipt: TransactionReceiptData = serde_json::from_str(include_str!("../../testdata/transaction_okx_swap_receipt.json")).unwrap();

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();
        let metadata = transaction.swap_metadata().unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(metadata.provider, Some(SwapProvider::Okx.id().to_string()));
        assert_eq!(metadata.from_asset, Chain::Tron.as_asset_id());
        assert_eq!(metadata.from_value, BigUint::from(220000000u64));
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Tron, "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"));
        assert_eq!(metadata.to_value, BigUint::from(72857124u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: Chain::Tron.as_asset_id(),
                value: BigUint::from(1540000u64),
            })
        );
    }

    #[test]
    fn test_map_transaction_sunswap_swap() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_sunswap_swap.json")).unwrap();
        let receipt: TransactionReceiptData = serde_json::from_str(include_str!("../../testdata/transaction_sunswap_swap_receipt.json")).unwrap();
        let logs = receipt.log.clone().unwrap();

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();
        let metadata = transaction.swap_metadata().unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(metadata.provider, Some(SwapProvider::SunswapV3.id().to_string()));
        assert_eq!(metadata.from_asset, Chain::Tron.as_asset_id());
        assert_eq!(metadata.from_value, BigUint::from(23_000_000u64));
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(metadata.to_value, BigUint::from(7_617_087u64));
        assert_eq!(metadata.referral_fee, None);
        assert_eq!(
            sunswap_referral_fee(Chain::Tron, &TronAddress::from_hex("df76f8a341abe1a06627eb01f7b0b4520ecb2c10").unwrap(), &logs),
            Some(TransactionSwapReferralFee {
                asset_id: Chain::Tron.as_asset_id(),
                value: BigUint::from(34_500u64),
            })
        );
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
            receipt: TransactionReceipt { result: Some("SUCCESS".to_string()) },
            log: Some(vec![TronLog::mock_transfer(
                TRON_USDT_TOKEN_ID,
                "0000000000000000000000002e1d447fa4169390cf5f5b3d12d380decfbfe20f",
                "0000000000000000000000006e2cf2878020b966786f01ab45ea1fcef6880092",
                "00000000000000000000000000000000000000000000000000000000017d7840",
            )]),
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.from, "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb");
        assert_eq!(transaction.to, "TL1mBABLk68nZwt92dgGc43p5uk1j3YtY9");
        assert_eq!(transaction.value, BigUint::from(25000000u64));
        assert_eq!(transaction.fee, BigUint::from(1000u64));
        assert_eq!(transaction.fee_asset_id, Chain::Tron.as_asset_id());
    }

    #[test]
    fn test_map_transaction_incoming_token_transfer() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_incoming_token_transfer.json")).unwrap();
        let receipt: TransactionReceiptData = serde_json::from_str(include_str!("../../testdata/transaction_incoming_token_transfer_receipt.json")).unwrap();

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();

        assert_eq!(transaction.hash(), "d61c72e80f48b6d014301fc088f5d7f5a512d08b17cc6208d6b9993aeea12b0a");
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.from, "TWn9ecrNEPY8qczxRoEsHxi5oac4aVVVDa");
        assert_eq!(transaction.to, "TYwzBzDWtLNT3xvhmBW2NSEFA9cZjt5ccC");
        assert_eq!(transaction.value, BigUint::from(10894000000u64));
    }

    #[test]
    fn test_map_transaction_gasfree_transfer() {
        let transaction: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_gasfree_transfer.json")).unwrap();
        let receipt: TransactionReceiptData = serde_json::from_str(include_str!("../../testdata/transaction_gasfree_transfer_receipt.json")).unwrap();
        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();

        assert_eq!(transaction.hash(), "81f98d55de44617532d28fa8449f1b9a47952a55637e32d93c54e020b00b64db");
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.from, "TELzUTyXVWaEpnfmaMm6gUghqmL5EyY2kY");
        assert_eq!(transaction.to, "TL6NY1hGXH2v6pF9fxtj1Fg6Ax5MzBMw5M");
        assert_eq!(transaction.value, BigUint::from(4653000000u64));
        assert_eq!(transaction.fee, BigUint::from(1500000u64));
        assert_eq!(transaction.fee_asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));

        let activation: TronTransaction = serde_json::from_str(include_str!("../../testdata/transaction_gasfree_activation.json")).unwrap();
        let activation_receipt: TransactionReceiptData = serde_json::from_str(include_str!("../../testdata/transaction_gasfree_activation_receipt.json")).unwrap();
        let activation = map_transaction(Chain::Tron, activation, activation_receipt).unwrap();

        assert_eq!(activation.hash(), "097fcb9ad088f99c102b656ef5a2b45b4c3b9f78b46729c087e45c42aa34805c");
        assert_eq!(activation.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(activation.transaction_type, TransactionType::Transfer);
        assert_eq!(activation.from, "TG54m1MyZ4iNXf4scDSc5Wg6uEJC2VMUNo");
        assert_eq!(activation.to, "TQD9VCkck55CrsH26dz9QHMp2exDiA4ua7");
        assert_eq!(activation.value, BigUint::from(5680000000u64));
        assert_eq!(activation.fee, BigUint::from(3000000u64));
        assert_eq!(activation.fee_asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
    }

    #[test]
    fn test_map_transaction_token_approval() {
        let failed = map_transaction(Chain::Tron, TronTransaction::mock_token_approval("OUT_OF_ENERGY"), TransactionReceiptData::mock_with_result("OUT_OF_ENERGY")).unwrap();

        assert_eq!(failed.hash(), TEST_TOKEN_APPROVAL_TRANSACTION_ID);
        assert_eq!(failed.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(failed.from, "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7");
        assert_eq!(failed.to, "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7");
        assert_eq!(failed.value, BigUint::from(0u64));
        assert_eq!(failed.transaction_type, TransactionType::TokenApproval);
        assert_eq!(failed.state, TransactionState::Failed);

        let confirmed = map_transaction(Chain::Tron, TronTransaction::mock_token_approval("SUCCESS"), TransactionReceiptData::mock_with_result("SUCCESS")).unwrap();

        assert_eq!(confirmed.transaction_type, TransactionType::TokenApproval);
        assert_eq!(confirmed.state, TransactionState::Confirmed);

        let increase: TronTransaction = serde_json::from_str(&include_str!("../../testdata/transaction_token_approval.json").replace("095ea7b3", "d73dd623")).unwrap();
        let increase = map_transaction(Chain::Tron, increase, TransactionReceiptData::mock_with_result("SUCCESS")).unwrap();

        assert_eq!(increase.transaction_type, TransactionType::TokenApproval);
        assert_eq!(increase.to, "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7");
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
            receipt: TransactionReceipt { result: Some("SUCCESS".to_string()) },
            log: swap_logs.logs,
            internal_transactions: None,
        };

        let result = map_transaction(Chain::Tron, transaction, receipt);
        assert!(result.is_some());
        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.contract.as_deref(), Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"));
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.asset_id, Chain::Tron.as_asset_id());
        assert_eq!(transaction.value, BigUint::from(1000000u64));
        let output_token = TronAddress::from_hex("4e4bee11cea0070f957b98fd8cf4138ef3295e0e").unwrap().encode();
        let metadata = transaction.swap_metadata().unwrap();
        assert_eq!(metadata.from_asset, Chain::Tron.as_asset_id());
        assert_eq!(metadata.from_value, BigUint::from(1000000u64));
        assert_eq!(
            metadata.to_asset,
            AssetId {
                chain: Chain::Tron,
                token_id: Some(output_token)
            }
        );
        assert_eq!(metadata.to_value, BigUint::from(329114u64));
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
        let trx_unwrap_in = InternalTransaction::mock("416e2cf2878020b966786f01ab45ea1fcef6880092", "412e1d447fa4169390cf5f5b3d12d380decfbfe20f", 900_000, None, false);
        let receipt = TransactionReceiptData {
            log: Some(vec![usdt_transfer_out]),
            internal_transactions: Some(vec![trx_unwrap_in]),
            ..TransactionReceiptData::mock_with_result("SUCCESS")
        };

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.contract.as_deref(), Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"));

        let metadata = transaction.swap_metadata().unwrap();
        let usdt = TronAddress::from_hex("41a614f803b6fd780986a42c78ec9c7f77e6ded13c").unwrap().encode();
        assert_eq!(metadata.from_asset, AssetId { chain: Chain::Tron, token_id: Some(usdt) });
        assert_eq!(metadata.from_value, BigUint::from(25000000u64));
        assert_eq!(metadata.to_asset, Chain::Tron.as_asset_id());
        assert_eq!(metadata.to_value, BigUint::from(900000u64));
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
        assert_eq!(transaction.hash(), "10f1e5b04c0dd39f14d4b5ca270899b36ae9c52ac1b9b64b76360c7373cc0893");
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID));
        assert_eq!(transaction.from, "TWBPGLwQw2EbqYLLw1DJnTDt2ZQ9yJW1JJ");
        assert_eq!(transaction.to, "TViSMURdt2dda6Pf163UBZoSfbV9hECvvc");
        assert_eq!(transaction.value, BigUint::from(249000000u64));
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
            receipt: TransactionReceipt { result: Some("SUCCESS".to_string()) },
            log: None,
            internal_transactions: None,
        };

        let transaction = map_transaction(Chain::Tron, transaction, receipt).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, BigUint::from(200000000u64));
        assert_eq!(transaction.memo.as_deref(), Some("=:TRON.USDT:TNAwd1WFe7GHTxovGU9MeT6mi3J4KAZMvP:0/1/0:g1:50"));
    }
}
