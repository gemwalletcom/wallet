use std::str::FromStr;
use std::sync::Arc;

use axum::extract::State;
use num_bigint::BigUint;
use primitives::{AssetId, swap::SwapResult};
use serde::Deserialize;
use swapper::{Options, QuoteRequest, SwapQuotes, SwapResultRequest, SwapperQuoteAsset, config::get_default_slippage, cross_chain::VaultAddresses, swapper::GemSwapper};

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{AddressParam, AssetIdParam, ChainQuery, Path, Query, SwapProviderParam};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct QuoteQuery {
    from_asset: AssetIdParam,
    to_asset: AssetIdParam,
    value: String,
    wallet_address: AddressParam,
    destination_address: AddressParam,
}

pub async fn get_swap_result(_permission: ChainRead, Path((provider, hash)): Path<(SwapProviderParam, String)>, Query(query): Query<ChainQuery>, State(swapper): State<Arc<GemSwapper>>) -> Result<ApiResponse<SwapResult>, ApiError> {
    Ok(swapper.get_swap_result(provider.0, &SwapResultRequest::new(query.chain.0, &hash)).await?.into())
}

pub async fn get_vault_addresses(_permission: ChainRead, Path(provider): Path<SwapProviderParam>, State(swapper): State<Arc<GemSwapper>>) -> Result<ApiResponse<VaultAddresses>, ApiError> {
    Ok(swapper.get_vault_addresses(&provider.0, None).await?.into())
}

pub async fn get_swap_quote(_permission: ChainRead, Query(query): Query<QuoteQuery>, State(swapper): State<Arc<GemSwapper>>) -> Result<ApiResponse<SwapQuotes>, ApiError> {
    let request = build_quote_request(query.from_asset.0, query.to_asset.0, &query.value, query.wallet_address.0, query.destination_address.0);
    Ok(swapper.get_quotes(&request).await?.into())
}

fn build_quote_request(from_asset_id: AssetId, to_asset_id: AssetId, value: &str, wallet_address: String, destination_address: String) -> QuoteRequest {
    let from_asset = SwapperQuoteAsset::from(from_asset_id.clone());
    let to_asset = SwapperQuoteAsset::from(to_asset_id);

    QuoteRequest {
        from_asset,
        to_asset,
        wallet_address,
        destination_address,
        value: BigUint::from_str(value).unwrap_or_default(),
        options: Options {
            slippage: get_default_slippage(&from_asset_id.chain),
            use_max_amount: false,
        },
    }
}
