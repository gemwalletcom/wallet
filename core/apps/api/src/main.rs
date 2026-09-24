mod admin;
mod api_clients;
mod assets;
mod auth;
mod catchers;
mod chain;
mod config;
mod devices;
mod markets;
mod metrics;
mod model;
mod nft;
mod params;
mod prices;
mod referral;
mod responders;
mod status;
mod support;
mod swap;
#[cfg(test)]
mod testkit;
mod webhooks;
mod websocket;
mod websocket_stream;

use std::{error::Error, str::FromStr, sync::Arc};

use gem_tracing::info_with_fields;
use strum::IntoEnumIterator;

use ::defi::{DefiProviderClient, DefiProviderConfig};
use ::nft::{NFTProviderClient, NFTProviderConfig};
use chain_providers::ProviderFactory;
use model::APIService;
use name_resolver::{NameClient, NameConfig, NameProviderFactory};
use rocket::{Build, Rocket, catchers, routes};
use services::Services;
use settings::Settings;
use swapper::okx::{OkxClientConfig, OkxProviderProxy};
use swapper::swapper::GemSwapper;

use crate::support::SupportImageUploadConfig;

fn mount_routes(rocket: Rocket<Build>, admin_enabled: bool) -> Rocket<Build> {
    let rocket = rocket
        .mount("/", routes![status::get_status, status::get_health, metrics::get_metrics])
        .mount(
            "/v1",
            routes![
                prices::get_price,
                prices::get_assets_prices,
                prices::get_charts,
                prices::get_fiat_rates,
                devices::get_fiat_assets,
                webhooks::create_webhook,
                webhooks::create_webhook_with_header,
                config::get_config,
                assets::get_asset,
                assets::get_assets,
                assets::get_assets_search,
                assets::get_search,
                swap::get_swap_assets,
                nft::get_nft_asset_preview,
                nft::get_nft_asset_resource,
                nft::get_nft_collection_preview,
                markets::get_markets,
                referral::get_rewards_leaderboard,
                chain::fee::get_fee_estimates,
                swap::post_near_intents_quote,
                swap::post_swaps_xyz_action,
                swap::okx::post_okx_quote_v6,
                swap::okx::post_okx_swap_v6,
            ],
        )
        .mount(
            "/v2",
            routes![
                devices::get_device_fiat_transactions_v2,
                devices::get_device_fiat_assets_v2,
                devices::get_fiat_quotes_v2,
                devices::get_fiat_quote_url_v2,
                devices::add_device_v2,
                devices::get_device_v2,
                devices::is_device_registered_v2,
                devices::update_device_v2,
                devices::send_push_notification_device_v2,
                devices::report_device_nft_v2,
                devices::scan_device_transaction_v2,
                devices::get_device_assets_v2,
                devices::get_device_wallet_configuration_v2,
                devices::get_device_name_resolve_v2,
                devices::get_device_transaction_v2,
                devices::get_device_transaction_by_id_v2,
                devices::get_device_transactions_v2,
                devices::get_device_address_names_v2,
                devices::get_device_nft_assets_v2,
                devices::get_device_nft_asset_v2,
                devices::refresh_device_nft_asset_v2,
                devices::get_device_defi_positions_v2,
                devices::get_device_rewards_v2,
                devices::get_device_rewards_events_v2,
                devices::get_device_rewards_redemption_v2,
                devices::create_device_referral_v2,
                devices::use_device_referral_code_v2,
                devices::redeem_device_rewards_v2,
                support::get_support_messages,
                support::post_support_action,
                support::post_support_image,
                support::post_support_message,
                devices::get_device_notifications_v2,
                devices::mark_device_notifications_read_v2,
                devices::get_device_subscriptions_v2,
                devices::add_device_subscriptions_v2,
                devices::delete_device_subscriptions_v2,
                devices::get_device_price_alerts_v2,
                devices::add_device_price_alerts_v2,
                devices::delete_device_price_alerts_v2,
                devices::get_auth_nonce_v2,
                devices::get_device_token_v2,
                devices::get_device_portfolio_assets_v2,
            ],
        )
        .register("/", catchers![catchers::default_catcher]);

    if admin_enabled {
        rocket.mount(
            "/v1/admin",
            routes![
                admin::devices::get_device,
                admin::devices::get_device_subscriptions,
                admin::devices::get_device_wallet_subscriptions,
                admin::devices::get_device_transactions,
                admin::devices::get_device_fiat_transactions,
                admin::assets::add_asset,
                admin::assets::fetch_asset_status,
                admin::assets::add_asset_associations,
                admin::transactions::get_transactions_by_hash,
                admin::transactions::add_transaction,
                admin::addresses::refresh_addresses,
                admin::prices::add_price,
                admin::lists::add_list,
                admin::nft::update_nft_asset,
                admin::nft::update_nft_collection,
                admin::fiat::get_fiat_quotes,
                chain::block::get_latest_block_number,
                chain::block::get_block_transactions,
                chain::block::get_block_transactions_finalize,
                chain::fee::get_chain_fee_estimates,
                chain::node::get_nodes_status,
                chain::swap::get_swap_result,
                chain::swap::get_swap_quote,
                chain::swap::get_vault_addresses,
                chain::staking::get_validators,
                chain::staking::get_staking_apy,
                chain::token::get_token,
                chain::address::get_balances,
                chain::address::get_assets,
                chain::address::get_transactions,
                chain::defi::get_defi_positions,
                chain::nft::get_nfts,
                chain::nft::get_nft_asset,
                chain::nft::get_nft_collection,
                chain::transaction::get_transaction,
                chain::transaction::get_transaction_status,
            ],
        )
    } else {
        rocket
    }
}

async fn rocket_api(settings: Settings) -> Result<Rocket<Build>, Box<dyn Error + Send + Sync>> {
    let settings_clone = settings.clone();

    let services = Services::new(Arc::new(settings.clone()))?;
    let cacher_client = services.cacher().await?;

    let price_client = services.prices(cacher_client.clone());
    let charts_client = services.charts();
    let config_client = services.app_config();
    let price_alert_client = services.price_alerts();
    let name_config = NameConfig {
        max_name_length: settings_clone.name.max_name_length,
    };
    let name_client = NameClient::new(NameProviderFactory::new_providers(settings_clone.name.clone()), name_config);

    let user_agent = settings::service_user_agent("api", None);
    let chain_client = services.chain(&user_agent);
    let nodes_status_client = services.nodes_status();
    let portfolio_client = services.portfolio();
    let endpoints = ProviderFactory::get_chain_endpoints(&settings);
    let native_provider = Arc::new(swapper::NativeProvider::new_with_endpoints(endpoints));
    let swapper = GemSwapper::new(native_provider.clone());

    let devices_client = services.devices();
    let transactions_client = services.transactions();
    let address_names_client = services.address_names();
    let stream_producer = services.stream_producer("api", services::no_shutdown()).await?;
    let wallets_client = services.wallets(stream_producer.clone());

    let providers = services.scan_providers(cacher_client.clone()).await?;
    let metrics = Arc::new(metrics::Metrics::new(&providers));
    let scan_client = services.scan(providers, cacher_client.clone(), metrics.clone()).await?;
    let wallet_configuration_client = services.wallet_configuration(cacher_client.clone(), &user_agent);
    let assets_client = services.assets();
    let search_client = services.search(price_client.clone()).await?;
    let fee_estimates_client = services.fee_estimates(assets_client.clone(), price_client.clone(), cacher_client.clone(), &user_agent);
    let swap_client = services.swap();
    let fiat_client = services.fiat(stream_producer.clone()).await?;
    let nft_config = NFTProviderConfig::from_settings(&settings);
    let nft_client = services.nft();
    let nft_provider_client = NFTProviderClient::new(nft_config);
    let defi_client = services.defi();
    let defi_provider_client = DefiProviderClient::new(DefiProviderConfig::from_settings(&settings));
    let auth_client = services.auth().await?;
    let markets_client = services.markets(cacher_client.clone());
    let webhooks_client = services.webhooks(stream_producer.clone());
    let indexer_client = services.indexer(cacher_client.clone(), stream_producer.clone());
    let access_client = services.access();
    let ip_security_client = services.ip_security().await?;
    let rewards_client = services.rewards(cacher_client.clone(), stream_producer.clone(), ip_security_client);
    let redemption_client = services.rewards_redemption(stream_producer.clone());
    let notifications_client = services.notifications();
    let support_client = services.support_api();
    let support_image_upload_config = SupportImageUploadConfig::new(&settings.support.types.images)?;
    let near_intents_client = services.near_intents(cacher_client.clone());
    let swaps_xyz_client = services.swaps_xyz(cacher_client.clone());
    let okx_provider = OkxProviderProxy::new(
        settings.swap.okx.url.clone(),
        OkxClientConfig {
            api_key: settings.swap.okx.key.public.clone(),
            secret_key: settings.swap.okx.key.secret.clone(),
            passphrase: settings.swap.okx.passphrase.clone(),
            project: settings.swap.okx.project.clone(),
        },
        native_provider.clone(),
    );
    let jwt_config = devices::auth_config::JwtConfig {
        secret: settings.api.auth.jwt.secret.clone(),
        expiry: settings.api.auth.jwt.expiry,
    };
    let auth_config = devices::auth_config::AuthConfig::new(settings.api.auth.tolerance, jwt_config);
    let rocket = rocket::build()
        .manage(metrics)
        .manage(auth_config)
        .manage(access_client)
        .manage(fiat_client)
        .manage(price_client)
        .manage(charts_client)
        .manage(config_client)
        .manage(name_client)
        .manage(devices_client)
        .manage(assets_client)
        .manage(search_client)
        .manage(transactions_client)
        .manage(address_names_client)
        .manage(wallet_configuration_client)
        .manage(scan_client)
        .manage(swap_client)
        .manage(nft_client)
        .manage(nft_provider_client)
        .manage(defi_client)
        .manage(defi_provider_client)
        .manage(price_alert_client)
        .manage(chain_client)
        .manage(fee_estimates_client)
        .manage(nodes_status_client)
        .manage(swapper)
        .manage(markets_client)
        .manage(webhooks_client)
        .manage(indexer_client)
        .manage(rewards_client)
        .manage(redemption_client)
        .manage(wallets_client)
        .manage(notifications_client)
        .manage(support_client)
        .manage(support_image_upload_config)
        .manage(near_intents_client)
        .manage(swaps_xyz_client)
        .manage(okx_provider)
        .manage(portfolio_client)
        .manage(auth_client);

    Ok(mount_routes(rocket, settings.api.admin.enabled))
}

async fn rocket_ws_stream(settings: Settings) -> Result<Rocket<Build>, Box<dyn Error + Send + Sync>> {
    let services = Services::new(Arc::new(settings.clone()))?;
    let cacher_client = services.cacher().await?;
    let price_client = services.prices(cacher_client.clone());
    let stream_observer_config = websocket_stream::StreamObserverConfig {
        redis_url: settings.redis.url.clone(),
        device_stream: services.device_stream(cacher_client).await?,
    };

    let jwt_config = devices::auth_config::JwtConfig {
        secret: settings.api.auth.jwt.secret.clone(),
        expiry: settings.api.auth.jwt.expiry,
    };
    let auth_config = devices::auth_config::AuthConfig::new(settings.api.auth.tolerance, jwt_config);

    Ok(rocket::build()
        .manage(auth_config)
        .manage(services.devices())
        .manage(price_client)
        .manage(stream_observer_config)
        .mount("/v2/devices", routes![websocket_stream::ws_stream])
        .mount("/", routes![websocket_stream::ws_health])
        .register("/", catchers![catchers::default_catcher]))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let service = match std::env::args().nth(1) {
        Some(arg) => APIService::from_str(&arg).unwrap_or_else(|_| {
            let services: Vec<_> = APIService::iter().map(|s| format!("api {}", s.as_ref())).collect();
            panic!("unknown service: {arg}\nAvailable:\n {}", services.join("\n "))
        }),
        None => APIService::Api,
    };
    let settings = Settings::new()?.with_postgres_application_name(service.as_ref())?;

    info_with_fields!("api start service", service = service.as_ref());

    let rocket = match service {
        APIService::Api => rocket_api(settings).await?,
        APIService::WebsocketStream => rocket_ws_stream(settings).await?,
    };
    rocket.launch().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rocket::async_test]
    async fn test_no_route_collisions() {
        let rocket = mount_routes(rocket::build(), true);
        if let Err(e) = rocket.ignite().await {
            let error = format!("{:?}", e);
            assert!(!error.contains("Collisions"), "Route collisions detected: {error}");
        }
    }
}
