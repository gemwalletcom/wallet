use super::{
    constants::{
        DEEP_COIN_TYPE, DEEPBOOK_PACKAGE, DEEPBOOK_REGISTRY, DEEPBOOK_REGISTRY_INIT_VERSION, FUNCTION_GET_BASE_QUANTITY_OUT_INPUT_FEE, FUNCTION_GET_POOL_ID_BY_ASSET, FUNCTION_GET_QUOTE_QUANTITY_IN,
        FUNCTION_GET_QUOTE_QUANTITY_OUT_INPUT_FEE, FUNCTION_POOL_BOOK_PARAMS, FUNCTION_POOL_TRADE_PARAMS, FUNCTION_SWAP_EXACT_QUANTITY, MODULE_POOL,
    },
    model::{OrderSide, Pool, Route, route_legs},
};
use crate::{
    Quote, SwapperError,
    fees::{FeeSide, ReferralFee},
    sui_swap::{coin_type, fee_side, inspect_transaction_kind_bytes, package_address, pay_referral_fee, send_coin, send_output, shared_object_input},
};
use gem_sui::{
    models::{Coin, OwnedCoins, TxOutput},
    sui_clock_object_input,
    tx_builder::{TransactionBuilderInput, build_input_coin, destroy_zero_balance, finish_transaction, into_balance, move_call, zero_coin},
};
use sui_transaction_builder::TransactionBuilder;

pub(super) fn build_pool_id_inspect(base_coin_type: &str, quote_coin_type: &str) -> Result<Vec<u8>, SwapperError> {
    let mut txb = TransactionBuilder::new();
    let registry = txb.object(shared_object_input(DEEPBOOK_REGISTRY, DEEPBOOK_REGISTRY_INIT_VERSION, false)?);
    move_call(&mut txb, package_address(DEEPBOOK_PACKAGE)?, MODULE_POOL, FUNCTION_GET_POOL_ID_BY_ASSET, &[base_coin_type, quote_coin_type], vec![registry]).map_err(SwapperError::compute_quote_error)?;
    inspect_transaction_kind_bytes(txb)
}

pub(super) fn build_quote_inspect(legs: &[(&Pool, OrderSide)], amount_in: u64) -> Result<Vec<u8>, SwapperError> {
    let mut txb = TransactionBuilder::new();
    let package = package_address(DEEPBOOK_PACKAGE)?;
    let clock = txb.object(sui_clock_object_input());
    let mut amount = txb.pure(&amount_in);
    for (pool, side) in legs {
        let type_args = [pool.base_coin_type.as_str(), pool.quote_coin_type.as_str()];
        let pool = txb.object(shared_object_input(&pool.pool_id, pool.initial_shared_version, false)?);
        let (quote_function, minimum_function, amount_out) = match side {
            OrderSide::Ask => (FUNCTION_GET_QUOTE_QUANTITY_OUT_INPUT_FEE, FUNCTION_POOL_TRADE_PARAMS, 1),
            OrderSide::Bid => (FUNCTION_GET_BASE_QUANTITY_OUT_INPUT_FEE, FUNCTION_GET_QUOTE_QUANTITY_IN, 0),
        };
        let quantities = move_call(&mut txb, package, MODULE_POOL, quote_function, &type_args, vec![pool, amount, clock])
            .map_err(SwapperError::compute_quote_error)?
            .to_nested(3);
        let book_params = move_call(&mut txb, package, MODULE_POOL, FUNCTION_POOL_BOOK_PARAMS, &type_args, vec![pool])
            .map_err(SwapperError::compute_quote_error)?
            .to_nested(3);
        let minimum_arguments = match side {
            OrderSide::Ask => vec![pool],
            OrderSide::Bid => vec![pool, book_params[2], txb.pure(&false), clock],
        };
        move_call(&mut txb, package, MODULE_POOL, minimum_function, &type_args, minimum_arguments).map_err(SwapperError::compute_quote_error)?;
        amount = quantities[amount_out];
    }
    inspect_transaction_kind_bytes(txb)
}

pub(super) fn build_swap_transaction(quote: &Quote, route: &Route, referral_fee: &ReferralFee, transaction: &TransactionBuilderInput, from_coins: &OwnedCoins<Coin>) -> Result<TxOutput, SwapperError> {
    let input_coin_type = coin_type(&quote.request.from_asset.asset_id());
    let output_coin_type = coin_type(&quote.request.to_asset.asset_id());
    let legs = route_legs(&route.pools, &input_coin_type, &output_coin_type).ok_or(SwapperError::InvalidRoute)?;
    let amount = quote.from_value_u64()?;
    let fee = referral_fee.transfer(fee_side(&input_coin_type, &output_coin_type), amount, route.amount_out);
    let pool_min_out = fee.minimum_output(route.amount_out, quote.request.options.slippage.bps);
    let min_out = fee.received_value(pool_min_out);

    let mut txb = TransactionBuilder::new();
    let package = package_address(DEEPBOOK_PACKAGE)?;
    let input_coin = build_input_coin(&mut txb, &input_coin_type, amount, from_coins).map_err(SwapperError::transaction_error)?;
    let mut coin = match fee.side {
        FeeSide::Input => pay_referral_fee(&mut txb, input_coin, fee.value, referral_fee)?,
        FeeSide::Output => input_coin,
    };
    let clock = txb.object(sui_clock_object_input());
    for (index, (pool, side)) in legs.iter().enumerate() {
        let (leg_input_type, leg_output_type) = pool.coin_types(*side);
        let pool_arg = txb.object(shared_object_input(&pool.pool_id, pool.initial_shared_version, true)?);
        let zero = zero_coin(&mut txb, leg_output_type).map_err(SwapperError::transaction_error)?;
        let (base_in, quote_in) = match side {
            OrderSide::Ask => (coin, zero),
            OrderSide::Bid => (zero, coin),
        };
        let deep_in = zero_coin(&mut txb, DEEP_COIN_TYPE).map_err(SwapperError::transaction_error)?;
        let leg_min_out = txb.pure(&if index + 1 == legs.len() { pool_min_out } else { 0 });
        let outputs = move_call(
            &mut txb,
            package,
            MODULE_POOL,
            FUNCTION_SWAP_EXACT_QUANTITY,
            &[&pool.base_coin_type, &pool.quote_coin_type],
            vec![pool_arg, base_in, quote_in, deep_in, leg_min_out, clock],
        )
        .map_err(SwapperError::transaction_error)?
        .to_nested(3);
        let [base_out, quote_out, deep_out] = outputs[..] else {
            return Err(SwapperError::TransactionError("DeepBook swap did not return expected outputs".into()));
        };
        let (output_coin, input_left) = match side {
            OrderSide::Ask => (quote_out, base_out),
            OrderSide::Bid => (base_out, quote_out),
        };
        let deep_balance = into_balance(&mut txb, DEEP_COIN_TYPE, deep_out).map_err(SwapperError::transaction_error)?;
        destroy_zero_balance(&mut txb, DEEP_COIN_TYPE, deep_balance).map_err(SwapperError::transaction_error)?;
        send_coin(&mut txb, quote, leg_input_type, input_left, &quote.request.wallet_address)?;
        coin = output_coin;
    }
    send_output(&mut txb, quote, &output_coin_type, coin, &fee, min_out, referral_fee)?;

    finish_transaction(txb, transaction.clone()).map_err(SwapperError::transaction_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        QuoteRequest,
        testkit::{TEST_SUI_WALLET, mock_quote},
    };
    use gem_sui::{decode_transaction, sui_framework_package_address};
    use num_bigint::BigUint;
    use primitives::{
        AssetId, Chain,
        asset_constants::{SUI_USDC_TOKEN_ID, SUI_WAL_TOKEN_ID},
        swap::SUI_REFERRAL_ADDRESS,
    };
    use sui_transaction_builder::ObjectInput;
    use sui_types::{Address, Argument, Command, Digest, Input, Transaction, TransactionKind};

    fn describe(transaction: &TxOutput) -> Vec<String> {
        let transaction: Transaction = decode_transaction(&transaction.base64_encoded()).unwrap();
        let ptb = match transaction.kind {
            TransactionKind::ProgrammableTransaction(ptb) => ptb,
            _ => return Vec::new(),
        };
        let pure = |argument: &Argument| match argument {
            Argument::Input(index) => match &ptb.inputs[*index as usize] {
                Input::Pure(bytes) if bytes.len() == 8 => u64::from_le_bytes(bytes.as_slice().try_into().unwrap()).to_string(),
                Input::Pure(bytes) => format!("0x{}", hex::encode(bytes)),
                Input::Shared(shared) => shared.object_id().to_string(),
                Input::FundsWithdrawal(_) => "withdrawal".to_string(),
                input => format!("{input:?}"),
            },
            Argument::Gas => "gas".to_string(),
            argument => format!("{argument:?}"),
        };
        let short = |value: String| value.rsplit("::").next().unwrap_or_default().to_string();
        let list = |values: Vec<String>| values.join(" ");
        ptb.commands
            .iter()
            .map(|command| match command {
                Command::MoveCall(call) => {
                    let package = if call.package == sui_framework_package_address() { "0x2".to_string() } else { call.package.to_string() };
                    let type_arguments = call.type_arguments.iter().map(|type_argument| short(type_argument.to_string())).collect::<Vec<_>>().join(", ");
                    format!("{package}::{}::{}<{type_arguments}> {}", call.module, call.function, list(call.arguments.iter().map(pure).collect()))
                }
                Command::SplitCoins(split) => format!("split {} {}", pure(&split.coin), list(split.amounts.iter().map(pure).collect())),
                Command::MergeCoins(merge) => format!("merge into {}", pure(&merge.coin)),
                Command::TransferObjects(transfer) => format!("transfer to {}", pure(&transfer.address)),
                command => format!("{command:?}"),
            })
            .collect()
    }

    #[test]
    fn test_build_swap_transaction() {
        let sui = AssetId::from_chain(Chain::Sui);
        let usdc = AssetId::from_token(Chain::Sui, SUI_USDC_TOKEN_ID);
        let transaction = TransactionBuilderInput::new(TEST_SUI_WALLET, 500, 50_000_000, vec![ObjectInput::owned(Address::ZERO, 1, Digest::ZERO)]);
        let pool = Pool::mock().pool_id;

        let sell_sui = Quote::mock_with_request(&QuoteRequest {
            wallet_address: TEST_SUI_WALLET.into(),
            destination_address: TEST_SUI_WALLET.into(),
            value: BigUint::from(1_071_500_000_000u64),
            ..mock_quote(sui.clone().into(), usdc.clone().into())
        });
        let route = Route {
            pools: vec![Pool::mock()],
            amount_out: 994_220_872,
        };
        let output = build_swap_transaction(&sell_sui, &route, &sell_sui.request.on_chain_referral_fee(), &transaction, &OwnedCoins::default()).unwrap();
        assert_eq!(
            describe(&output),
            vec![
                "split gas 1071500000000".to_string(),
                "split NestedResult(0, 0) 5357500000".to_string(),
                format!("transfer to {SUI_REFERRAL_ADDRESS}"),
                "0x2::coin::zero<USDC> ".to_string(),
                "0x2::coin::zero<DEEP> ".to_string(),
                format!("{DEEPBOOK_PACKAGE}::pool::swap_exact_quantity<SUI, USDC> {pool} NestedResult(0, 0) Result(3) Result(4) 989249767 0x0000000000000000000000000000000000000000000000000000000000000006"),
                "0x2::coin::into_balance<DEEP> NestedResult(5, 2)".to_string(),
                "0x2::balance::destroy_zero<DEEP> Result(6)".to_string(),
                "merge into gas".to_string(),
                "split NestedResult(5, 1) 989249767".to_string(),
                "merge into NestedResult(5, 1)".to_string(),
                format!("transfer to {TEST_SUI_WALLET}"),
            ]
        );

        let buy_sui = Quote::mock_with_request(&QuoteRequest {
            wallet_address: TEST_SUI_WALLET.into(),
            destination_address: TEST_SUI_WALLET.into(),
            value: BigUint::from(200_000_000u64),
            ..mock_quote(usdc.clone().into(), sui.clone().into())
        });
        let usdc_coins = OwnedCoins::new(SUI_USDC_TOKEN_ID.to_string(), vec![], 500_000_000);
        let route = Route {
            pools: vec![Pool::mock()],
            amount_out: 214_200_000_000,
        };
        let output = build_swap_transaction(&buy_sui, &route, &buy_sui.request.on_chain_referral_fee(), &transaction, &usdc_coins).unwrap();
        assert_eq!(
            describe(&output),
            vec![
                "0x2::coin::redeem_funds<USDC> withdrawal".to_string(),
                "0x2::coin::zero<SUI> ".to_string(),
                "0x2::coin::zero<DEEP> ".to_string(),
                format!("{DEEPBOOK_PACKAGE}::pool::swap_exact_quantity<SUI, USDC> {pool} Result(1) Result(0) Result(2) 213134355000 0x0000000000000000000000000000000000000000000000000000000000000006"),
                "0x2::coin::into_balance<DEEP> NestedResult(3, 2)".to_string(),
                "0x2::balance::destroy_zero<DEEP> Result(4)".to_string(),
                format!("transfer to {TEST_SUI_WALLET}"),
                "split NestedResult(3, 0) 1071000000".to_string(),
                format!("transfer to {SUI_REFERRAL_ADDRESS}"),
                "split NestedResult(3, 0) 212063355000".to_string(),
                "merge into NestedResult(3, 0)".to_string(),
                "merge into gas".to_string(),
            ]
        );

        let deep_to_wal = Quote::mock_with_request(&QuoteRequest {
            wallet_address: TEST_SUI_WALLET.into(),
            destination_address: TEST_SUI_WALLET.into(),
            value: BigUint::from(1_000_000_000u64),
            ..mock_quote(AssetId::from_token(Chain::Sui, DEEP_COIN_TYPE).into(), AssetId::from_token(Chain::Sui, SUI_WAL_TOKEN_ID).into())
        });
        let deep_usdc = Pool {
            pool_id: "0xf948981b806057580f91622417534f491da5f61aeaf33d0ed8e69fd5691c95ce".into(),
            base_coin_type: DEEP_COIN_TYPE.into(),
            ..Pool::mock()
        };
        let wal_usdc = Pool {
            pool_id: "0x56a1c985c1f1123181d6b881714793689321ba24301b3585eec427436eb1c76d".into(),
            base_coin_type: SUI_WAL_TOKEN_ID.into(),
            ..Pool::mock()
        };
        let deep_coins = OwnedCoins::new(DEEP_COIN_TYPE.to_string(), vec![], 2_000_000_000);
        let route = Route {
            pools: vec![deep_usdc.clone(), wal_usdc.clone()],
            amount_out: 470_000_000_000,
        };
        let output = build_swap_transaction(&deep_to_wal, &route, &deep_to_wal.request.on_chain_referral_fee(), &transaction, &deep_coins).unwrap();
        assert_eq!(
            describe(&output),
            vec![
                "0x2::coin::redeem_funds<DEEP> withdrawal".to_string(),
                "0x2::coin::zero<USDC> ".to_string(),
                "0x2::coin::zero<DEEP> ".to_string(),
                format!(
                    "{DEEPBOOK_PACKAGE}::pool::swap_exact_quantity<DEEP, USDC> {} Result(0) Result(1) Result(2) 0 0x0000000000000000000000000000000000000000000000000000000000000006",
                    deep_usdc.pool_id
                ),
                "0x2::coin::into_balance<DEEP> NestedResult(3, 2)".to_string(),
                "0x2::balance::destroy_zero<DEEP> Result(4)".to_string(),
                format!("transfer to {TEST_SUI_WALLET}"),
                "0x2::coin::zero<WAL> ".to_string(),
                "0x2::coin::zero<DEEP> ".to_string(),
                format!(
                    "{DEEPBOOK_PACKAGE}::pool::swap_exact_quantity<WAL, USDC> {} Result(7) NestedResult(3, 1) Result(8) 467661750000 0x0000000000000000000000000000000000000000000000000000000000000006",
                    wal_usdc.pool_id
                ),
                "0x2::coin::into_balance<DEEP> NestedResult(9, 2)".to_string(),
                "0x2::balance::destroy_zero<DEEP> Result(10)".to_string(),
                format!("transfer to {TEST_SUI_WALLET}"),
                "split NestedResult(9, 0) 2350000000".to_string(),
                format!("transfer to {SUI_REFERRAL_ADDRESS}"),
                "split NestedResult(9, 0) 465311750000".to_string(),
                "merge into NestedResult(9, 0)".to_string(),
                format!("transfer to {TEST_SUI_WALLET}"),
            ]
        );

        let wrong_pair = Quote::mock_with_request(&QuoteRequest {
            wallet_address: TEST_SUI_WALLET.into(),
            destination_address: TEST_SUI_WALLET.into(),
            ..mock_quote(sui.into(), AssetId::from_token(Chain::Sui, DEEP_COIN_TYPE).into())
        });
        assert_eq!(
            build_swap_transaction(
                &wrong_pair,
                &Route { pools: vec![Pool::mock()], amount_out: 1 },
                &wrong_pair.request.on_chain_referral_fee(),
                &transaction,
                &OwnedCoins::default()
            )
            .err(),
            Some(SwapperError::InvalidRoute)
        );
    }
}
