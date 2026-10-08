mod addresses;
mod assets;
mod auth;
mod device;
mod fiat;
mod names;
mod nft;
mod notifications;
mod portfolio;
mod price_alerts;
mod rewards;
mod scan;
mod subscriptions;
pub mod support;
mod transactions;
mod wallet_configuration;

use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};

use crate::auth::device::{DeviceAuth, SignedPath, device_auth};
use crate::state::AppState;

const DEVICE_JSON_LIMIT: usize = 1024 * 1024;
const WALLET_SIGNED_LIMIT: usize = 32 * 1024 * 1024;

fn authenticated(router: Router<AppState>, state: &AppState, limit: usize, scheme: SignedPath) -> Router<AppState> {
    let auth = DeviceAuth {
        config: state.0.auth_config.clone(),
        limit,
        scheme,
        replay: (scheme == SignedPath::PathAndQuery).then(|| state.0.auth.clone()),
    };
    router.layer(from_fn_with_state(auth, device_auth))
}

pub fn v3_router(state: &AppState) -> Router<AppState> {
    let routes = Router::new()
        .route("/", post(device::register).get(device::get_device).put(device::update))
        .route("/is-registered", get(device::is_registered))
        .route("/auth/nonce", get(auth::get_nonce))
        .route("/assets", get(assets::get_assets))
        .route("/transactions", get(transactions::get_transactions))
        .route("/transactions/{id}", get(transactions::get_transaction_by_wallet))
        .route("/transactions/scan", post(scan::scan_transaction))
        .route("/address-names", post(addresses::get_address_names))
        .route("/addresses/{chain}/{address}", get(addresses::get_address_details))
        .route("/nft-assets", get(nft::get_nft_assets))
        .route("/nft-assets/report", post(nft::report_nft))
        .route("/nft-assets/{asset_id}", get(nft::get_nft_asset))
        .route("/nft-assets/{asset_id}/refresh", post(nft::refresh_nft_asset))
        .route("/rewards", get(rewards::get_rewards))
        .route("/names/{name}", get(names::get_name))
        .route("/wallet-configuration", get(wallet_configuration::get_wallet_configuration))
        .route("/notifications", get(notifications::get_notifications))
        .route("/notifications/read", post(notifications::mark_read))
        .route("/subscriptions", get(subscriptions::get_subscriptions).post(subscriptions::add_subscriptions).delete(subscriptions::delete_subscriptions))
        .route("/price-alerts", get(price_alerts::get_price_alerts).post(price_alerts::add_price_alerts).delete(price_alerts::delete_price_alerts))
        .route("/fiat/transactions", get(fiat::get_fiat_transactions))
        .route("/fiat/quotes/{quote_type}/{asset_id}", get(fiat::get_fiat_quotes))
        .route("/fiat/quotes/{quote_id}/url", get(fiat::get_fiat_quote_url))
        .route("/portfolio/assets", post(portfolio::get_portfolio_assets))
        .route("/support/messages", get(support::get_messages).post(support::post_message));
    let images = Router::new().route("/support/messages/images", post(support::post_image));
    let wallet_signed = Router::new()
        .route("/rewards/referrals", post(rewards::create_referral))
        .route("/rewards/referrals/use", post(rewards::use_referral_code))
        .route("/rewards/redemptions", post(rewards::redeem));
    authenticated(routes, state, DEVICE_JSON_LIMIT, SignedPath::PathAndQuery)
        .merge(authenticated(images, state, support::MAX_SUPPORT_IMAGE_BYTES, SignedPath::PathAndQuery))
        .merge(authenticated(wallet_signed, state, WALLET_SIGNED_LIMIT, SignedPath::PathAndQuery))
}

pub fn v2_router(state: &AppState) -> Router<AppState> {
    let routes = Router::new()
        .route("/", post(device::register).get(device::get_device).put(device::update))
        .route("/is_registered", get(device::is_registered))
        .route("/push-notification", post(device::send_push_notification))
        .route("/token", get(auth::get_token))
        .route("/auth/nonce", get(auth::get_nonce))
        .route("/assets", get(assets::get_assets))
        .route("/transactions", get(transactions::get_transactions))
        .route("/transactions/{id}", get(transactions::get_transaction_by_wallet))
        .route("/transaction/{id}", get(transactions::get_transaction))
        .route("/address_names", post(addresses::get_address_names))
        .route("/addresses/{chain}/{address}", get(addresses::get_address_details))
        .route("/nft_assets", get(nft::get_nft_assets))
        .route("/nft_assets/{asset_id}", get(nft::get_nft_asset))
        .route("/nft_assets/{asset_id}/refresh", post(nft::refresh_nft_asset))
        .route("/nft/report", post(nft::report_nft))
        .route("/defi/positions", get(assets::get_defi_positions))
        .route("/rewards", get(rewards::get_rewards))
        .route("/rewards/events", get(rewards::get_rewards_events))
        .route("/rewards/redemptions/{code}", get(rewards::get_redemption_option))
        .route("/name/resolve/{name}", get(names::get_name))
        .route("/scan/transaction", post(scan::scan_transaction))
        .route("/wallet_configuration", get(wallet_configuration::get_wallet_configuration))
        .route("/notifications", get(notifications::get_notifications))
        .route("/notifications/read", post(notifications::mark_read))
        .route("/subscriptions", get(subscriptions::get_subscriptions).post(subscriptions::add_subscriptions).delete(subscriptions::delete_subscriptions))
        .route("/price_alerts", get(price_alerts::get_price_alerts).post(price_alerts::add_price_alerts).delete(price_alerts::delete_price_alerts))
        .route("/fiat/transactions", get(fiat::get_fiat_transactions))
        .route("/fiat/assets/{quote_type}", get(fiat::get_fiat_assets))
        .route("/fiat/quotes/{quote_type}/{asset_id}", get(fiat::get_fiat_quotes))
        .route("/fiat/quotes/{quote_id}/url", get(fiat::get_fiat_quote_url))
        .route("/portfolio/assets", post(portfolio::get_portfolio_assets))
        .route("/support/messages", get(support::get_messages).post(support::post_message))
        .route("/support/action", post(support::post_action));
    let images = Router::new().route("/support/messages/images", post(support::post_image));
    let wallet_signed = Router::new()
        .route("/rewards/referrals/create", post(rewards::create_referral))
        .route("/rewards/referrals/use", post(rewards::use_referral_code))
        .route("/rewards/redeem", post(rewards::redeem));
    authenticated(routes, state, DEVICE_JSON_LIMIT, SignedPath::PathOnly)
        .merge(authenticated(images, state, support::MAX_SUPPORT_IMAGE_BYTES, SignedPath::PathOnly))
        .merge(authenticated(wallet_signed, state, WALLET_SIGNED_LIMIT, SignedPath::PathOnly))
}
