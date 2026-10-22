use std::{
    collections::{BTreeSet, HashMap},
    fmt::{Debug, Display},
};

use gem_client::Client;
use gem_sui::{
    ESTIMATION_GAS_BUDGET, SuiClient,
    address::SuiAddress,
    gas_budget::GAS_BUDGET_MULTIPLIER,
    is_sui_coin,
    models::{Coin, OwnedCoins, TxOutput},
    tx_builder::{ResolvedObjectInput, TransactionJsonReplay, build_input_coin, prepare_transaction_json_replay},
};
use num_bigint::BigUint;
use serde::Deserialize;
use serde_json::Value;
use sui_transaction_builder::{Argument, TransactionBuilder};
use sui_types::Address;

use crate::{
    SwapperError, SwapperQuoteData,
    mayan::{
        cctp_domain::CCTP_TOKEN_DECIMALS,
        client::MayanClient,
        model::{GetSwapSuiParams, MayanQuoteCommon, SuiClientSwap},
        tx_builder::amount::fractional_amount,
    },
    route_cache::Cache,
};

pub(in crate::mayan::tx_builder) struct SuiClientSwapReplay {
    swap: SuiClientSwap,
    replay: TransactionJsonReplay,
}

impl SuiClientSwapReplay {
    pub(in crate::mayan::tx_builder) async fn fetch<C>(client: &MayanClient<C>, sui_client: &SuiClient, params: GetSwapSuiParams) -> Result<Self, SwapperError>
    where
        C: Client + Clone + Send + Sync + Debug + 'static,
    {
        let swap: SuiClientSwap = client.get_swap_sui(params).await?;
        let replay = prepare_transaction_json_replay(sui_client, &swap.tx).await.map_err(sui_error)?;
        Ok(Self { swap, replay })
    }
}

pub(in crate::mayan::tx_builder) struct SuiInputCoin {
    pub txb: TransactionBuilder,
    pub coin: Argument,
    pub wh_fee_coin: Option<Argument>,
}

pub(in crate::mayan::tx_builder) fn build_sui_input_coin(swap: Option<&SuiClientSwapReplay>, coin_type: &str, amount: u64, coins: &OwnedCoins<Coin>) -> Result<SuiInputCoin, SwapperError> {
    if let Some(swap) = swap {
        let replayed = swap.replay.replay().map_err(sui_error)?;
        let coin = replayed.argument(&swap.swap.out_coin).map_err(sui_error)?;
        let wh_fee_coin = swap.swap.wh_fee_coin.as_ref().map(|argument| replayed.argument(argument).map_err(sui_error)).transpose()?;
        return Ok(SuiInputCoin { txb: replayed.txb, coin, wh_fee_coin });
    }

    let mut txb = TransactionBuilder::new();
    let coin = build_input_coin(&mut txb, coin_type, amount, coins).map_err(sui_error)?;
    Ok(SuiInputCoin { txb, coin, wh_fee_coin: None })
}

pub(in crate::mayan::tx_builder) fn requires_client_swap(route: &MayanQuoteCommon, input_contract: &str) -> bool {
    route.from_token.contract != input_contract
}

pub(in crate::mayan::tx_builder) fn bridge_amount(route: &MayanQuoteCommon, min_middle_amount: Option<&Value>, input_contract: &str) -> Result<u64, SwapperError> {
    if !requires_client_swap(route, input_contract) {
        return route.effective_amount_in64.parse::<u64>().map_err(SwapperError::from);
    }
    fractional_amount(min_middle_amount.ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)
}

pub(in crate::mayan::tx_builder) async fn get_sui_input_coins(client: &SuiClient, sender: &str, route: &MayanQuoteCommon, input_contract: &str) -> Result<OwnedCoins<Coin>, SwapperError> {
    if requires_client_swap(route, input_contract) || is_sui_coin(input_contract) {
        return Ok(OwnedCoins::default());
    }
    client.get_coins(sender, input_contract).await.map_err(sui_error)
}

pub(in crate::mayan::tx_builder) fn sui_package_address(package_id: &str) -> Result<Address, SwapperError> {
    SuiAddress::parse(package_id).map(Address::from).map_err(sui_error)
}

pub(in crate::mayan::tx_builder) async fn get_sui_objects(client: &SuiClient, object_ids: Vec<String>) -> Result<HashMap<String, ResolvedObjectInput>, SwapperError> {
    let object_ids: Vec<String> = object_ids.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
    let objects = ResolvedObjectInput::get_multiple(client, object_ids.clone()).await.map_err(sui_error)?;
    if objects.len() != object_ids.len() {
        return Err(SwapperError::transaction_error("Failed to prefetch all Mayan Sui objects"));
    }
    Ok(object_ids.into_iter().zip(objects).collect())
}

pub(in crate::mayan::tx_builder) async fn get_package_id(client: &SuiClient, state_object_id: &'static str, package_id_cache: &Cache<&'static str, String>) -> Result<String, SwapperError> {
    if let Some(package_id) = package_id_cache.get(&state_object_id) {
        return Ok(package_id);
    }
    let state: MayanStateObject = client.get_object_json(state_object_id.to_string()).await.map_err(sui_error)?;
    let package_id = state.latest_package_id();
    package_id_cache.put(state_object_id, package_id.clone());
    Ok(package_id)
}

pub(in crate::mayan::tx_builder) async fn build_sui_quote_data(sui_client: &SuiClient, build_transaction: impl Fn(u64) -> Result<TxOutput, SwapperError>) -> Result<SwapperQuoteData, SwapperError> {
    let estimate = build_transaction(ESTIMATION_GAS_BUDGET)?;
    let dry_run = sui_client.dry_run(estimate.base64_encoded()).await.map_err(SwapperError::transaction_error)?;
    if dry_run.effects.status.status != "success" {
        let detail = dry_run.effects.status.error.as_deref().unwrap_or("no details available");
        return Err(SwapperError::TransactionError(format!("Sui swap simulation failed: {detail}")));
    }

    let fee = dry_run.effects.gas_used.calculate_gas_budget().map_err(SwapperError::transaction_error)?;
    let gas_budget = fee * GAS_BUDGET_MULTIPLIER / 100;
    let output = build_transaction(gas_budget)?;

    Ok(SwapperQuoteData::new_contract(String::new(), BigUint::from(0u64), output.base64_encoded(), None, Some(gas_budget.to_string())))
}

pub(in crate::mayan::tx_builder) fn sui_error(err: impl Display) -> SwapperError {
    SwapperError::TransactionError(format!("Sui transaction error: {err}"))
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MayanStateObject {
    Direct(MayanStateFields),
    Fields { fields: MayanStateFields },
    Content { content: MayanStateContent },
    Data { data: MayanStateContent },
}

impl MayanStateObject {
    fn latest_package_id(self) -> String {
        match self {
            Self::Direct(fields) => fields.latest_package_id,
            Self::Fields { fields } => fields.latest_package_id,
            Self::Content { content } => content.fields.latest_package_id,
            Self::Data { data } => data.fields.latest_package_id,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MayanStateContent {
    fields: MayanStateFields,
}

#[derive(Debug, Deserialize)]
struct MayanStateFields {
    #[serde(alias = "latestPackageId")]
    latest_package_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mayan::model::{MayanFastMctpQuote, MayanMctpQuote};
    use gem_sui::SUI_COIN_TYPE;

    #[test]
    fn test_bridge_amount() {
        let mut route = MayanMctpQuote::mock();
        let input_contract = route.mctp_input_contract.clone().unwrap();
        assert_eq!(bridge_amount(&route, route.min_middle_amount.as_ref(), &input_contract).unwrap(), 1_000_000);

        route.common.from_token.contract = SUI_COIN_TYPE.to_string();
        route.min_middle_amount = Some(serde_json::json!(0.99));
        assert_eq!(bridge_amount(&route, route.min_middle_amount.as_ref(), &input_contract).unwrap(), 990_000);

        let route = MayanFastMctpQuote::mock_sui();
        let input_contract = route.fast_mctp_input_contract.clone().unwrap();
        assert_eq!(bridge_amount(&route, route.min_middle_amount.as_ref(), &input_contract).unwrap(), 1_038_528);
    }

    #[test]
    fn test_mayan_state_object_latest_package_id() {
        let state: MayanStateObject = serde_json::from_value(serde_json::json!({
            "latest_package_id": "0x123"
        }))
        .unwrap();
        assert_eq!(state.latest_package_id(), "0x123");

        let state: MayanStateObject = serde_json::from_value(serde_json::json!({
            "fields": {
                "latest_package_id": "0xabc"
            }
        }))
        .unwrap();
        assert_eq!(state.latest_package_id(), "0xabc");

        let state: MayanStateObject = serde_json::from_value(serde_json::json!({
            "content": {
                "fields": {
                    "latestPackageId": "0xdef"
                }
            }
        }))
        .unwrap();
        assert_eq!(state.latest_package_id(), "0xdef");
    }
}
