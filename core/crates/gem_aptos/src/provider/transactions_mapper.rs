use crate::models::{DelegationPoolAddStakeData, DelegationPoolUnlockStakeData, DelegationPoolWithdrawStakeData, Event, Transaction, TransactionResponse, fungible_store_change};
use crate::{
    APTOS_NATIVE_COIN, APTOS_NATIVE_METADATA_ADDRESS, AccountAddress, BLOCK_METADATA_TRANSACTION_TYPE, DELEGATION_POOL_ADD_STAKE_EVENT, DELEGATION_POOL_UNLOCK_STAKE_EVENT, DELEGATION_POOL_WITHDRAW_STAKE_EVENT, FUNGIBLE_ASSET_DEPOSIT_EVENT,
    FUNGIBLE_ASSET_WITHDRAW_EVENT, FUNGIBLE_BATCH_TRANSFER_FUNCTION, FUNGIBLE_TRANSFER_FUNCTION, HYPERION_CONTRACT, STAKE_DEPOSIT_EVENT,
};
use chain_primitives::{BalanceDiff, SwapMapper};
use num_bigint::{BigInt, BigUint};
use primitives::{AssetId, Chain, DateTime, SwapProvider, Transaction as PrimitivesTransaction, TransactionState, TransactionSwapReferralFee, TransactionType, swap::APTOS_REFERRAL_ADDRESS};
use std::error::Error;

const PANORA_SWAP_EVENT: &str = "panora_swap";
const PANORA_SWAP_EVENT_ADDRESS: &str = "0x1c3206329806286fd2223647c9f9b130e66baeb6d7224a18c1f642ffe48f3b4c";
const PANORA_SWAP_SUMMARY_EVENT: &str = "PanoraSwapSummaryEvent";
const PANORA_FEE_INTEGRATOR_EVENT: &str = "FeeEventIntegrator";

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
    if token_address == APTOS_NATIVE_METADATA_ADDRESS || token_address == APTOS_NATIVE_COIN {
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

pub fn map_block_range_transactions(transactions: Vec<Transaction>, versions: u64, blocks: u64) -> Result<Vec<Vec<Transaction>>, Box<dyn Error + Sync + Send>> {
    if transactions.len() as u64 != versions {
        return Err(format!("Aptos returned {} of {versions} versions", transactions.len()).into());
    }
    let range = transactions
        .chunk_by(|_, next| next.transaction_type.as_deref() != Some(BLOCK_METADATA_TRANSACTION_TYPE))
        .map(<[Transaction]>::to_vec)
        .collect::<Vec<_>>();
    if range.len() as u64 != blocks {
        return Err(format!("Aptos returned {} of {blocks} blocks", range.len()).into());
    }
    Ok(range)
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
    created_at: DateTime,
}

fn extract_meta(transaction: &Transaction) -> Option<TransactionMeta> {
    let hash = transaction.hash.clone().unwrap_or_default();
    let sender = transaction.sender.clone().unwrap_or_default();
    let state = if transaction.success { TransactionState::Confirmed } else { TransactionState::Failed };
    let gas_used = BigUint::from(transaction.gas_used.unwrap_or_default());
    let gas_unit_price = BigUint::from(transaction.gas_unit_price.unwrap_or_default());
    let fee = gas_used * gas_unit_price;
    Some(TransactionMeta {
        hash,
        sender,
        state,
        fee,
        created_at: transaction.timestamp,
    })
}

fn called_contract(transaction: &Transaction) -> Option<String> {
    transaction.payload.as_ref()?.function.as_deref()?.split("::").next().filter(|address| !address.is_empty()).map(str::to_string)
}

fn map_hyperion_swap(events: &[Event], chain: Chain, sender: &str) -> Option<(Vec<BalanceDiff>, Option<TransactionSwapReferralFee>)> {
    let pool_swaps = events.iter().filter_map(Event::hyperion_pool_swap).collect::<Vec<_>>();
    let token_in = pool_swaps.first()?.from_token.inner.clone();
    let token_out = pool_swaps.last()?.to_token.inner.clone();
    let owner = AccountAddress::from_hex(sender).ok()?;
    let input_store = owner.primary_store(&AccountAddress::from_hex(&token_in).ok()?);
    let output_store = owner.primary_store(&AccountAddress::from_hex(&token_out).ok()?);

    let referral_address = AccountAddress::from_hex(APTOS_REFERRAL_ADDRESS).ok()?;
    let referral_fee = events
        .iter()
        .filter_map(Event::hyperion_third_party_swap)
        .find(|data| owner != referral_address && AccountAddress::from_hex(&data.third_party_receiver).ok().as_ref() == Some(&referral_address))
        .map(|data| TransactionSwapReferralFee {
            asset_id: map_token_address_to_asset_id(chain, &data.token_in.inner),
            value: data.third_party_fee,
        });

    let balance_diffs = vec![
        BalanceDiff {
            asset_id: map_token_address_to_asset_id(chain, &token_in),
            diff: fungible_store_change(events, &input_store),
        },
        BalanceDiff {
            asset_id: map_token_address_to_asset_id(chain, &token_out),
            diff: fungible_store_change(events, &output_store),
        },
    ];
    Some((balance_diffs, referral_fee))
}

fn map_swap_transaction(transaction: Transaction, events: Vec<Event>, chain: Chain) -> Option<PrimitivesTransaction> {
    let meta = extract_meta(&transaction)?;
    let contract = called_contract(&transaction);
    if contract.as_deref() == Some(HYPERION_CONTRACT) {
        let (balance_diffs, referral_fee) = map_hyperion_swap(&events, chain, &meta.sender)?;
        return build_swap_transaction(meta, contract, chain, &balance_diffs, Some(SwapProvider::Hyperion), referral_fee);
    }
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

        return build_swap_transaction(meta, contract, chain, &balance_diffs, Some(SwapProvider::Panora), referral_fee);
    }

    let withdraw_event = events.iter().find(|event| event.event_type == FUNGIBLE_ASSET_WITHDRAW_EVENT)?;
    let deposit_event = events.iter().find(|event| event.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT)?;
    let withdraw_amount = withdraw_event.get_amount()?;
    let deposit_amount = deposit_event.get_amount()?;

    let type_args = transaction.payload.as_ref()?.type_arguments.clone();
    if type_args.len() != 2 {
        return None;
    }

    let from_asset = map_token_address_to_asset_id(chain, &type_args[0]);
    let to_asset = map_token_address_to_asset_id(chain, &type_args[1]);

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

    build_swap_transaction(meta, contract, chain, &balance_diffs, provider, referral_fee)
}

fn build_swap_transaction(meta: TransactionMeta, contract: Option<String>, chain: Chain, balance_diffs: &[BalanceDiff], provider: Option<SwapProvider>, referral_fee: Option<TransactionSwapReferralFee>) -> Option<PrimitivesTransaction> {
    let swap = SwapMapper::map_swap(balance_diffs, &BigUint::from(0u8), &chain.as_asset_id(), provider)?.with_referral_fee(referral_fee);
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

fn map_stake_transaction(transaction: &Transaction, events: &[Event], chain: Chain) -> Option<PrimitivesTransaction> {
    let meta = extract_meta(transaction)?;
    let asset_id = chain.as_asset_id();
    for event in events {
        match event.event_type.as_str() {
            DELEGATION_POOL_ADD_STAKE_EVENT => {
                let data: DelegationPoolAddStakeData = serde_json::from_value(event.data.clone()?).ok()?;
                return Some(build_transaction(meta, asset_id.clone(), asset_id, data.pool_address, data.amount_added, TransactionType::StakeDelegate, None));
            }
            DELEGATION_POOL_UNLOCK_STAKE_EVENT => {
                let data: DelegationPoolUnlockStakeData = serde_json::from_value(event.data.clone()?).ok()?;
                return Some(build_transaction(meta, asset_id.clone(), asset_id, data.pool_address, data.amount_unlocked, TransactionType::StakeUndelegate, None));
            }
            DELEGATION_POOL_WITHDRAW_STAKE_EVENT => {
                let data: DelegationPoolWithdrawStakeData = serde_json::from_value(event.data.clone()?).ok()?;
                return Some(build_transaction(meta, asset_id.clone(), asset_id, data.pool_address, data.amount_withdrawn, TransactionType::StakeWithdraw, None));
            }
            _ => {}
        }
    }
    None
}

fn map_transfer_recipients(transaction: &Transaction, events: &[Event], chain: Chain) -> Option<(AssetId, Vec<String>)> {
    if transaction.transaction_type.as_deref() != Some("user_transaction") {
        return None;
    }
    let payload = transaction.payload.as_ref()?;
    let token_asset_id = || Some(map_token_address_to_asset_id(chain, payload.arguments.first()?.get("inner")?.as_str()?));
    match payload.function.as_deref() {
        Some(FUNGIBLE_TRANSFER_FUNCTION) => Some((token_asset_id()?, vec![payload.arguments.get(1)?.as_str()?.to_string()])),
        Some(FUNGIBLE_BATCH_TRANSFER_FUNCTION) => Some((token_asset_id()?, serde_json::from_value(payload.arguments.get(1)?.clone()).ok()?)),
        _ if events.len() <= 4 => Some((chain.as_asset_id(), vec![payload.arguments.first()?.as_str()?.to_string()])),
        _ => None,
    }
}

pub fn map_transaction(transaction: Transaction) -> Vec<PrimitivesTransaction> {
    let chain = Chain::Aptos;
    let events = transaction.clone().events.unwrap_or_default();

    if events.iter().any(|event| event.event_type.contains("Swap")) {
        return map_swap_transaction(transaction, events, chain).into_iter().collect();
    }
    if let Some(stake) = map_stake_transaction(&transaction, &events, chain) {
        return vec![stake];
    }
    let Some((asset_id, recipients)) = map_transfer_recipients(&transaction, &events, chain) else {
        return vec![];
    };
    let deposits = events.iter().filter(|event| event.event_type == STAKE_DEPOSIT_EVENT || event.event_type == FUNGIBLE_ASSET_DEPOSIT_EVENT);
    recipients
        .into_iter()
        .zip(deposits)
        .filter_map(|(to, deposit)| {
            Some(build_transaction(
                extract_meta(&transaction)?,
                asset_id.clone(),
                chain.as_asset_id(),
                to,
                deposit.get_amount()?,
                TransactionType::Transfer,
                None,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionResponse;
    use crate::provider::testkit::TEST_TRANSACTION_ID;
    use primitives::asset_constants::APTOS_USDT_TOKEN_ID;

    fn transactions(json: &str) -> Vec<PrimitivesTransaction> {
        map_transaction(serde_json::from_str(json).unwrap())
    }

    fn transaction(json: &str) -> PrimitivesTransaction {
        let [transaction] = transactions(json).try_into().unwrap();
        transaction
    }

    #[test]
    fn test_map_block_range_transactions() {
        let transactions = ["block_metadata_transaction", "user_transaction", "block_epilogue_transaction", "block_metadata_transaction", "block_epilogue_transaction"]
            .map(|transaction_type| serde_json::from_value::<Transaction>(serde_json::json!({"type": transaction_type, "success": true})).unwrap())
            .to_vec();

        let blocks = map_block_range_transactions(transactions.clone(), 5, 2).unwrap();

        assert_eq!(blocks.iter().map(Vec::len).collect::<Vec<_>>(), vec![3, 2]);
        assert_eq!(map_block_range_transactions(transactions.clone(), 6, 2).unwrap_err().to_string(), "Aptos returned 5 of 6 versions");
        assert_eq!(map_block_range_transactions(transactions, 5, 3).unwrap_err().to_string(), "Aptos returned 2 of 3 blocks");
    }

    #[test]
    fn test_map_transaction_broadcast() {
        let response = TransactionResponse {
            hash: Some("0xabc123".to_string()),
            message: None,
            error_code: None,
            vm_error_code: None,
        };
        let invalid: TransactionResponse = serde_json::from_str(include_str!("../../testdata/invalid_transaction_response.json")).unwrap();

        assert_eq!(map_transaction_broadcast(&response).unwrap(), "0xabc123");
        assert_eq!(
            map_transaction_broadcast(&invalid).unwrap_err().to_string(),
            "Invalid transaction: Type: Validation Code: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS"
        );
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let mapped = transaction(include_str!("../../testdata/transaction_near_intent_transfer.json"));

        assert_eq!(mapped.hash(), TEST_TRANSACTION_ID);
        assert_eq!(mapped.id.to_string(), format!("aptos_{TEST_TRANSACTION_ID}"));
        assert_eq!(mapped.from, "0xd1a1c1804e91ba85a569c7f018bb7502d2f13d4742d2611953c9c14681af6446");
        assert_eq!(mapped.to, "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc");
        assert_eq!(mapped.value, BigUint::from(2431838058u64));
        assert_eq!(mapped.state, TransactionState::Confirmed);
        assert_eq!(mapped.transaction_type, TransactionType::Transfer);
    }

    #[test]
    fn test_map_transaction_fungible_asset_transfer() {
        assert_eq!(
            transaction(include_str!("../../testdata/transaction_fungible_asset_transfer.json")),
            PrimitivesTransaction::new(
                "0x42fb1bd38596b4f14d103e591fd631987e73fa9666d5d6dd05bed111a2dba0c6".to_string(),
                AssetId::from_token(Chain::Aptos, APTOS_USDT_TOKEN_ID),
                "0xd35f048a5d1de4299c4aa43e79758f8bb6047b8c7030faf680af4530c40c4ea7".to_string(),
                "0x51c6abe562e755582d268340b2cf0e2d8895a155dc9b7a7fb5465000d62d770b".to_string(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                BigUint::from(15_000u32),
                Chain::Aptos.as_asset_id(),
                BigUint::from(14_820_000u32),
                None,
                None,
                DateTime::from_timestamp_micros(1_789_429_930_321_207),
            )
        );
    }

    #[test]
    fn test_map_transaction_batch_fungible_asset_transfer() {
        let usdt = AssetId::from_token(Chain::Aptos, APTOS_USDT_TOKEN_ID);

        assert_eq!(
            transactions(include_str!("../../testdata/transaction_batch_fungible_asset_transfer.json"))
                .iter()
                .map(|transfer| (transfer.asset_id.clone(), transfer.to.as_str(), transfer.value.clone()))
                .collect::<Vec<_>>(),
            [
                ("0x868639e25f660315ced59988baad07a5d60007e981d2d593b7833a8645c405ed", 13_000_410u64),
                ("0x4b6ae525d90a098b2b8a713c7c16707c7856da2fc1b6886ea16e7f3b4417c654", 3_900_000),
                ("0xc22d90300d4a120122814068d8df4ab46e57ae0d860419a9b4ef1c9754971ead", 5_492_910),
                ("0xbd033343e300c2394df2328504ed407ef4ba412642c12d22d039d16c52ba2442", 6_900_000),
                ("0xc413089f62438a1fb0fdd210b5e335578d61e69dd8340ea9808272fefe35ce81", 6_399_830_000),
            ]
            .map(|(to, value)| (usdt.clone(), to, BigUint::from(value)))
        );
    }

    #[test]
    fn test_map_transaction_batch_native_transfer() {
        assert_eq!(
            transaction(include_str!("../../testdata/transaction_batch_native_transfer.json")),
            PrimitivesTransaction::new(
                "0xbb20d4d4a3877185036586f78beea39607cd480f8bb11e5c3ef5352f2d3865ae".to_string(),
                Chain::Aptos.as_asset_id(),
                "0x9577885192fe855857dfd699ba15ef89e7eac15f770f1d89565d7188339a7fbb".to_string(),
                "0xef63b80162a96a03127f35704bb9f3557bacb2d8327f1328e0274d3f2630d8fd".to_string(),
                None,
                TransactionType::Transfer,
                TransactionState::Confirmed,
                BigUint::from(10_100u32),
                Chain::Aptos.as_asset_id(),
                BigUint::from(4_367_727_073u64),
                None,
                None,
                DateTime::from_timestamp_micros(1_795_838_590_592_490),
            )
        );
    }

    #[test]
    fn test_map_transaction_swap_panora() {
        let mapped = transaction(include_str!("../../testdata/transaction_swap_panora.json"));

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

        let metadata = mapped.swap_metadata().unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::Aptos, APTOS_USDT_TOKEN_ID));
        assert_eq!(metadata.from_value, BigUint::from(2346314u64));
        assert_eq!(metadata.to_asset, Chain::Aptos.as_asset_id());
        assert_eq!(metadata.to_value, BigUint::from(120590251u64));
        assert_eq!(metadata.provider.unwrap(), "panora");
        assert_eq!(metadata.referral_fee, None);
    }

    #[test]
    fn test_map_transaction_swap_from_type_arguments() {
        let usdc = "0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::USDC";
        let mut transaction: Transaction = serde_json::from_str(include_str!("../../testdata/transaction_swap_panora.json")).unwrap();
        transaction.events.as_mut().unwrap().retain(|event| !event.event_type.contains(PANORA_SWAP_SUMMARY_EVENT));
        transaction.payload.as_mut().unwrap().type_arguments = vec![APTOS_NATIVE_COIN.to_string(), usdc.to_string()];

        let [mapped] = map_transaction(transaction).try_into().unwrap();
        let metadata = mapped.swap_metadata().unwrap();

        assert_eq!(metadata.from_asset, Chain::Aptos.as_asset_id());
        assert_eq!(metadata.from_value, BigUint::from(2346314u64));
        assert_eq!(metadata.to_asset, AssetId::from_token(Chain::Aptos, usdc));
        assert_eq!(metadata.to_value, BigUint::from(83u64));
        assert_eq!(metadata.provider.unwrap(), "panora");
    }

    #[test]
    fn test_map_transaction_swap_panora_referral_fee() {
        let mapped = transaction(include_str!("../../testdata/transaction_swap_panora_referral_fee.json"));

        let metadata = mapped.swap_metadata().unwrap();

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
    fn test_map_transaction_swap_hyperion_referral_fee() {
        let mapped = transaction(include_str!("../../testdata/transaction_swap_hyperion_referral_fee.json"));

        let metadata = mapped.swap_metadata().unwrap();

        assert_eq!(mapped.transaction_type, TransactionType::Swap);
        assert_eq!(mapped.contract.as_deref(), Some(HYPERION_CONTRACT));
        assert_eq!(mapped.asset_id, Chain::Aptos.as_asset_id());
        assert_eq!(mapped.value, BigUint::from(100_000_000u64));
        assert_eq!(
            metadata,
            primitives::TransactionSwapMetadata {
                from_asset: Chain::Aptos.as_asset_id(),
                from_value: BigUint::from(100_000_000u64),
                to_asset: AssetId::from_token(Chain::Aptos, APTOS_USDT_TOKEN_ID),
                to_value: BigUint::from(835_326u64),
                provider: Some(SwapProvider::Hyperion.id().to_string()),
                referral_fee: Some(TransactionSwapReferralFee {
                    asset_id: Chain::Aptos.as_asset_id(),
                    value: BigUint::from(500_000u64),
                }),
            }
        );
    }

    #[test]
    fn test_map_transaction_stake_delegate() {
        let mapped = transaction(include_str!("../../testdata/transaction_stake_delegate.json"));

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
        let mapped = transaction(include_str!("../../testdata/transaction_stake_undelegate.json"));

        assert_eq!(mapped.id.to_string(), "aptos_0xef6430bef0e8de7090b2c4bce210adb75d648be4614dcc37232b0d67f819b137");
        assert_eq!(mapped.from, "0x6467997d9c3a5bc9f714e17a168984595ce9bec7350645713a1fe7983a7f5fcc");
        assert_eq!(mapped.to, "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e");
        assert_eq!(mapped.value, BigUint::from(1109984251u64));
        assert_eq!(mapped.state, TransactionState::Confirmed);
        assert_eq!(mapped.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(mapped.fee, BigUint::from(88400u64));
    }

    #[test]
    fn test_map_transaction_stake_withdraw() {
        assert_eq!(
            transaction(include_str!("../../testdata/transaction_stake_withdraw.json")),
            PrimitivesTransaction::new(
                "0x4603c296ee77e4972351d5c9262fe884f011767a6046562c94e18802704e43da".to_string(),
                Chain::Aptos.as_asset_id(),
                "0x7393c1f923d4e8ce6af21dbc697f84e648d56b9cb8c69939040dc20888e3221d".to_string(),
                "0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e".to_string(),
                None,
                TransactionType::StakeWithdraw,
                TransactionState::Confirmed,
                BigUint::from(47_100u32),
                Chain::Aptos.as_asset_id(),
                BigUint::from(1_255_419_289u64),
                None,
                None,
                DateTime::from_timestamp_micros(1_795_246_058_086_814),
            )
        );
    }
}
