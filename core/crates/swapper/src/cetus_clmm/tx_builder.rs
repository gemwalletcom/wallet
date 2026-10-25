use super::{
    constants::{
        CETUS_CLMM_PUBLISHED_AT, CETUS_GLOBAL_CONFIG, CETUS_PARTNER, CETUS_PARTNER_INIT_VERSION, CETUS_POOLS_REGISTRY, CETUS_SHARED_INIT_VERSION, FUNCTION_CALCULATE_SWAP_RESULT, FUNCTION_CALCULATED_SWAP_RESULT_AMOUNT_OUT,
        FUNCTION_FLASH_SWAP_WITH_PARTNER, FUNCTION_NEW_POOL_KEY, FUNCTION_POOL_ID, FUNCTION_POOL_SIMPLE_INFO, FUNCTION_REPAY_FLASH_SWAP_WITH_PARTNER, MODULE_FACTORY, MODULE_POOL,
    },
    model::{Hop, PoolRoute},
};
use crate::{
    Quote, SwapperError, SwapperQuoteData,
    clmm::{MAX_SQRT_PRICE, MIN_SQRT_PRICE},
    fees::{FeeSide, ReferralFee, subtract_bps},
    sui_swap::{self, inspect_transaction_kind_bytes, package_address, pay_referral_fee, shared_object_input},
};
use gem_sui::{
    ESTIMATION_GAS_BUDGET, SuiClient,
    models::{Coin, OwnedCoins, TxOutput},
    sui_clock_object_input,
    tx_builder::{ObjectResolver, PrefetchedTransactionData, TransactionBuilderInput, balance_value, balance_zero, build_input_coin, destroy_zero_balance, finish_transaction, from_balance, into_balance, move_call},
};
use std::{collections::HashMap, fmt::Display};
use sui_transaction_builder::{Argument, TransactionBuilder};

#[derive(Clone)]
pub(super) struct BuildInput<'a> {
    pub transaction: TransactionBuilderInput,
    pub amount: u64,
    pub from_coins: &'a OwnedCoins<Coin>,
}

impl BuildInput<'_> {
    fn with_gas_budget(&self, gas_budget: u64) -> Self {
        Self {
            transaction: self.transaction.with_gas_budget(gas_budget),
            ..self.clone()
        }
    }
}

pub(super) async fn build_quote_data(client: &SuiClient, quote: &Quote, route: &PoolRoute, referral_fee: &ReferralFee, published_at: &str) -> Result<SwapperQuoteData, SwapperError> {
    let sender = quote.request.wallet_address.as_str();
    let amount = quote.from_value_u64()?;
    let mut pinned = HashMap::from([(CETUS_GLOBAL_CONFIG.to_string(), CETUS_SHARED_INIT_VERSION), (CETUS_PARTNER.to_string(), CETUS_PARTNER_INIT_VERSION)]);
    let mut object_ids = vec![CETUS_GLOBAL_CONFIG.to_string(), CETUS_PARTNER.to_string()];
    for hop in &route.hops {
        pinned.insert(hop.pool_id.clone(), hop.pool_init_version);
        object_ids.push(hop.pool_id.clone());
    }
    let PrefetchedTransactionData { transaction, input_coins, resolver, .. } = PrefetchedTransactionData::prefetch(client, sender, route.input_coin_type(), None, object_ids, &pinned, ESTIMATION_GAS_BUDGET)
        .await
        .map_err(SwapperError::transaction_error)?;

    let input = BuildInput {
        transaction,
        amount,
        from_coins: &input_coins,
    };

    sui_swap::build_quote_data(client, |gas_budget| build_transaction(&resolver, quote, route, referral_fee, published_at, &input.with_gas_budget(gas_budget)))
        .await
        .map_err(|error| match error {
            SwapperError::TransactionError(detail) if detail.contains("checked_package_version") => sui_swap::upgraded_error("Cetus"),
            error => error,
        })
}

pub(super) fn build_batch_quote_inspect(quotes: &[(&Hop, u64)]) -> Result<Vec<u8>, SwapperError> {
    let package = package_address(CETUS_CLMM_PUBLISHED_AT)?;
    let mut txb = TransactionBuilder::new();
    for (hop, amount_in) in quotes {
        let pool = txb.object(shared_object_input(&hop.pool_id, hop.pool_init_version, false)?);
        let a2b = txb.pure(&hop.a2b);
        let by_amount_in = txb.pure(&true);
        let amount = txb.pure(amount_in);
        move_call(&mut txb, package, MODULE_POOL, FUNCTION_CALCULATE_SWAP_RESULT, &[&hop.coin_a, &hop.coin_b], vec![pool, a2b, by_amount_in, amount]).map_err(error)?;
    }
    inspect_transaction_kind_bytes(txb)
}

pub(super) fn build_batch_multi_hop_quote_inspect(routes: &[(&Hop, &Hop, u64)]) -> Result<Vec<u8>, SwapperError> {
    let package = package_address(CETUS_CLMM_PUBLISHED_AT)?;
    let mut txb = TransactionBuilder::new();
    for (hop1, hop2, amount_in) in routes {
        let pool1 = txb.object(shared_object_input(&hop1.pool_id, hop1.pool_init_version, false)?);
        let a2b1 = txb.pure(&hop1.a2b);
        let by_amount_in_1 = txb.pure(&true);
        let amount = txb.pure(amount_in);
        let csr1 = move_call(&mut txb, package, MODULE_POOL, FUNCTION_CALCULATE_SWAP_RESULT, &[&hop1.coin_a, &hop1.coin_b], vec![pool1, a2b1, by_amount_in_1, amount]).map_err(error)?;
        let amount2 = move_call(&mut txb, package, MODULE_POOL, FUNCTION_CALCULATED_SWAP_RESULT_AMOUNT_OUT, &[], vec![csr1]).map_err(error)?;
        let pool2 = txb.object(shared_object_input(&hop2.pool_id, hop2.pool_init_version, false)?);
        let a2b2 = txb.pure(&hop2.a2b);
        let by_amount_in_2 = txb.pure(&true);
        move_call(&mut txb, package, MODULE_POOL, FUNCTION_CALCULATE_SWAP_RESULT, &[&hop2.coin_a, &hop2.coin_b], vec![pool2, a2b2, by_amount_in_2, amount2]).map_err(error)?;
    }
    inspect_transaction_kind_bytes(txb)
}

pub(super) fn build_pool_id_inspect(coin_a: &str, coin_b: &str, tick_spacing: u32) -> Result<Vec<u8>, SwapperError> {
    let package = package_address(CETUS_CLMM_PUBLISHED_AT)?;
    let mut txb = TransactionBuilder::new();
    let tick = txb.pure(&tick_spacing);
    let key = move_call(&mut txb, package, MODULE_FACTORY, FUNCTION_NEW_POOL_KEY, &[coin_a, coin_b], vec![tick]).map_err(error)?;
    let pools = txb.object(shared_object_input(CETUS_POOLS_REGISTRY, CETUS_SHARED_INIT_VERSION, false)?);
    let info = move_call(&mut txb, package, MODULE_FACTORY, FUNCTION_POOL_SIMPLE_INFO, &[], vec![pools, key]).map_err(error)?;
    move_call(&mut txb, package, MODULE_FACTORY, FUNCTION_POOL_ID, &[], vec![info]).map_err(error)?;
    inspect_transaction_kind_bytes(txb)
}

struct PendingRepay<'a> {
    hop: &'a Hop,
    pool: Argument,
    pay_input: Argument,
    pay_zero_output: Argument,
    receipt: Argument,
}

fn build_transaction(resolver: &ObjectResolver, quote: &Quote, route: &PoolRoute, referral_fee: &ReferralFee, published_at: &str, input: &BuildInput<'_>) -> Result<TxOutput, SwapperError> {
    let mut txb = TransactionBuilder::new();
    let published_at = package_address(published_at)?;
    let input_coin = build_input_coin(&mut txb, route.input_coin_type(), input.amount, input.from_coins).map_err(error)?;
    let (swap_coin, swap_amount) = match route.fee.side {
        FeeSide::Input => {
            let after_fee = pay_referral_fee(&mut txb, input_coin, route.fee.value, referral_fee)?;
            let swap_amount = input
                .amount
                .checked_sub(route.fee.value)
                .ok_or_else(|| SwapperError::ComputeQuoteError("Cetus CLMM referral fee exceeds input amount".into()))?;
            (after_fee, swap_amount)
        }
        FeeSide::Output => (input_coin, input.amount),
    };

    let first_hop = route.hops.first().ok_or_else(|| SwapperError::TransactionError("Cetus CLMM route has no hops".into()))?;
    if first_hop.amount_in != swap_amount {
        return Err(SwapperError::TransactionError("Cetus CLMM first hop amount mismatch".into()));
    }

    let global_config = resolver.shared_object(&mut txb, CETUS_GLOBAL_CONFIG, false).map_err(error)?;
    let partner = resolver.shared_object(&mut txb, CETUS_PARTNER, true).map_err(error)?;
    let clock = txb.object(sui_clock_object_input());

    let mut current_balance = into_balance(&mut txb, route.input_coin_type(), swap_coin).map_err(error)?;
    let mut current_coin_type = route.input_coin_type().to_string();
    let mut pending_repays: Vec<PendingRepay<'_>> = Vec::new();

    for (idx, hop) in route.hops.iter().enumerate() {
        let pool = resolver.shared_object(&mut txb, &hop.pool_id, true).map_err(error)?;
        let pay_input = current_balance;
        let amount_arg = if idx == 0 {
            txb.pure(&hop.amount_in)
        } else {
            balance_value(&mut txb, &current_coin_type, current_balance).map_err(error)?
        };

        let zero_output = balance_zero(&mut txb, hop.output_coin_type()).map_err(error)?;
        let a2b_arg = txb.pure(&hop.a2b);
        let by_amount_in_arg = txb.pure(&true);
        let sqrt_price_limit_arg = txb.pure(&sqrt_price_limit_with_slippage(hop, quote.request.options.slippage.bps));

        let returns = move_call(
            &mut txb,
            published_at,
            MODULE_POOL,
            FUNCTION_FLASH_SWAP_WITH_PARTNER,
            &[&hop.coin_a, &hop.coin_b],
            vec![global_config, pool, partner, a2b_arg, by_amount_in_arg, amount_arg, sqrt_price_limit_arg, clock],
        )
        .map_err(error)?
        .to_nested(3);
        let (recv_a, recv_b, receipt) = match returns.as_slice() {
            [recv_a, recv_b, receipt] => (*recv_a, *recv_b, *receipt),
            _ => return Err(SwapperError::TransactionError("Cetus CLMM flash_swap did not return expected outputs".into())),
        };

        let (output_balance, empty_balance) = hop.order_by_direction(recv_b, recv_a);
        destroy_zero_balance(&mut txb, hop.input_coin_type(), empty_balance).map_err(error)?;

        pending_repays.push(PendingRepay {
            hop,
            pool,
            pay_input,
            pay_zero_output: zero_output,
            receipt,
        });

        current_balance = output_balance;
        current_coin_type = hop.output_coin_type().to_string();
    }

    for repay in pending_repays {
        let (pay_a, pay_b) = repay.hop.order_by_direction(repay.pay_input, repay.pay_zero_output);
        move_call(
            &mut txb,
            published_at,
            MODULE_POOL,
            FUNCTION_REPAY_FLASH_SWAP_WITH_PARTNER,
            &[&repay.hop.coin_a, &repay.hop.coin_b],
            vec![global_config, repay.pool, partner, pay_a, pay_b, repay.receipt],
        )
        .map_err(error)?;
    }

    let output_coin = from_balance(&mut txb, route.output_coin_type(), current_balance).map_err(error)?;
    let min_out = subtract_bps(&route.net_amount_out(), quote.request.options.slippage.bps);
    sui_swap::send_output(&mut txb, quote, route.output_coin_type(), output_coin, &route.fee, min_out, referral_fee)?;

    finish_transaction(txb, input.transaction.clone()).map_err(error)
}

fn sqrt_price_limit_with_slippage(hop: &Hop, slippage_bps: u32) -> u128 {
    if hop.after_sqrt_price == 0 {
        return if hop.a2b { MIN_SQRT_PRICE } else { MAX_SQRT_PRICE };
    }
    let bps = u128::from(slippage_bps);
    if hop.a2b {
        let limit = hop.after_sqrt_price.saturating_mul(10_000u128.saturating_sub(bps)) / 10_000;
        limit.max(MIN_SQRT_PRICE)
    } else {
        let limit = hop.after_sqrt_price.saturating_mul(10_000u128.saturating_add(bps)).saturating_add(9_999) / 10_000;
        limit.min(MAX_SQRT_PRICE)
    }
}

fn error(err: impl Display) -> SwapperError {
    SwapperError::transaction_error(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt_price_limit_with_slippage() {
        assert_eq!(sqrt_price_limit_with_slippage(&Hop::mock(), 50), MIN_SQRT_PRICE);
        assert_eq!(sqrt_price_limit_with_slippage(&Hop { a2b: false, ..Hop::mock() }, 50), MAX_SQRT_PRICE);

        let after = 100_000_000_000_000u128;
        let a2b_limit = sqrt_price_limit_with_slippage(&Hop { after_sqrt_price: after, ..Hop::mock() }, 50);
        assert_eq!(a2b_limit, after * 9_950 / 10_000);
        assert!(a2b_limit < after);

        let b2a_limit = sqrt_price_limit_with_slippage(
            &Hop {
                a2b: false,
                after_sqrt_price: after,
                ..Hop::mock()
            },
            50,
        );
        assert!(b2a_limit > after);
        assert!(b2a_limit <= MAX_SQRT_PRICE);
    }
}
