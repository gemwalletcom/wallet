use std::{
    collections::{BTreeSet, HashMap},
    fmt::{Debug, Display},
};

use gem_client::Client;
use gem_sui::{
    ESTIMATION_GAS_BUDGET, SuiClient,
    gas_budget::GAS_BUDGET_MULTIPLIER,
    models::{Coin, OwnedCoins, TxOutput},
    tx_builder::{ResolvedObjectInput, TransactionJsonReplay, build_input_coin, prepare_transaction_json_replay},
};
use num_bigint::BigUint;
use serde::Deserialize;
use sui_transaction_builder::{Argument, TransactionBuilder};

use crate::{
    SwapperError, SwapperQuoteData,
    mayan::{
        client::MayanClient,
        model::{GetSwapSuiParams, SuiClientSwap},
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
