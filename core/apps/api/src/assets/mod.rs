pub mod client;
mod filter;
mod model;

use crate::params::{AssetIdParam, CurrencyParam};
use crate::responders::{ApiError, ApiResponse};
pub use client::{AssetsClient, SearchClient};
use model::SearchParams;
pub use model::SearchRequest;
use pricer::PriceClient;
use primitives::{AssetBasic, AssetFull, AssetId, SearchResponse};
use rocket::{State, get, post, serde::json::Json};

#[get("/assets/<asset_id>?<currency>")]
pub async fn get_asset(
    asset_id: AssetIdParam,
    currency: CurrencyParam,
    client: &State<AssetsClient>,
    price_client: &State<PriceClient>,
) -> Result<ApiResponse<AssetFull>, ApiError> {
    let asset = client.get_asset_full(&asset_id.0)?;
    let rate = price_client.get_fiat_rate(currency.0.as_ref())?.rate;
    Ok(asset.with_rate(rate).into())
}

#[post("/assets?<currency>", format = "json", data = "<asset_ids>")]
pub async fn get_assets(
    asset_ids: Json<Vec<AssetId>>,
    currency: CurrencyParam,
    client: &State<AssetsClient>,
    price_client: &State<PriceClient>,
) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let rate = price_client.get_fiat_rate(currency.0.as_ref())?.rate;

    Ok(client.get_assets(asset_ids.0, rate)?.into())
}

#[get("/assets/search?<params..>")]
pub async fn get_assets_search(params: SearchParams<'_>, client: &State<SearchClient>) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let request = SearchRequest::new(&params.query.0, params.chains, params.tags, params.limit.0, params.offset);
    Ok(client.get_assets_search(&request).await?.into())
}

#[get("/search?<params..>")]
pub async fn get_search(params: SearchParams<'_>, client: &State<SearchClient>) -> Result<ApiResponse<SearchResponse>, ApiError> {
    let request = SearchRequest::new(&params.query.0, params.chains, params.tags, params.limit.0, params.offset);

    let assets = client.get_assets_search(&request).await?;
    let lists = if request.should_search_lists() {
        client.get_asset_lists_search(&request).await?
    } else {
        vec![]
    };
    let perpetuals = client.get_perpetuals_search(&request).await?;
    let nfts = if request.has_tag_filter() { vec![] } else { client.get_nfts_search(&request).await? };

    Ok(SearchResponse { assets, perpetuals, nfts, lists }.into())
}
