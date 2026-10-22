use super::sui_error;
use crate::{
    SwapperError,
    mayan::{
        constants::{SUI_CCTP_CORE_STATE, SUI_CCTP_DENY_LIST, SUI_CCTP_TOKEN_STATE, SUI_MCTP_FEE_MANAGER_STATE, SUI_MCTP_STATE, SUI_WORMHOLE_STATE},
        model::MayanMctpQuote,
        tx_builder::sui::{get_package_id, get_sui_input_coins, get_sui_objects},
    },
    route_cache::Cache,
};
use futures::try_join;
use gem_sui::{
    SuiClient,
    models::{Coin, OwnedCoins},
    tx_builder::{ResolvedObjectInput, TransactionBuilderInput},
};
use std::collections::HashMap;

pub(super) struct PrefetchedSuiData {
    pub(super) transaction: TransactionBuilderInput,
    pub(super) input_coins: OwnedCoins<Coin>,
    pub(super) objects: HashMap<String, ResolvedObjectInput>,
    pub(super) mctp_package_id: String,
    pub(super) fee_manager_package_id: Option<String>,
    pub(super) mctp_input_contract: String,
    pub(super) from_token_verified_address: String,
    pub(super) mctp_verified_input_address: String,
    pub(super) mctp_input_treasury: String,
}

impl PrefetchedSuiData {
    pub(super) async fn prefetch(client: &SuiClient, sender: &str, route: &MayanMctpQuote, gas_budget: u64, package_id_cache: &Cache<&'static str, String>) -> Result<Self, SwapperError> {
        let mctp_input_contract = route.mctp_input_contract.clone().ok_or(SwapperError::InvalidRoute)?;
        let from_token_verified_address = route.from_token.verified_address.clone().ok_or(SwapperError::InvalidRoute)?;
        let mctp_verified_input_address = route.mctp_verified_input_address.clone().ok_or(SwapperError::InvalidRoute)?;
        let mctp_input_treasury = route.mctp_input_treasury.clone().ok_or(SwapperError::InvalidRoute)?;
        let has_auction = route.has_auction == Some(true);
        let transaction = async { TransactionBuilderInput::prefetch(client, sender, gas_budget).await.map_err(sui_error) };
        let input_coins = get_sui_input_coins(client, sender, route, &mctp_input_contract);
        let objects = get_sui_objects(client, sui_object_ids(has_auction, &from_token_verified_address, &mctp_verified_input_address, &mctp_input_treasury));
        let mctp_package_id = get_package_id(client, SUI_MCTP_STATE, package_id_cache);
        let fee_manager_package_id = async {
            if has_auction {
                get_package_id(client, SUI_MCTP_FEE_MANAGER_STATE, package_id_cache).await.map(Some)
            } else {
                Ok(None)
            }
        };
        let (transaction, input_coins, objects, mctp_package_id, fee_manager_package_id) = try_join!(transaction, input_coins, objects, mctp_package_id, fee_manager_package_id)?;

        Ok(Self {
            transaction,
            input_coins,
            objects,
            mctp_package_id,
            fee_manager_package_id,
            mctp_input_contract,
            from_token_verified_address,
            mctp_verified_input_address,
            mctp_input_treasury,
        })
    }
}

fn sui_object_ids(has_auction: bool, from_token_verified_address: &str, mctp_verified_input_address: &str, mctp_input_treasury: &str) -> Vec<String> {
    [
        SUI_MCTP_STATE.to_string(),
        SUI_CCTP_CORE_STATE.to_string(),
        SUI_CCTP_TOKEN_STATE.to_string(),
        SUI_CCTP_DENY_LIST.to_string(),
        SUI_WORMHOLE_STATE.to_string(),
        from_token_verified_address.to_string(),
        mctp_verified_input_address.to_string(),
        mctp_input_treasury.to_string(),
    ]
    .into_iter()
    .chain(has_auction.then(|| SUI_MCTP_FEE_MANAGER_STATE.to_string()))
    .collect()
}
