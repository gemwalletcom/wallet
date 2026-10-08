mod assets;
mod config;
mod fee;
mod fiat;
mod markets;
mod nft;
mod prices;
mod swap;
mod webhooks;

use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/prices/{asset_id}", get(prices::get_price))
        .route("/prices", post(prices::get_assets_prices))
        .route("/charts/{asset_id}", get(prices::get_charts))
        .route("/fiat_rates", get(prices::get_fiat_rates))
        .route("/fiat/assets/{quote_type}", get(fiat::get_fiat_assets))
        .route("/config", get(config::get_config))
        .route("/assets/{asset_id}", get(assets::get_asset))
        .route("/assets", post(assets::get_assets))
        .route("/assets/search", get(assets::get_assets_search))
        .route("/search", get(assets::get_search))
        .route("/swap/assets", get(swap::get_swap_assets))
        .route("/swaps/near_intents/quote", post(swap::post_near_intents_quote))
        .route("/swaps/swaps_xyz/action", post(swap::post_swaps_xyz_action))
        .route("/swaps/providers/okx/v6/quote", post(swap::post_okx_quote_v6))
        .route("/swaps/providers/okx/v6/swap", post(swap::post_okx_swap_v6))
        .route("/nft/assets/{asset_id}/preview", get(nft::get_nft_asset_preview))
        .route("/nft/assets/{asset_id}/resource", get(nft::get_nft_asset_resource))
        .route("/nft/collections/{collection_id}/preview", get(nft::get_nft_collection_preview))
        .route("/markets", get(markets::get_markets))
        .route("/rewards/leaderboard", get(markets::get_rewards_leaderboard))
        .route("/chain/fee-estimates", get(fee::get_fee_estimates))
        .merge(webhooks::router())
}
