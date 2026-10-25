use crate::{
    Quote, SwapperError, SwapperQuoteData,
    fees::{FeeSide, ReferralFee, ReferralTransfer},
    route_path::RoutePath,
};
use futures::future::join_all;
use gem_sui::{
    EMPTY_ADDRESS, ESTIMATION_GAS_BUDGET, SUI_COIN_TYPE, SUI_COIN_TYPE_FULL, SuiClient, address::SuiAddress, coin_type_matches, full_coin_type, gas_budget::GAS_BUDGET_MULTIPLIER, is_sui_coin, models::TxOutput, tx_builder::ObjectResolver,
};
use num_bigint::BigUint;
use primitives::{Address as AddressTrait, AssetId, asset_constants::SUI_USDC_TOKEN_ID};
use std::collections::HashMap;
use sui_transaction_builder::{Argument, ObjectInput, TransactionBuilder};
use sui_types::{Address, Digest};

const PRIMARY_COIN_TYPES: &[&str] = &[SUI_COIN_TYPE_FULL, SUI_USDC_TOKEN_ID];

pub(crate) type MoveAbort<'a> = (&'a str, &'a str, u64);

pub(crate) fn coin_type(asset_id: &AssetId) -> String {
    full_coin_type(asset_id.token_id.as_deref().unwrap_or(SUI_COIN_TYPE))
}

pub(crate) fn fee_side(input_coin_type: &str, output_coin_type: &str) -> FeeSide {
    FeeSide::by_priority(PRIMARY_COIN_TYPES, |preferred| coin_type_matches(input_coin_type, preferred), |preferred| coin_type_matches(output_coin_type, preferred)).unwrap_or(FeeSide::Output)
}

pub(crate) fn route_paths(from: &str, to: &str) -> Vec<RoutePath<String>> {
    RoutePath::all(&from.to_string(), &to.to_string(), PRIMARY_COIN_TYPES.iter().map(|hub| full_coin_type(hub)), |left, right| coin_type_matches(left, right))
}

pub(crate) fn discovery_pairs(from: &str, to: &str) -> Vec<(String, String)> {
    RoutePath::all_pairs(&route_paths(from, to), &from.to_string(), &to.to_string())
}

pub(crate) fn package_address(package_id: &str) -> Result<Address, SwapperError> {
    SuiAddress::parse(package_id).map(Address::from).map_err(SwapperError::transaction_error)
}

pub(crate) async fn lookup_pools<Probe>(client: &SuiClient, lookups: Vec<(Probe, Vec<u8>)>, missing_pool: MoveAbort<'_>, pinned: &HashMap<String, u64>) -> Vec<(Probe, Option<(String, u64)>)> {
    let results = join_all(
        lookups
            .into_iter()
            .map(|(probe, transaction)| async move { lookup_pool_id(client, &transaction, missing_pool).await.map(|pool_id| (probe, pool_id)) }),
    )
    .await
    .into_iter()
    .filter_map(Result::ok)
    .collect::<Vec<_>>();
    let pool_ids = results.iter().filter_map(|(_, pool_id)| pool_id.clone()).collect();
    let Ok(resolver) = ObjectResolver::prefetch(client, pool_ids, pinned).await else {
        return results.into_iter().filter(|(_, pool_id)| pool_id.is_none()).map(|(probe, _)| (probe, None)).collect();
    };
    results
        .into_iter()
        .filter_map(|(probe, pool_id)| match pool_id {
            Some(pool_id) => resolver.initial_shared_version(&pool_id).map(|version| (probe, Some((pool_id, version)))),
            None => Some((probe, None)),
        })
        .collect()
}

async fn lookup_pool_id(client: &SuiClient, transaction: &[u8], (module, function, code): MoveAbort<'_>) -> Result<Option<String>, SwapperError> {
    let result = client.inspect_transaction_block(EMPTY_ADDRESS, transaction).await.map_err(SwapperError::compute_quote_error)?;
    if let Some(error) = result.error.as_deref() {
        return if is_move_abort(error, (module, function, code)) {
            Ok(None)
        } else {
            Err(SwapperError::ComputeQuoteError(format!("Sui pool lookup failed: {error}")))
        };
    }
    let pool_id = result
        .results
        .last()
        .and_then(|command| command.return_values.first())
        .map(|(bytes, _)| bytes)
        .filter(|bytes| bytes.len() == 32)
        .ok_or_else(|| SwapperError::ComputeQuoteError("Sui pool lookup returned no pool id".into()))?;
    Ok(Some(format!("0x{}", hex::encode(pool_id))))
}

pub(crate) fn is_move_abort(error: &str, (module, function, code): MoveAbort<'_>) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("moveabort") && error.contains(module) && error.contains(function) && [format!(", {code})"), format!("abort code {code}"), format!("abort_code: {code}")].iter().any(|abort| error.contains(abort))
}

pub(crate) fn upgraded_error(provider: &str) -> SwapperError {
    SwapperError::TransactionError(format!("{provider} was upgraded since this app was built; on-chain {provider} quotes are temporarily unavailable."))
}

pub(crate) async fn build_quote_data(client: &SuiClient, build_transaction: impl Fn(u64) -> Result<TxOutput, SwapperError>) -> Result<SwapperQuoteData, SwapperError> {
    let estimate = build_transaction(ESTIMATION_GAS_BUDGET)?;
    let dry_run = client.dry_run(estimate.base64_encoded()).await.map_err(SwapperError::transaction_error)?;
    if dry_run.effects.status.status != "success" {
        let detail = dry_run.effects.status.error.as_deref().unwrap_or("no details available");
        return Err(SwapperError::TransactionError(format!("Sui swap simulation failed: {detail}")));
    }

    let fee = dry_run.effects.gas_used.calculate_gas_budget().map_err(SwapperError::transaction_error)?;
    let gas_budget = fee * GAS_BUDGET_MULTIPLIER / 100;
    let output = build_transaction(gas_budget)?;

    Ok(SwapperQuoteData::new_contract(String::new(), BigUint::from(0u64), output.base64_encoded(), None, Some(gas_budget.to_string())))
}

pub(crate) fn pay_referral_fee(txb: &mut TransactionBuilder, input_coin: Argument, fee: u64, referral_fee: &ReferralFee) -> Result<Argument, SwapperError> {
    if fee == 0 {
        return Ok(input_coin);
    }
    let fee_amount = txb.pure(&fee);
    let fee_coin = txb.split_coins(input_coin, vec![fee_amount]).pop().ok_or_else(|| SwapperError::TransactionError("Sui referral fee split failed".into()))?;
    let recipient = SuiAddress::from_str(&referral_fee.address).map(Address::from)?;
    transfer_coin(txb, fee_coin, recipient);
    Ok(input_coin)
}

pub(crate) fn send_output(txb: &mut TransactionBuilder, quote: &Quote, output_coin_type: &str, output_coin: Argument, fee: &ReferralTransfer, min_out: u64, referral_fee: &ReferralFee) -> Result<(), SwapperError> {
    let pay_output_fee = match fee.side {
        FeeSide::Output => fee.value > 0,
        FeeSide::Input => false,
    };
    if pay_output_fee {
        let fee_amount_arg = txb.pure(&fee.value);
        let fee_coin = txb.split_coins(output_coin, vec![fee_amount_arg]).pop().ok_or_else(|| SwapperError::TransactionError("Sui output fee split failed".into()))?;
        let recipient = SuiAddress::from_str(&referral_fee.address).map(Address::from)?;
        transfer_coin(txb, fee_coin, recipient);
    }

    let min_out_arg = txb.pure(&min_out);
    let split_off = txb.split_coins(output_coin, vec![min_out_arg]).pop().ok_or_else(|| SwapperError::TransactionError("Sui min-out split failed".into()))?;
    txb.merge_coins(output_coin, vec![split_off]);

    let destination = if quote.request.destination_address.is_empty() {
        &quote.request.wallet_address
    } else {
        &quote.request.destination_address
    };
    send_coin(txb, quote, output_coin_type, output_coin, destination)
}

pub(crate) fn send_coin(txb: &mut TransactionBuilder, quote: &Quote, coin_type: &str, coin: Argument, recipient: &str) -> Result<(), SwapperError> {
    let recipient = SuiAddress::from_str(recipient).map(Address::from)?;
    let sender = SuiAddress::from_str(&quote.request.wallet_address).map(Address::from)?;
    if recipient == sender && is_sui_coin(coin_type) {
        let gas = txb.gas();
        txb.merge_coins(gas, vec![coin]);
    } else {
        transfer_coin(txb, coin, recipient);
    }
    Ok(())
}

fn transfer_coin(txb: &mut TransactionBuilder, coin: Argument, recipient: Address) {
    let recipient = txb.pure(&recipient);
    txb.transfer_objects(vec![coin], recipient);
}

pub(crate) fn shared_object_input(object_id: &str, initial_shared_version: u64, mutable: bool) -> Result<ObjectInput, SwapperError> {
    let address = SuiAddress::from_str(object_id).map(Address::from)?;
    Ok(ObjectInput::shared(address, initial_shared_version, mutable))
}

pub(crate) fn inspect_transaction_kind_bytes(mut txb: TransactionBuilder) -> Result<Vec<u8>, SwapperError> {
    txb.set_sender(SuiAddress::from_str(EMPTY_ADDRESS).map(Address::from)?);
    txb.set_gas_price(0);
    txb.set_gas_budget(0);
    txb.add_gas_objects(vec![ObjectInput::owned(Address::ZERO, 0, Digest::ZERO)]);
    let tx = txb.try_build().map_err(SwapperError::compute_quote_error)?;
    bcs::to_bytes(&tx.kind).map_err(SwapperError::compute_quote_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLUE: &str = "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE";

    #[test]
    fn test_is_move_abort() {
        let cetus = r#"MoveAbort(MoveLocation { module: ModuleId { name: Identifier("factory") }, function_name: Some("pool_simple_info") }, 10) in command 1"#;
        let deepbook = r#"MoveAbort(MoveLocation { module: ModuleId { address: 0e735f8c93a95722efd73521aca7a7652c0bb71ed1daf41b26dfd7d1ff71f748, name: Identifier("registry") }, function: 21, instruction: 16, function_name: Some("get_pool_id") }, 2) in command 0"#;

        assert!(is_move_abort(cetus, ("factory", "pool_simple_info", 10)));
        assert!(!is_move_abort(&cetus.replace(", 10)", ", 11)"), ("factory", "pool_simple_info", 10)));
        assert!(!is_move_abort(&cetus.replace("pool_simple_info", "pool_id"), ("factory", "pool_simple_info", 10)));
        assert!(is_move_abort(deepbook, ("registry", "get_pool_id", 2)));
        assert!(!is_move_abort(&deepbook.replace("}, 2)", "}, 3)"), ("registry", "get_pool_id", 2)));
        assert!(!is_move_abort(&deepbook.replace("registry", "pool"), ("registry", "get_pool_id", 2)));
        assert!(!is_move_abort("RPC timeout", ("registry", "get_pool_id", 2)));
        assert!(is_move_abort(
            &deepbook.replace("registry", "pool").replace("get_pool_id", "load_inner_mut").replace("}, 2)", "}, 10)"),
            ("pool", "load_inner", 10)
        ));
    }

    #[test]
    fn test_discovery_pairs() {
        assert_eq!(discovery_pairs(SUI_COIN_TYPE_FULL, SUI_USDC_TOKEN_ID), vec![(SUI_COIN_TYPE_FULL.to_string(), SUI_USDC_TOKEN_ID.to_string())]);
        assert_eq!(discovery_pairs(SUI_COIN_TYPE_FULL, BLUE).len(), 3);
        assert_eq!(discovery_pairs(BLUE, "0x123::coin::COIN").len(), 5);
    }

    #[test]
    fn test_fee_side() {
        assert_eq!(fee_side(SUI_COIN_TYPE_FULL, SUI_USDC_TOKEN_ID), FeeSide::Input);
        assert_eq!(fee_side(SUI_USDC_TOKEN_ID, SUI_COIN_TYPE_FULL), FeeSide::Output);
        assert_eq!(fee_side(SUI_COIN_TYPE_FULL, BLUE), FeeSide::Input);
        assert_eq!(fee_side(BLUE, SUI_COIN_TYPE_FULL), FeeSide::Output);
        assert_eq!(fee_side(SUI_USDC_TOKEN_ID, BLUE), FeeSide::Input);
        assert_eq!(fee_side(BLUE, SUI_USDC_TOKEN_ID), FeeSide::Output);
        assert_eq!(fee_side(BLUE, "0xfoo::bar::BAR"), FeeSide::Output);
    }
}
