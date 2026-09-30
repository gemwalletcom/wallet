use crate::address::AccountAddress;
use crate::models::{DelegationPoolAddStakeData, DelegationPoolUnlockStakeData, Event, IndexerTransaction, Transaction, TransactionResponse};
use crate::{APTOS_NATIVE_COIN, DELEGATION_POOL_ADD_STAKE_EVENT, DELEGATION_POOL_UNLOCK_STAKE_EVENT, FUNGIBLE_ASSET_DEPOSIT_EVENT, FUNGIBLE_ASSET_WITHDRAW_EVENT, STAKE_DEPOSIT_EVENT};
use chain_primitives::{BalanceDiff, SwapMapper};
use chrono::{DateTime, NaiveDateTime};
use num_bigint::{BigInt, BigUint};
use primitives::{AssetId, Chain, SwapProvider, Transaction as PrimitivesTransaction, TransactionState, TransactionSwapReferralFee, TransactionType, swap::APTOS_REFERRAL_ADDRESS};
use std::collections::HashMap;
use std::error::Error;

const PANORA_SWAP_EVENT: &str = "panora_swap";
const PANORA_SWAP_EVENT_ADDRESS: &str = "0x1c3206329806286fd2223647c9f9b130e66baeb6d7224a18c1f642ffe48f3b4c";
const PANORA_SWAP_SUMMARY_EVENT: &str = "PanoraSwapSummaryEvent";
const PANORA_FEE_INTEGRATOR_EVENT: &str = "FeeEventIntegrator";
const APTOS_NATIVE_METADATA_ADDRESS: u8 = 0x0a;
const INDEXER_TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f";
const FUNGIBLE_ASSET_DEPOSIT_ACTIVITY: &str = "0x1::fungible_asset::Deposit";
const FUNGIBLE_ASSET_WITHDRAW_ACTIVITY: &str = "0x1::fungible_asset::Withdraw";

#[derive(serde::Deserialize)]
struct PanoraSwapSummaryEventData {
    input_token_address: String,
    #[serde(deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    input_token_amount: BigUint,
    output_token_address: String,
    #[serde(deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    output_token_amount: BigUint,
}

#[derive(serde::Deserialize)]
struct PanoraFeeIntegratorEventData {
    integrator_address: String,
    token_address: String,
    #[serde(deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    token_amount: BigUint,
}

fn map_referral_fee(events: &[Event], chain: Chain, sender: &str) -> Option<TransactionSwapReferralFee> {
    if sender == APTOS_REFERRAL_ADDRESS {
        return None;
    }
    events
        .iter()
        .filter(|event| event.event_type.contains(PANORA_SWAP_EVENT_ADDRESS) && event.event_type.contains(PANORA_FEE_INTEGRATOR_EVENT))
        .filter_map(|event| serde_json::from_value::<PanoraFeeIntegratorEventData>(event.data.clone()?).ok())
        .find(|data| data.integrator_address == APTOS_REFERRAL_ADDRESS)
        .map(|data| TransactionSwapReferralFee {
            asset_id: map_token_address_to_asset_id(chain, &data.token_address),
            value: data.token_amount,
        })
}

fn map_token_address_to_asset_id(chain: Chain, token_address: &str) -> AssetId {
    if token_address == APTOS_NATIVE_COIN || AccountAddress::from_hex(token_address).ok() == AccountAddress::from_bytes(&[APTOS_NATIVE_METADATA_ADDRESS]).ok() {
        chain.as_asset_id()
    } else {
        AssetId::from_token(chain, token_address)
    }
}

pub fn map_transaction_broadcast(response: &TransactionResponse) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(message) = &response.message {
        return Err(message.clone().into());
    }

    response.hash.clone().ok_or_else(|| "Transaction response missing hash".into())
}

pub fn map_transactions(transactions: Vec<Transaction>) -> Vec<PrimitivesTransaction> {
    let mut transactions = transactions.into_iter().flat_map(map_transaction).collect::<Vec<_>>();

    transactions.sort_by_key(|b| std::cmp::Reverse(b.created_at));
    transactions
}

struct TransactionMeta {
    hash: String,
    sender: String,
    state: TransactionState,
    fee: BigUint,
    created_at: DateTime<chrono::Utc>,
}

fn extract_meta(transaction: &Transaction) -> Option<TransactionMeta> {
    let hash = transaction.hash.clone().unwrap_or_default();
    let sender = transaction.sender.clone().unwrap_or_default();
    let state = if transaction.success { TransactionState::Confirmed } else { TransactionState::Failed };
    let gas_used = BigUint::from(transaction.gas_used.unwrap_or_default());
    let gas_unit_price = BigUint::from(transaction.gas_unit_price.unwrap_or_default());
    let fee = gas_used * gas_unit_price;
    let created_at = DateTime::from_timestamp_micros(transaction.timestamp as i64)?;

    Some(TransactionMeta { hash, sender, state, fee, created_at })
}

fn called_contract(transaction: &Transaction) -> Option<String> {
    transaction.payload.as_ref()?.function.as_deref()?.split("::").next().filter(|address| !address.is_empty()).map(str::to_string)
}

fn map_swap_transaction(transaction: Transaction, events: Vec<Event>, chain: Chain) -> Option<PrimitivesTransaction> {
    let meta = extract_meta(&transaction)?;
    let contract = called_contract(&transaction);
    let referral_fee = map_referral_fee(&events, chain, &meta.sender);

    if let Some(summary) = events
        .iter()
        .find(|event| event.event_type.contains(PANORA_SWAP_EVENT_ADDRESS) && event.event_type.contains(PANORA_SWAP_SUMMARY_EVENT))
        .and_then(|event| event.data.clone())
        .and_then(|data| serde_json::from_value::<PanoraSwapSummaryEventData>(data).ok())
    {
        let from_asset = map_token_address_to_asset_id(chain, &summary.input_token_address);
        let to_asset = map_token_address_to_asset_id(chain, &summary.output_token_address);

        let balance_diffs = vec![
            BalanceDiff {
                asset_id: from_asset,
                diff: -BigInt::from(summary.input_token_amount),
            },
            BalanceDiff {
                asset_id: to_asset,
                diff: BigInt::from(summary.output_token_amount),
            },
        ];

        let swap = SwapMapper::map_swap(&balance_diffs, &BigUint::from(0u8), &chain.as_asset_id(), Some(SwapProvider::Panora))?.with_referral_fee(referral_fee);
        let asset_id = swap.from_asset.clone();
        let metadata = serde_json::to_value(&swap).ok();
        let to = meta.sender.clone();

        return Some(PrimitivesTransaction {
            contract,
            ..build_transaction(meta, asset_id, chain.as_asset_id(), to, swap.from_value, TransactionType::Swap, metadata)
        });
    }

    let withdraw_event = events.iter().find(|event| event.event_type == FUNGIBLE_ASSET_WITHDRAW_EVENT)?;
    let deposit_event = events.iter().find(|event| event.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT)?;
    let withdraw_amount = withdraw_event.get_amount()?;
    let deposit_amount = deposit_event.get_amount()?;

    let type_args = transaction.payload.as_ref()?.type_arguments.clone();
    if type_args.len() != 2 {
        return None;
    }

    let map_asset = |coin_type: &str| {
        if coin_type == APTOS_NATIVE_COIN { chain.as_asset_id() } else { AssetId::from_token(chain, coin_type) }
    };

    let from_asset = map_asset(&type_args[0]);
    let to_asset = map_asset(&type_args[1]);

    let balance_diffs = vec![
        BalanceDiff {
            asset_id: from_asset,
            diff: -BigInt::from(withdraw_amount),
        },
        BalanceDiff {
            asset_id: to_asset,
            diff: BigInt::from(deposit_amount),
        },
    ];

    let provider = events
        .iter()
        .find(|event| event.event_type.contains(PANORA_SWAP_EVENT))
        .and_then(|event| if event.event_type.contains(PANORA_SWAP_EVENT_ADDRESS) { Some(SwapProvider::Panora) } else { None });

    let swap = SwapMapper::map_swap(&balance_diffs, &BigUint::from(0u8), &chain.as_asset_id(), provider)?.with_referral_fee(referral_fee);
    let asset_id = swap.from_asset.clone();
    let metadata = serde_json::to_value(&swap).ok();
    let to = meta.sender.clone();

    Some(PrimitivesTransaction {
        contract,
        ..build_transaction(meta, asset_id, chain.as_asset_id(), to, swap.from_value, TransactionType::Swap, metadata)
    })
}

pub fn map_indexer_transaction(hash: String, transaction: IndexerTransaction) -> Option<PrimitivesTransaction> {
    let chain = Chain::Aptos;
    let user_transaction = transaction.user_transactions.first()?;
    let activities = &transaction.fungible_asset_activities;
    let sender = AccountAddress::from_hex(&user_transaction.sender).ok()?;
    let is_owner = |owner: &Option<String>, address: &AccountAddress| owner.as_deref().and_then(|owner| AccountAddress::from_hex(owner).ok()).as_ref() == Some(address);
    let referral_address = AccountAddress::from_hex(APTOS_REFERRAL_ADDRESS).ok()?;

    let fee: u64 = activities.iter().filter(|activity| activity.is_gas_fee).filter_map(|activity| activity.amount).sum();
    let mut balance_changes: HashMap<AssetId, BigInt> = HashMap::new();
    for activity in activities.iter().filter(|activity| !activity.is_gas_fee && is_owner(&activity.owner_address, &sender)) {
        let asset_id = map_token_address_to_asset_id(chain, activity.asset_type.as_deref()?);
        let amount = BigInt::from(activity.amount?);
        let change = match activity.activity_type.as_str() {
            FUNGIBLE_ASSET_DEPOSIT_ACTIVITY => amount,
            FUNGIBLE_ASSET_WITHDRAW_ACTIVITY => -amount,
            _ => continue,
        };
        *balance_changes.entry(asset_id).or_default() += change;
    }
    let balance_diffs = balance_changes.into_iter().map(|(asset_id, diff)| BalanceDiff { asset_id, diff }).collect::<Vec<_>>();
    let referral_fee = activities
        .iter()
        .find(|activity| activity.activity_type == FUNGIBLE_ASSET_DEPOSIT_ACTIVITY && is_owner(&activity.owner_address, &referral_address))
        .and_then(|activity| {
            Some(TransactionSwapReferralFee {
                asset_id: map_token_address_to_asset_id(chain, activity.asset_type.as_deref()?),
                value: BigUint::from(activity.amount?),
            })
        });
    let contract = user_transaction.entry_function_id_str.as_deref().and_then(|function| function.split("::").next()).map(str::to_string);
    let provider = contract.as_deref().filter(|contract| *contract == PANORA_SWAP_EVENT_ADDRESS).map(|_| SwapProvider::Panora);
    let swap = SwapMapper::map_swap(&balance_diffs, &BigUint::from(0u8), &chain.as_asset_id(), provider)?.with_referral_fee(referral_fee);

    let meta = TransactionMeta {
        hash,
        sender: user_transaction.sender.clone(),
        state: if activities.iter().all(|activity| activity.is_transaction_success) {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        },
        fee: BigUint::from(fee),
        created_at: NaiveDateTime::parse_from_str(&user_transaction.timestamp, INDEXER_TIMESTAMP_FORMAT).ok()?.and_utc(),
    };
    let asset_id = swap.from_asset.clone();
    let metadata = serde_json::to_value(&swap).ok();
    let to = meta.sender.clone();

    Some(PrimitivesTransaction {
        contract,
        ..build_transaction(meta, asset_id, chain.as_asset_id(), to, swap.from_value, TransactionType::Swap, metadata)
    })
}

fn build_transaction(meta: TransactionMeta, asset_id: AssetId, fee_asset_id: AssetId, to: String, value: BigUint, transaction_type: TransactionType, metadata: Option<serde_json::Value>) -> PrimitivesTransaction {
    PrimitivesTransaction::new(meta.hash, asset_id, meta.sender, to, None, transaction_type, meta.state, meta.fee, fee_asset_id, value, None, metadata, meta.created_at)
}

pub fn map_transaction(transaction: Transaction) -> Option<PrimitivesTransaction> {
    let chain = Chain::Aptos;
    let events = transaction.clone().events.unwrap_or_default();
    let meta = extract_meta(&transaction)?;
    let asset_id = chain.as_asset_id();

    if events.iter().any(|event| event.event_type.contains("Swap")) {
        return map_swap_transaction(transaction, events, chain);
    }

    for event in &events {
        match event.event_type.as_str() {
            DELEGATION_POOL_ADD_STAKE_EVENT => {
                let data: DelegationPoolAddStakeData = serde_json::from_value(event.data.clone()?).ok()?;
                return Some(build_transaction(meta, asset_id.clone(), asset_id, data.pool_address, data.amount_added, TransactionType::StakeDelegate, None));
            }
            DELEGATION_POOL_UNLOCK_STAKE_EVENT => {
                let data: DelegationPoolUnlockStakeData = serde_json::from_value(event.data.clone()?).ok()?;
                return Some(build_transaction(meta, asset_id.clone(), asset_id, data.pool_address, data.amount_unlocked, TransactionType::StakeUndelegate, None));
            }
            _ => continue,
        }
    }

    if transaction.transaction_type.as_deref() == Some("user_transaction") && events.len() <= 4 {
        let deposit_event = events.iter().find(|x| x.event_type == STAKE_DEPOSIT_EVENT || x.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT)?;

        let to = if deposit_event.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT {
            transaction.payload.as_ref()?.arguments.first()?.as_str()?.to_string()
        } else {
            deposit_event.guid.account_address.clone()
        };

        let value = deposit_event.get_amount()?;

        return Some(build_transaction(meta, asset_id.clone(), asset_id, to, value, TransactionType::Transfer, None));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionResponse;
    use crate::provider::testkit::TEST_TRANSACTION_ID;
    use primitives::asset_constants::APTOS_USDT_TOKEN_ID;

    #[test]
    fn test_map_indexer_transaction_panora_swap() {
        let response: primitives::graphql::GraphqlData<IndexerTransaction> = serde_json::from_str(include_str!("../../testdata/indexer_transaction_panora_swap.json")).unwrap();
        let transaction = map_indexer_transaction("0xhash".to_string(), response.data.unwrap()).unwrap();
        let metadata = transaction.swap_metadata().unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.from, "0xe65f9bd40d97937a92e5e4d67dfe29c47bb29e39ecb53ea30ebc4f748712d4bf");
        assert_eq!(transaction.fee, BigUint::from(1873500u64));
        assert_eq!(metadata.provider.as_deref(), Some("panora"));
        assert_eq!(metadata.from_asset, Chain::Aptos.as_asset_id());
        assert_eq!(metadata.from_value, BigUint::from(1300000000u64));
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Aptos, APTOS_USDT_TOKEN_ID));
        assert_eq!(metadata.to_value, BigUint::from(7368961u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: Chain::Aptos.as_asset_id(),
                value: BigUint::from(3250648u64),
            })
        );
    }

    #[test]
    fn test_map_transaction_broadcast() {
        let response = TransactionResponse {
            hash: Some("0xabc123".to_string()),
            message: None,
            error_code: None,
            vm_error_code: None,
        };

        let result = map_transaction_broadcast(&response).unwrap();
        assert_eq!(result, "0xabc123");
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response = TransactionResponse {
            hash: None,
            message: Some("Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS".to_string()),
            error_code: Some("vm_error".to_string()),
            vm_error_code: Some(14),
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS");
    }

    #[test]
    fn test_map_transaction_broadcast_from_testdata() {
        let response: TransactionResponse = serde_json::from_str(include_str!("../../testdata/invalid_transaction_response.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS");
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_near_intent_transfer.json")).unwrap();

        let result = map_transaction(transaction);

        assert!(result.is_some());
        let mapped = result.unwrap();
        assert_eq!(mapped.hash(), TEST_TRANSACTION_ID);
        assert_eq!(mapped.id.to_string(), format!("aptos_{TEST_TRANSACTION_ID}"));
        assert_eq!(mapped.from, "0xd1a1c1804e91ba85a569c7f018bb7502d2f13d4742d2611953c9c14681af6446");
        assert_eq!(mapped.to, "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc");
        assert_eq!(mapped.value, BigUint::from(2431838058u64));
        assert_eq!(mapped.state, TransactionState::Confirmed);
        assert_eq!(mapped.transaction_type, TransactionType::Transfer);
    }

    #[test]
    fn test_map_transaction_swap_panora() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_swap_panora.json")).unwrap();

        let result = map_transaction(transaction);

        assert!(result.is_some());
        let mapped = result.unwrap();
        assert_eq!(mapped.id.to_string(), "aptos_0xf1c24162c08b6b8b452c00adad1836d72949902a8701611479d4e49fec0a9e3c");
        assert_eq!(mapped.from, "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef");
        assert_eq!(mapped.to, "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef");
        assert_eq!(mapped.state, TransactionState::Confirmed);
        assert_eq!(mapped.transaction_type, TransactionType::Swap);
        assert_eq!(mapped.contract.as_deref(), Some("0x1c3206329806286fd2223647c9f9b130e66baeb6d7224a18c1f642ffe48f3b4c"));
        assert_eq!(mapped.asset_id, AssetId::from_token(Chain::Aptos, APTOS_USDT_TOKEN_ID));
        assert_eq!(mapped.fee_asset_id, Chain::Aptos.as_asset_id());
        assert_eq!(mapped.fee, BigUint::from(142_600u32));
        assert!(mapped.metadata.is_some());

        let metadata: primitives::TransactionSwapMetadata = serde_json::from_value(mapped.metadata.unwrap()).unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::Aptos, APTOS_USDT_TOKEN_ID));
        assert_eq!(metadata.from_value, BigUint::from(2346314u64));
        assert_eq!(metadata.to_asset, Chain::Aptos.as_asset_id());
        assert_eq!(metadata.to_value, BigUint::from(120590251u64));
        assert_eq!(metadata.provider.unwrap(), "panora");
        assert_eq!(metadata.referral_fee, None);
    }

    #[test]
    fn test_map_transaction_swap_panora_referral_fee() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_swap_panora_referral_fee.json")).unwrap();

        let mapped = map_transaction(transaction).unwrap();
        let metadata: primitives::TransactionSwapMetadata = serde_json::from_value(mapped.metadata.unwrap()).unwrap();

        assert_eq!(mapped.transaction_type, TransactionType::Swap);
        assert_eq!(metadata.from_asset, Chain::Aptos.as_asset_id());
        assert_eq!(metadata.from_value, BigUint::from(500000000u64));
        assert_eq!(metadata.to_value, BigUint::from(3782787u64));
        assert_eq!(
            metadata.referral_fee,
            Some(TransactionSwapReferralFee {
                asset_id: Chain::Aptos.as_asset_id(),
                value: BigUint::from(1249510u64),
            })
        );
    }

    #[test]
    fn test_map_transaction_stake_delegate() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_stake_delegate.json")).unwrap();

        let result = map_transaction(transaction);

        assert!(result.is_some());
        let mapped = result.unwrap();
        assert_eq!(mapped.id.to_string(), "aptos_0x130cc74c1a768780ca062a97bc833a01dec85b2d315484869559b7cdee4d0e75");
        assert_eq!(mapped.from, "0xc95615aa095c100b18eb6eaa0f0a0f30b9cd96685118a7cbc1a2328a91ca2eda");
        assert_eq!(mapped.to, "0xe5452230b8d5f4a664e33b8ad95354e50da64caaf003f11c0158391e96a4db2c");
        assert_eq!(mapped.value, BigUint::from(1100000000u64));
        assert_eq!(mapped.state, TransactionState::Confirmed);
        assert_eq!(mapped.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(mapped.fee, BigUint::from(142400u64));
    }

    #[test]
    fn test_map_transaction_stake_undelegate() {
        let transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_stake_undelegate.json")).unwrap();

        let result = map_transaction(transaction);

        assert!(result.is_some());
        let mapped = result.unwrap();
        assert_eq!(mapped.id.to_string(), "aptos_0xef6430bef0e8de7090b2c4bce210adb75d648be4614dcc37232b0d67f819b137");
        assert_eq!(mapped.from, "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc");
        assert_eq!(mapped.to, "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e");
        assert_eq!(mapped.value, BigUint::from(1109984251u64));
        assert_eq!(mapped.state, TransactionState::Confirmed);
        assert_eq!(mapped.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(mapped.fee, BigUint::from(88400u64));
    }
}
