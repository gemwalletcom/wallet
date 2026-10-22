use std::{collections::HashMap, fmt::Debug, sync::Arc};

use futures::try_join;
use gem_client::Client;
use gem_sui::{
    ESTIMATION_GAS_BUDGET, SuiClient,
    models::{Coin, OwnedCoins, TxOutput},
    tx_builder::{ResolvedObjectInput, TransactionBuilderInput, finish_transaction, move_call},
};
use sui_transaction_builder::{Argument, TransactionBuilder};
use sui_types::Address;

use super::{FAST_MCTP_PAYLOAD_TYPE_DEFAULT, circle_max_fee64, destination_referrer_address, fast_mctp_input_contract, fast_mctp_min_finality, redeem_relayer_fee, referrer_bytes, refund_relayer_fee64, token_out};
use crate::{
    Quote, RpcProvider, SwapperError, SwapperQuoteData,
    client_factory::create_sui_client,
    mayan::{
        cctp_domain::domain_for_wormhole_chain,
        client::MayanClient,
        constants::{SUI_CCTP_DENY_LIST, SUI_CCTP_V2_CORE_STATE, SUI_CCTP_V2_HANDLER_PACKAGE_ID, SUI_CCTP_V2_HANDLER_STATE, SUI_CCTP_V2_TOKEN_PACKAGE_ID, SUI_CCTP_V2_TOKEN_STATE, SUI_FAST_MCTP_STATE, SUI_USDC_TREASURY},
        model::{GetSwapSuiParams, MayanFastMctpQuote, QuoteType},
        tx_builder::{
            address::native_address_to_bytes32,
            amount::{gas_drop_amount, min_amount_out, optional_bps_u8},
            route::quote_destination_address,
            sui::{SuiClientSwapReplay, SuiInputCoin, bridge_amount, build_sui_input_coin, build_sui_quote_data, get_package_id, get_sui_input_coins, get_sui_objects, requires_client_swap, sui_error, sui_package_address},
        },
        wormhole_chain::id_for_name as wormhole_chain_id,
    },
    route_cache::Cache,
};

const BRIDGE_MODULE: &str = "bridge";
const CREATE_ORDER_MODULE: &str = "create_order";
const DEPOSIT_FOR_BURN_MODULE: &str = "deposit_for_burn";

struct PrefetchedFastMctpData {
    transaction: TransactionBuilderInput,
    input_coins: OwnedCoins<Coin>,
    objects: HashMap<String, ResolvedObjectInput>,
    package_id: String,
}

impl PrefetchedFastMctpData {
    async fn prefetch(client: &SuiClient, sender: &str, route: &MayanFastMctpQuote, package_id_cache: &Cache<&'static str, String>) -> Result<Self, SwapperError> {
        let input_contract = fast_mctp_input_contract(route)?;
        let object_ids = [
            SUI_FAST_MCTP_STATE,
            SUI_CCTP_V2_CORE_STATE,
            SUI_CCTP_V2_TOKEN_STATE,
            SUI_CCTP_V2_HANDLER_STATE,
            SUI_CCTP_DENY_LIST,
            SUI_USDC_TREASURY,
            input_currency_id(route)?,
            from_token_currency_id(route)?,
        ]
        .map(str::to_string)
        .to_vec();
        let transaction = async { TransactionBuilderInput::prefetch(client, sender, ESTIMATION_GAS_BUDGET).await.map_err(sui_error) };
        let input_coins = get_sui_input_coins(client, sender, route, input_contract);
        let objects = get_sui_objects(client, object_ids);
        let package_id = get_package_id(client, SUI_FAST_MCTP_STATE, package_id_cache);
        let (transaction, input_coins, objects, package_id) = try_join!(transaction, input_coins, objects, package_id)?;

        Ok(Self {
            transaction,
            input_coins,
            objects,
            package_id,
        })
    }

    fn object(&self, txb: &mut TransactionBuilder, object_id: &str, mutable: bool) -> Result<Argument, SwapperError> {
        let object = self.objects.get(object_id).ok_or_else(|| SwapperError::transaction_error(format!("Mayan Sui object was not prefetched: {object_id}")))?;
        Ok(txb.object(object.input(mutable)))
    }
}

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanFastMctpQuote, rpc_provider: Arc<dyn RpcProvider>, package_id_cache: &Cache<&'static str, String>) -> Result<SwapperQuoteData, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let sui_client = create_sui_client(rpc_provider)?;
    let prefetched = PrefetchedFastMctpData::prefetch(&sui_client, &quote.request.wallet_address, route, package_id_cache);
    let swap = get_swap_transaction(client, &sui_client, quote, route);
    let (prefetched, swap) = try_join!(prefetched, swap)?;

    build_sui_quote_data(&sui_client, |gas_budget| build_fast_mctp_transaction(quote, route, &prefetched, swap.as_ref(), gas_budget)).await
}

async fn get_swap_transaction<C>(client: &MayanClient<C>, sui_client: &SuiClient, quote: &Quote, route: &MayanFastMctpQuote) -> Result<Option<SuiClientSwapReplay>, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    if !requires_client_swap(route, fast_mctp_input_contract(route)?) {
        return Ok(None);
    }

    let params = GetSwapSuiParams::new(route, quote.request.wallet_address.clone(), fast_mctp_input_contract(route)?.to_string(), false, destination_referrer_address(route)?);
    SuiClientSwapReplay::fetch(client, sui_client, params).await.map(Some)
}

fn build_fast_mctp_transaction(quote: &Quote, route: &MayanFastMctpQuote, prefetched: &PrefetchedFastMctpData, swap: Option<&SuiClientSwapReplay>, gas_budget: u64) -> Result<TxOutput, SwapperError> {
    let input_contract = fast_mctp_input_contract(route)?;
    let SuiInputCoin { mut txb, coin: input_coin, .. } = build_sui_input_coin(swap, input_contract, route.effective_amount_in64.parse::<u64>()?, &prefetched.input_coins)?;
    let package = sui_package_address(&prefetched.package_id)?;

    let burn_ticket = if route.has_auction == Some(true) {
        add_create_order_move_calls(&mut txb, quote, route, prefetched, package, input_coin)?
    } else {
        add_bridge_move_calls(&mut txb, quote, route, prefetched, package, input_coin)?
    };
    add_circle_burn_move_calls(&mut txb, route, prefetched, burn_ticket)?;
    log_initialize_fast_mctp(&mut txb, route, prefetched, package)?;

    finish_transaction(txb, prefetched.transaction.with_gas_budget(gas_budget)).map_err(sui_error)
}

fn add_bridge_move_calls(txb: &mut TransactionBuilder, quote: &Quote, route: &MayanFastMctpQuote, prefetched: &PrefetchedFastMctpData, package: Address, input_coin: Argument) -> Result<Argument, SwapperError> {
    let arguments = vec![
        txb.pure(&FAST_MCTP_PAYLOAD_TYPE_DEFAULT),
        input_coin,
        txb.pure(&amount_in_min(route)?),
        txb.pure(&destination_address(quote, route)?),
        txb.pure(&domain_for_wormhole_chain(&route.to_chain)?.id()),
        txb.pure(&gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::FastMctp, false)?),
        txb.pure(&redeem_relayer_fee(route)?),
        txb.pure(&Address::new(referrer_bytes(route)?)),
        txb.pure(&optional_bps_u8(route.referrer_bps)?),
        txb.pure(&circle_max_fee64(route)?.parse::<u64>()?),
        txb.pure(&fast_mctp_min_finality(route)?),
        txb.pure(&Vec::<u8>::new()),
    ];
    let bridge_ticket = move_call(txb, package, BRIDGE_MODULE, "prepare_bridge", &[fast_mctp_input_contract(route)?], arguments).map_err(sui_error)?;
    add_burn_ticket_move_call(txb, route, prefetched, package, BRIDGE_MODULE, "bridge", bridge_ticket)
}

fn add_create_order_move_calls(txb: &mut TransactionBuilder, quote: &Quote, route: &MayanFastMctpQuote, prefetched: &PrefetchedFastMctpData, package: Address, input_coin: Argument) -> Result<Argument, SwapperError> {
    let arguments = vec![
        input_coin,
        txb.pure(&amount_in_min(route)?),
        txb.pure(&destination_address(quote, route)?),
        txb.pure(&domain_for_wormhole_chain(&route.to_chain)?.id()),
        txb.pure(&Address::new(token_out(route)?)),
        txb.pure(&min_amount_out(&route.min_amount_out, route.to_token.decimals, &route.to_chain, &QuoteType::FastMctp)?),
        txb.pure(&gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::FastMctp, false)?),
        txb.pure(&redeem_relayer_fee(route)?),
        txb.pure(&refund_relayer_fee64(route)?),
        txb.pure(&route.deadline64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?),
        txb.pure(&Address::new(referrer_bytes(route)?)),
        txb.pure(&optional_bps_u8(route.referrer_bps)?),
        txb.pure(&circle_max_fee64(route)?.parse::<u64>()?),
        txb.pure(&fast_mctp_min_finality(route)?),
    ];
    let order_ticket = move_call(txb, package, CREATE_ORDER_MODULE, "prepare_create_order", &[fast_mctp_input_contract(route)?], arguments).map_err(sui_error)?;
    add_burn_ticket_move_call(txb, route, prefetched, package, CREATE_ORDER_MODULE, "create_order", order_ticket)
}

fn add_burn_ticket_move_call(txb: &mut TransactionBuilder, route: &MayanFastMctpQuote, prefetched: &PrefetchedFastMctpData, package: Address, module: &str, function: &str, ticket: Argument) -> Result<Argument, SwapperError> {
    let state = prefetched.object(txb, SUI_FAST_MCTP_STATE, false)?;
    let input_currency = prefetched.object(txb, input_currency_id(route)?, false)?;
    move_call(txb, package, module, function, &[fast_mctp_input_contract(route)?], vec![state, input_currency, ticket]).map_err(sui_error)
}

fn add_circle_burn_move_calls(txb: &mut TransactionBuilder, route: &MayanFastMctpQuote, prefetched: &PrefetchedFastMctpData, burn_ticket: Argument) -> Result<(), SwapperError> {
    let input_contract = fast_mctp_input_contract(route)?;
    let token_package = sui_package_address(SUI_CCTP_V2_TOKEN_PACKAGE_ID)?;
    let handler_package = sui_package_address(SUI_CCTP_V2_HANDLER_PACKAGE_ID)?;

    let circle_auth = format!("{}::circle_auth::CircleAuth", prefetched.package_id);
    let token_state = prefetched.object(txb, SUI_CCTP_V2_TOKEN_STATE, false)?;
    let burn = move_call(
        txb,
        token_package,
        DEPOSIT_FOR_BURN_MODULE,
        "deposit_for_burn_with_package_auth",
        &[input_contract, &circle_auth],
        vec![burn_ticket, token_state],
    )
    .map_err(sui_error)?
    .to_nested(2);

    let handler_state = prefetched.object(txb, SUI_CCTP_V2_HANDLER_STATE, false)?;
    let deny_list = prefetched.object(txb, SUI_CCTP_DENY_LIST, false)?;
    let treasury = prefetched.object(txb, SUI_USDC_TREASURY, true)?;
    let complete_burn_ticket = move_call(txb, handler_package, "handler", "burn", &[], vec![handler_state, burn[0], burn[1], deny_list, treasury]).map_err(sui_error)?;

    let handler_auth = format!("{SUI_CCTP_V2_HANDLER_PACKAGE_ID}::handler::Auth");
    let token_state = prefetched.object(txb, SUI_CCTP_V2_TOKEN_STATE, false)?;
    let core_state = prefetched.object(txb, SUI_CCTP_V2_CORE_STATE, false)?;
    move_call(
        txb,
        token_package,
        DEPOSIT_FOR_BURN_MODULE,
        "complete_burn",
        &[input_contract, &handler_auth],
        vec![complete_burn_ticket, token_state, core_state],
    )
    .map_err(sui_error)?;
    Ok(())
}

fn log_initialize_fast_mctp(txb: &mut TransactionBuilder, route: &MayanFastMctpQuote, prefetched: &PrefetchedFastMctpData, package: Address) -> Result<(), SwapperError> {
    let amount = txb.pure(&route.effective_amount_in64.parse::<u64>()?);
    let from_currency = prefetched.object(txb, from_token_currency_id(route)?, false)?;
    let payload = txb.pure(&Vec::<u8>::new());
    move_call(txb, package, BRIDGE_MODULE, "log_initialize_fast_mctp", &[&route.from_token.contract], vec![amount, from_currency, payload]).map_err(sui_error)?;
    Ok(())
}

fn amount_in_min(route: &MayanFastMctpQuote) -> Result<u64, SwapperError> {
    bridge_amount(route, route.min_middle_amount.as_ref(), fast_mctp_input_contract(route)?)
}

fn destination_address(quote: &Quote, route: &MayanFastMctpQuote) -> Result<Address, SwapperError> {
    Ok(Address::new(native_address_to_bytes32(quote_destination_address(quote), wormhole_chain_id(&route.to_chain)?)?))
}

fn input_currency_id(route: &MayanFastMctpQuote) -> Result<&str, SwapperError> {
    route.fast_mctp_input_currency_id.as_deref().ok_or(SwapperError::InvalidRoute)
}

fn from_token_currency_id(route: &MayanFastMctpQuote) -> Result<&str, SwapperError> {
    route.from_token.currency_id.as_deref().ok_or(SwapperError::InvalidRoute)
}
