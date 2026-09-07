use crate::address::{Address, hex_to_base64_address};
use crate::constants::{FAILED_OPERATION_OPCODES, STONFI_PTON_ADDRESSES};
use crate::models::{
    BroadcastTransaction, JettonSwapDetails, JettonTransferDetails, NftTransferDetails, OutMessage, TRACE_ACTION_JETTON_SWAP, TRACE_ACTION_JETTON_TRANSFER,
    TRACE_ACTION_NFT_TRANSFER, Trace, TraceAction, TransactionMessage,
};
use chrono::DateTime;
use gem_encoding::decode_base64;
use num_bigint::BigUint;
use primitives::{AssetId, NFTAssetId, Transaction, TransactionNFTTransferMetadata, TransactionState, TransactionSwapMetadata, TransactionType, chain::Chain};
use std::error::Error;

pub fn map_transaction_broadcast(broadcast_result: BroadcastTransaction) -> Result<String, Box<dyn Error + Sync + Send>> {
    let hash_bytes = decode_base64(&broadcast_result.hash)?;
    Ok(hex::encode(hash_bytes))
}

pub(crate) fn map_transaction_state(transaction: &TransactionMessage) -> TransactionState {
    if let Some(description) = &transaction.description {
        if description.aborted {
            return TransactionState::Failed;
        }
        if let Some(compute_phase) = &description.compute_ph {
            if !compute_phase.success.unwrap_or(false) {
                return TransactionState::Failed;
            }
            if let Some(exit_code) = compute_phase.exit_code
                && exit_code != 0
                && exit_code != 1
            {
                return TransactionState::Failed;
            }
        }
        if let Some(action) = &description.action
            && !action.success.unwrap_or(false)
        {
            return TransactionState::Failed;
        }
    }

    if transaction.out_msgs.is_empty() {
        return TransactionState::Failed;
    }

    if let Some(in_msg) = &transaction.in_msg
        && let Some(opcode) = &in_msg.opcode
        && FAILED_OPERATION_OPCODES.contains(&opcode.as_str())
    {
        return TransactionState::Reverted;
    }

    TransactionState::Confirmed
}

pub(crate) fn base64_hash_to_hex(base64_hash: &str) -> Option<String> {
    decode_base64(base64_hash).ok().map(hex::encode)
}

pub fn map_trace_transactions(traces: Vec<Trace>) -> Vec<Transaction> {
    traces.into_iter().filter_map(map_root_trace_transaction).collect()
}

struct TransferDetails {
    asset_id: AssetId,
    from: String,
    to: String,
    value: BigUint,
    transaction_type: TransactionType,
    memo: Option<String>,
    metadata: Option<serde_json::Value>,
}

fn map_root_trace_transaction(trace: Trace) -> Option<Transaction> {
    let state = if trace.is_incomplete || trace.has_actions() { Some(trace.action_state()) } else { None };
    let root = trace.root_transaction()?;

    let details = jetton_swap_details(&trace.actions)
        .or_else(|| nft_transfer_details(&trace.actions))
        .or_else(|| jetton_transfer_details(&trace.actions))
        .or_else(|| simple_transfer_details(root))?;

    build_transaction(root, state, details)
}

fn build_transaction(message: &TransactionMessage, state: Option<TransactionState>, details: TransferDetails) -> Option<Transaction> {
    let fee_asset_id = Chain::Ton.as_asset_id();
    let state = state.unwrap_or_else(|| map_transaction_state(message));
    let created_at = DateTime::from_timestamp(message.now, 0)?;
    let hash = base64_hash_to_hex(&message.hash)?;

    Some(Transaction::new(
        hash,
        details.asset_id,
        details.from,
        details.to,
        None,
        details.transaction_type,
        state,
        message.total_fees.clone(),
        fee_asset_id,
        details.value,
        details.memo,
        details.metadata,
        created_at,
    ))
}

fn find_action<'a>(actions: &'a [TraceAction], action_type: &str) -> Option<&'a TraceAction> {
    actions
        .iter()
        .find(|action| action.action_type.as_deref() == Some(action_type) && action.success == Some(true))
}

fn jetton_transfer_details(actions: &[TraceAction]) -> Option<TransferDetails> {
    let details: JettonTransferDetails = serde_json::from_value(find_action(actions, TRACE_ACTION_JETTON_TRANSFER)?.details.clone()?).ok()?;
    let token_id = hex_to_base64_address(&details.asset)?;
    Some(TransferDetails {
        asset_id: AssetId::from_token(Chain::Ton, &token_id),
        from: parse_address(&details.sender)?,
        to: parse_address(&details.receiver)?,
        value: details.amount.clone(),
        transaction_type: TransactionType::Transfer,
        memo: details.comment.filter(|comment| !comment.is_empty()),
        metadata: None,
    })
}

fn nft_transfer_details(actions: &[TraceAction]) -> Option<TransferDetails> {
    let details: NftTransferDetails = serde_json::from_value(find_action(actions, TRACE_ACTION_NFT_TRANSFER)?.details.clone()?).ok()?;
    let collection = details.nft_collection.encode_bounceable();
    let item = details.nft_item.encode_bounceable();
    let metadata = TransactionNFTTransferMetadata::from_asset_id(NFTAssetId::new(Chain::Ton, &collection, &item));
    let metadata_value = serde_json::to_value(metadata).ok()?;

    Some(TransferDetails {
        asset_id: AssetId::from_chain(Chain::Ton),
        from: details.old_owner.encode_non_bounceable(),
        to: details.new_owner.encode_non_bounceable(),
        value: BigUint::from(0u32),
        transaction_type: TransactionType::TransferNFT,
        memo: details.comment.filter(|comment| !comment.is_empty()),
        metadata: Some(metadata_value),
    })
}

fn jetton_swap_details(actions: &[TraceAction]) -> Option<TransferDetails> {
    let (sender, metadata) = jetton_swap_metadata(actions)?;
    let asset_id = metadata.from_asset.clone();
    let value = metadata.from_value.clone();
    let metadata_value = serde_json::to_value(metadata).ok()?;

    Some(TransferDetails {
        asset_id,
        from: sender.clone(),
        to: sender,
        value,
        transaction_type: TransactionType::Swap,
        memo: None,
        metadata: Some(metadata_value),
    })
}

fn jetton_swap_metadata(actions: &[TraceAction]) -> Option<(String, TransactionSwapMetadata)> {
    let action = find_action(actions, TRACE_ACTION_JETTON_SWAP)?;
    let swap: JettonSwapDetails = action.details.clone().and_then(|value| serde_json::from_value(value).ok())?;
    let sender = parse_address(&swap.sender)?;
    let (Some(from_asset), Some(to_asset)) = (ton_asset_id(swap.asset_in.as_deref()), ton_asset_id(swap.asset_out.as_deref())) else {
        return None;
    };
    let metadata = TransactionSwapMetadata {
        from_asset,
        from_value: swap.dex_incoming_transfer.amount.clone(),
        to_asset,
        to_value: swap.dex_outgoing_transfer.amount.clone(),
        provider: swap.dex,
    };
    Some((sender, metadata))
}

fn ton_asset_id(raw_address: Option<&str>) -> Option<AssetId> {
    match raw_address {
        None => Some(AssetId::from_chain(Chain::Ton)),
        Some(hex_address) => {
            let token_id = hex_to_base64_address(hex_address)?;
            Some(if STONFI_PTON_ADDRESSES.contains(&token_id.as_str()) {
                AssetId::from_chain(Chain::Ton)
            } else {
                AssetId::from_token(Chain::Ton, &token_id)
            })
        }
    }
}

fn simple_transfer_details(message: &TransactionMessage) -> Option<TransferDetails> {
    let asset_id = Chain::Ton.as_asset_id();

    if message.out_msgs.len() == 1 && is_simple_transfer(message.out_msgs.first()?) {
        let out_message = message.out_msgs.first()?;
        let to = parse_address(out_message.destination.as_deref()?)?;
        return Some(TransferDetails {
            asset_id,
            from: parse_address(&out_message.source)?,
            to,
            value: out_message.value.clone().unwrap_or_else(|| BigUint::from(0u32)),
            transaction_type: TransactionType::Transfer,
            memo: extract_memo(out_message),
            metadata: None,
        });
    }

    if message.out_msgs.is_empty()
        && let Some(in_msg) = &message.in_msg
        && let (Some(value), Some(source)) = (&in_msg.value, &in_msg.source)
        && value > &BigUint::ZERO
    {
        return Some(TransferDetails {
            asset_id,
            from: parse_address(source)?,
            to: parse_address(&in_msg.destination)?,
            value: value.clone(),
            transaction_type: TransactionType::Transfer,
            memo: None,
            metadata: None,
        });
    }

    None
}

fn parse_address(address: &str) -> Option<String> {
    Address::try_parse_hex(address).map(|a| a.encode_non_bounceable())
}

fn is_simple_transfer(out_message: &crate::models::OutMessage) -> bool {
    match &out_message.op_code {
        None => true,
        Some(op_code) => op_code == "0x00000000" || op_code == "0x0",
    }
}

fn extract_memo(message: &OutMessage) -> Option<String> {
    if let Some(comment) = &message.comment
        && !comment.is_empty()
    {
        return Some(comment.clone());
    }

    if let Some(decoded_body) = &message.decoded_body {
        if let Some(text) = &decoded_body.text
            && !text.is_empty()
        {
            return Some(text.clone());
        }
        if let Some(comment) = &decoded_body.comment
            && !comment.is_empty()
        {
            return Some(comment.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::base64_to_hex_address;
    use crate::models::{MessageTransactions, TraceResponse};
    use crate::provider::testkit::{FAILED_SWAP_ROOT_TRANSACTION_HEX_HASH, SUCCESS_SWAP_ROOT_TRANSACTION_HEX_HASH, TEST_TRANSACTION_ID};
    use primitives::testkit::signer_mock::TEST_TON_SENDER;
    use serde_json::{Map, Value, json};

    const NFT_NEW_OWNER: &str = "UQDSkZZueXRl0lUk4hagLa8KrJzZbmtTE_RPZwTDSIw32WNH";

    #[test]
    fn test_map_trace_transactions_dust_to_native_ton() {
        let traces: TraceResponse = serde_json::from_str(include_str!("../../testdata/jetton_swap_dust_ton_trace.json")).unwrap();
        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        let dust = AssetId::from_token(Chain::Ton, "EQBlqsm144Dq6SjbPI4jjZvA1hqTIP3CvHovbIfW_t-SCALE");
        assert_eq!(transaction.hash(), "ff12c5cab57bb02a23092d2ebb6b7319cd73f2cbc7329fff5cbc4810894159ef");
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.asset_id, dust);
        assert_eq!(transaction.value, BigUint::from(2263786603u64));
        assert_eq!(
            serde_json::from_value::<TransactionSwapMetadata>(transaction.metadata.clone().unwrap()).unwrap(),
            TransactionSwapMetadata {
                from_asset: dust,
                from_value: BigUint::from(2263786603u64),
                to_asset: Chain::Ton.as_asset_id(),
                to_value: BigUint::from(726191509u64),
                provider: Some("stonfi".to_string()),
            }
        );
    }

    #[test]
    fn test_map_trace_transactions_proxy_ton_input() {
        let mut traces = TraceResponse::mock_jetton_swap();
        let details = traces.traces[0].actions[0].details.as_mut().unwrap();
        details["dex"] = json!("stonfi");
        details["asset_in"] = json!("0:8cdc1d7640ad5ee326527fc1ad0514f468b30dc84b0173f0e155f451b4e11f7c");

        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.asset_id, Chain::Ton.as_asset_id());
        assert_eq!(transaction.value, BigUint::from(1000000000u64));
        assert_eq!(
            serde_json::from_value::<TransactionSwapMetadata>(transaction.metadata.clone().unwrap()).unwrap(),
            TransactionSwapMetadata {
                from_asset: Chain::Ton.as_asset_id(),
                from_value: BigUint::from(1000000000u64),
                to_asset: AssetId::from_token(Chain::Ton, "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs"),
                to_value: BigUint::from(2436222u64),
                provider: Some("stonfi".to_string()),
            }
        );
    }

    #[test]
    fn test_map_trace_transactions_proxy_ton_v2() {
        for (mut traces, field) in [
            (TraceResponse::mock_jetton_swap(), "asset_in"),
            (TraceResponse::mock_jetton_swap_from_jetton_transfer(), "asset_out"),
        ] {
            let expected = map_trace_transactions(traces.traces.clone());
            traces.traces[0].actions[0].details.as_mut().unwrap()[field] = json!("0:671963027F7F85659AB55B821671688601CDCF1EE674FC7FBBB1A776A18D34A3");

            let transactions = map_trace_transactions(traces.traces);

            assert_eq!(transactions.len(), 1);
            assert_eq!(transactions[0].asset_id, expected[0].asset_id);
            assert_eq!(transactions[0].value, expected[0].value);
            assert_eq!(transactions[0].metadata, expected[0].metadata);
        }
    }

    #[test]
    fn test_transaction_transfer_state_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_transaction_status_response_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_status_response.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_jetton_transfer_failed() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_error.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "ZEC9rE/pUvEHGAJVzDn/6QdWevOOR4sA2dN4BaTA8hQ=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Reverted);
    }

    #[test]
    fn test_jetton_transfer_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "X2rQTJQF38kXLWdQL42pP8NKrd2X1YDyp5h7Erq7sBA=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_jetton_transfer_success_2() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_jetton_success_2.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "pI2WtPJ6516pwuNti1h+Hetg0NZ8C/kBOboRkayUKL8=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_swap_ton_jetton_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_swap_ton_jetton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "wsQ2mvEWkMbw3QnyeBl85O+uuUsDNfuWJnc2mBh8lPg=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_swap_jetton_ton_success() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_swap_jetton_ton_success.json")).unwrap();

        assert_eq!(transactions.transactions.len(), 1);
        let transaction = &transactions.transactions[0];
        assert_eq!(transaction.hash, "psAXHb1HyvR53f9LHmOzQWohJu3tDRWbxvZbHB1B+iY=");

        let state = map_transaction_state(transaction);
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_transaction_with_null_values() {
        let transaction: TransactionMessage = serde_json::from_str(include_str!("../../testdata/transaction_null_values.json")).unwrap();

        assert_eq!(transaction.hash, "MhO9bk6+qCMfveyGBQYvoklath4SA7F/LegdwACJAvg=");
        assert_eq!(transaction.out_msgs.len(), 2);

        assert_eq!(transaction.out_msgs[0].value, None);
        assert_eq!(transaction.out_msgs[0].destination, None);

        assert_eq!(transaction.out_msgs[1].value, Some(BigUint::from(137245095u64)));
    }

    #[test]
    fn test_map_trace_transactions_jetton_swap() {
        let traces = TraceResponse::mock_jetton_swap();
        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.from, "UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV");
        assert_eq!(transaction.from, transaction.to);

        let metadata = transaction.metadata.as_ref().expect("swap metadata");
        let swap: TransactionSwapMetadata = serde_json::from_value(metadata.clone()).unwrap();
        assert_eq!(swap.from_asset, AssetId::from_chain(Chain::Ton));
        assert_eq!(swap.from_value, BigUint::from(1000000000u64));
        assert_eq!(swap.to_asset.chain, Chain::Ton);
        assert!(swap.to_asset.token_id.is_some());
        assert_eq!(swap.to_value, BigUint::from(2436222u64));
        assert_eq!(swap.provider.as_deref(), Some("stonfi_v2"));
    }

    #[test]
    fn test_map_trace_transactions_jetton_swap_from_jetton_transfer() {
        let traces = TraceResponse::mock_jetton_swap_from_jetton_transfer();
        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.from, "UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV");
        assert_eq!(transaction.from, transaction.to);
        assert_eq!(transaction.asset_id.chain, Chain::Ton);
        assert_eq!(transaction.asset_id.token_id.as_deref(), Some("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs"));
        assert_eq!(transaction.value, BigUint::from(1000000u64));

        let metadata = transaction.metadata.as_ref().unwrap();
        let swap: TransactionSwapMetadata = serde_json::from_value(metadata.clone()).unwrap();
        assert_eq!(swap.from_asset, AssetId::from_token(Chain::Ton, "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs"));
        assert_eq!(swap.from_value, BigUint::from(1000000u64));
        assert_eq!(swap.to_asset, AssetId::from_chain(Chain::Ton));
        assert_eq!(swap.to_value, BigUint::from(476299454u64));
        assert_eq!(swap.provider.as_deref(), Some("stonfi_v2"));
    }

    #[test]
    fn test_map_trace_transactions_jetton_transfer() {
        let traces = TraceResponse::mock_jetton_transfer();
        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];

        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.from, "UQAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz3VV");
        assert_eq!(transaction.to, "UQDSkZZueXRl0lUk4hagLa8KrJzZbmtTE_RPZwTDSIw32WNH");
        assert_eq!(transaction.value, BigUint::from(120000u64));
        assert_eq!(transaction.asset_id.chain, Chain::Ton);
        assert_eq!(transaction.asset_id.token_id.as_deref(), Some("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs"));
        assert_eq!(transaction.fee, BigUint::from(472458u64));
        assert_eq!(transaction.fee_asset_id, AssetId::from_chain(Chain::Ton));
        assert_eq!(transaction.memo, None);
    }

    #[test]
    fn test_map_trace_transactions_nft_transfer() {
        let transactions: MessageTransactions = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();
        let root = transactions.transactions.first().unwrap().clone();
        let nft_asset_id = NFTAssetId::mock_ton();
        let traces = TraceResponse::mock(
            root,
            false,
            vec![TraceAction {
                success: Some(true),
                action_type: Some(TRACE_ACTION_NFT_TRANSFER.to_string()),
                details: Some(json!({
                    "nft_collection": base64_to_hex_address(&nft_asset_id.contract_address).unwrap(),
                    "nft_item": base64_to_hex_address(&nft_asset_id.token_id).unwrap(),
                    "old_owner": base64_to_hex_address(TEST_TON_SENDER).unwrap(),
                    "new_owner": base64_to_hex_address(NFT_NEW_OWNER).unwrap(),
                    "comment": "gift",
                })),
            }],
        );

        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.asset_id, AssetId::from_chain(Chain::Ton));
        assert_eq!(transaction.from, TEST_TON_SENDER);
        assert_eq!(transaction.to, NFT_NEW_OWNER);
        assert_eq!(transaction.value, BigUint::from(0u64));
        assert_eq!(transaction.memo.as_deref(), Some("gift"));
        assert_eq!(transaction.nft_asset_id(), Some(nft_asset_id));
    }

    #[test]
    fn test_map_trace_transactions_nft_transfer_real() {
        let traces: TraceResponse = serde_json::from_str(include_str!("../../testdata/nft_transfer_trace.json")).unwrap();

        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.asset_id, AssetId::from_chain(Chain::Ton));
        assert_eq!(transaction.value, BigUint::from(0u64));

        let nft_asset_id = transaction.nft_asset_id().expect("nft asset id");
        assert_eq!(nft_asset_id.chain, Chain::Ton);
        assert!(!nft_asset_id.contract_address.is_empty());
        assert!(!nft_asset_id.token_id.is_empty());
    }

    #[test]
    fn test_map_trace_transactions_by_block() {
        let traces = TraceResponse::mock_block_traces();

        assert_eq!(traces.traces.len(), 2);

        let transactions = map_trace_transactions(traces.traces);
        let hashes = transactions.iter().map(|transaction| transaction.hash()).collect::<Vec<_>>();

        assert_eq!(hashes, vec![SUCCESS_SWAP_ROOT_TRANSACTION_HEX_HASH, FAILED_SWAP_ROOT_TRANSACTION_HEX_HASH]);
        assert_eq!(transactions[0].state, TransactionState::Confirmed);
        assert_eq!(transactions[1].state, TransactionState::Reverted);
    }

    #[test]
    fn test_map_trace_transactions_without_transactions_order() {
        let message_transactions: Value = serde_json::from_str(include_str!("../../testdata/transaction_transfer_state_success.json")).unwrap();
        let root = message_transactions["transactions"][0].clone();
        let root_hash = root["hash"].as_str().unwrap().to_string();

        let mut transactions = Map::new();
        transactions.insert(root_hash, root);

        let mut trace = Map::new();
        trace.insert("is_incomplete".to_string(), json!(false));
        trace.insert("actions".to_string(), json!([]));
        trace.insert("transactions".to_string(), Value::Object(transactions));

        let mut response = Map::new();
        response.insert("traces".to_string(), Value::Array(vec![Value::Object(trace)]));

        let traces: TraceResponse = serde_json::from_value(Value::Object(response)).unwrap();

        assert_eq!(traces.traces[0].transactions_order, Vec::<String>::new());

        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].hash(), "9ba7d317aaed24089e5cc3b0df4432294cb88ed485d202768cc2766432aec3e0");
        assert_eq!(transactions[0].state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_trace_transactions_skips_oversized_trace_without_transactions() {
        let traces: TraceResponse = serde_json::from_str(include_str!("../../testdata/oversized_trace_without_transactions.json")).unwrap();

        assert_eq!(traces.traces[0].transactions_order, Vec::<String>::new());
        assert_eq!(traces.traces[0].transactions.len(), 0);

        let transactions = map_trace_transactions(traces.traces);

        assert_eq!(transactions.len(), 0);
    }
}
