mod bridge;
pub(super) mod fees;
mod order;

use self::{
    bridge::{add_bridge_locked_fee_move_calls, add_bridge_with_fee_move_calls},
    order::add_init_order_move_calls,
};
use super::{context::MctpSuiContext, sui_error};
use crate::mayan::{
    constants::{SUI_CCTP_CORE_STATE, SUI_CCTP_DENY_LIST, SUI_CCTP_TOKEN_PACKAGE_ID, SUI_CCTP_TOKEN_STATE, SUI_LOGGER_PACKAGE_ID, SUI_WORMHOLE_PACKAGE_ID, SUI_WORMHOLE_STATE},
    model::MayanMctpQuote,
    wormhole_chain::WormholeChain,
};
use crate::{
    Quote, SwapperError,
    mayan::tx_builder::{
        amount::optional_bps_u8,
        sui::{SuiClientSwapReplay, SuiInputCoin, WormholeFeeCoin, build_sui_input_coin},
        swift::referrer_bytes,
    },
    sui_swap::package_address,
};
use gem_sui::tx_builder::{Argument, TransactionBuilder};
use gem_sui::types::Address;
use gem_sui::{
    models::TxOutput,
    sui_clock_object_input,
    tx_builder::{finish_transaction, move_call},
};

pub(super) fn build_mctp_transaction(quote: &Quote, route: &MayanMctpQuote, context: &MctpSuiContext, destination_address: &str, swap: Option<&SuiClientSwapReplay>, gas_budget: u64) -> Result<TxOutput, SwapperError> {
    let SuiInputCoin { mut txb, coin: input_coin, wh_fee_coin } = build_sui_input_coin(swap, &context.mctp_input_contract, route.effective_amount_in64.parse::<u64>()?, &context.input_coins, context.transaction.address_balance)?;

    if route.has_auction == Some(true) {
        add_init_order_move_calls(&mut txb, route, quote, context, input_coin, wh_fee_coin, destination_address)?;
    } else if route.cheaper_chain.as_deref() == Some(WormholeChain::Sui.name()) {
        add_bridge_locked_fee_move_calls(&mut txb, route, context, input_coin, destination_address)?;
    } else {
        add_bridge_with_fee_move_calls(&mut txb, route, context, input_coin, wh_fee_coin, destination_address)?;
    }

    log_initialize_mctp(&mut txb, route, context)?;
    log_referrer(&mut txb, route)?;

    finish_transaction(txb, context.transaction.with_gas_budget(gas_budget)).map_err(sui_error)
}

fn deposit_for_burn_with_auth(txb: &mut TransactionBuilder, context: &MctpSuiContext, auth_module: &str, deposit_ticket: Argument) -> Result<Argument, SwapperError> {
    let cctp_token_state = txb.object(context.objects[SUI_CCTP_TOKEN_STATE].input(true));
    let cctp_core_state = txb.object(context.objects[SUI_CCTP_CORE_STATE].input(true));
    let deny_list = txb.object(context.objects[SUI_CCTP_DENY_LIST].input(false));
    let treasury = txb.object(context.objects[&context.mctp_input_treasury].input(true));
    let auth_type = format!("{}::{auth_module}::CircleAuth", context.mctp_package_id);
    move_call(
        txb,
        package_address(SUI_CCTP_TOKEN_PACKAGE_ID)?,
        "deposit_for_burn",
        "deposit_for_burn_with_caller_with_package_auth",
        &[&context.mctp_input_contract, &auth_type],
        vec![deposit_ticket, cctp_token_state, cctp_core_state, deny_list, treasury],
    )
    .map_err(sui_error)
}

fn add_publish_wormhole_message(txb: &mut TransactionBuilder, context: &MctpSuiContext, wormhole_message: Argument, bridge_fee: u64, wh_fee_coin: WormholeFeeCoin) -> Result<(), SwapperError> {
    let fee_coin = wh_fee_coin.build_coin(txb, bridge_fee)?;
    let clock = txb.object(sui_clock_object_input());
    let wormhole_state = txb.object(context.objects[SUI_WORMHOLE_STATE].input(true));
    move_call(
        txb,
        package_address(SUI_WORMHOLE_PACKAGE_ID)?,
        "publish_message",
        "publish_message",
        &[],
        vec![wormhole_state, fee_coin, wormhole_message, clock],
    )
    .map_err(sui_error)?;
    Ok(())
}

fn log_initialize_mctp(txb: &mut TransactionBuilder, route: &MayanMctpQuote, context: &MctpSuiContext) -> Result<(), SwapperError> {
    let amount = route.effective_amount_in64.parse::<u64>()?;
    let verified_input = txb.object(context.objects[&context.from_token_verified_address].input(false));
    let amount = txb.pure(&amount);
    let payload = txb.pure(&Vec::<u8>::new());
    move_call(
        txb,
        package_address(&context.mctp_package_id)?,
        "init_order",
        "log_initialize_mctp",
        &[&route.from_token.contract],
        vec![amount, verified_input, payload],
    )
    .map_err(sui_error)?;
    Ok(())
}

fn log_referrer(txb: &mut TransactionBuilder, route: &MayanMctpQuote) -> Result<(), SwapperError> {
    let referrer = referrer_bytes(&route.to_chain)?;
    let referrer = txb.pure(&Address::new(referrer));
    let referrer_bps = txb.pure(&optional_bps_u8(route.referrer_bps)?);
    move_call(txb, package_address(SUI_LOGGER_PACKAGE_ID)?, "referrer_logger", "log_referrer", &[], vec![referrer, referrer_bps]).map_err(sui_error)?;
    Ok(())
}
