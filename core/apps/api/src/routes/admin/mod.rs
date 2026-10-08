mod addresses;
mod assets;
pub mod chain;
mod devices;
mod fiat;
mod lists;
mod nft;
mod prices;
mod transactions;

use axum::Router;
use axum::routing::{get, post, put};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/devices/{device_id}", get(devices::get_device))
        .route("/devices/{device_id}/subscriptions", get(devices::get_device_subscriptions))
        .route("/devices/{device_id}/wallets/{wallet_id}/subscriptions", get(devices::get_device_wallet_subscriptions))
        .route("/devices/{device_id}/transactions", get(devices::get_device_transactions))
        .route("/devices/{device_id}/fiat/transactions", get(devices::get_device_fiat_transactions))
        .route("/assets/add", post(assets::add_asset))
        .route("/assets/status", post(assets::get_asset_status))
        .route("/assets/associations/add", post(assets::add_asset_associations))
        .route("/transactions/{hash}", get(transactions::get_transactions_by_hash))
        .route("/transactions/add", post(transactions::add_transaction))
        .route("/addresses/refresh", post(addresses::refresh_addresses))
        .route("/prices/add", post(prices::add_price))
        .route("/lists/add", post(lists::add_list))
        .route("/nft/assets/update/{asset_id}", put(nft::update_nft_asset))
        .route("/nft/collections/update/{collection_id}", put(nft::update_nft_collection))
        .route("/fiat/quotes/{quote_type}", get(fiat::get_fiat_quotes))
        .nest("/chain", chain::router())
}
