mod admin;
mod api_clients;
mod assets;
mod auth;
mod catchers;
mod chain;
mod config;
mod devices;
mod markets;
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
mod websocket_prices;
mod websocket_stream;

use std::{error::Error, str::FromStr, sync::Arc};

use gem_tracing::info_with_fields;
use strum::IntoEnumIterator;

use ::defi::{DefiClient, DefiProviderClient, DefiProviderConfig};
use ::fiat::FiatClient;
use ::fiat::FiatProviderFactory;
use ::nft::{NFTClient, NFTProviderClient, NFTProviderConfig};
use api_connector::PusherClient;
use assets::{AssetsClient, SearchClient};
use cacher::CacherClient;
use config::ConfigClient;
use devices::DevicesClient;
use devices::{
    AddressNamesClient, FiatQuotesClient, NotificationsClient, PortfolioClient, RewardsClient, RewardsRedemptionClient, ScanClient, ScanProviderFactory, TransactionsClient,
    WalletConfigurationClient, WalletsClient,
};
use gem_auth::AuthClient;
use gem_rewards::{AbuseIPDBClient, IpApiClient, IpCheckProvider, IpSecurityClient};
use model::APIService;
use name_resolver::NameProviderFactory;
use name_resolver::client::{Client as NameClient, NameConfig};
use pricer::{ChartClient, MarketsClient, PriceAlertClient, PriceClient};
use primitives::{ConfigKey, PriceConfig};
use rocket::{Build, Rocket, catchers, routes};
use search_index::{SearchIndexClient, SearchIndexConfig};
use settings::Settings;
use settings_chain::{ChainProviders, ProviderFactory};
use storage::Database;
use streamer::{StreamProducer, StreamProducerConfig};
use swap::SwapClient;
use swapper::okx::{OkxClientConfig, OkxProviderProxy};
use swapper::swapper::GemSwapper;
use webhooks::WebhooksClient;
use websocket_prices::PriceObserverConfig;

use crate::support::{SupportApiClient, SupportImageUploadConfig};

fn mount_routes(rocket: Rocket<Build>, admin_enabled: bool) -> Rocket<Build> {
    let rocket = rocket
        .mount("/", routes![status::get_status, status::get_health])
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
                swap::okx::post_okx_quote_legacy,
                swap::okx::post_okx_quote_data_legacy,
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
    let redis_url = settings.redis.url.as_str();
    let postgres_url = settings.postgres.url.as_str();
    let settings_clone = settings.clone();

    let database = Database::new(postgres_url, settings.postgres.pool);
    let cacher_client = CacherClient::new(redis_url).await?;
    let config_cacher = storage::ConfigCacher::new(database.clone());
    let price_config = PriceConfig {
        primary_price_max_age: config_cacher.get_duration(primitives::ConfigKey::PricePrimaryMaxAge)?,
    };

    let price_client = PriceClient::new(database.clone(), cacher_client.clone());
    let charts_client = ChartClient::new(database.clone(), price_config);
    let config_client = ConfigClient::new(database.clone());
    let price_alert_client = PriceAlertClient::new(database.clone());
    let name_config = NameConfig {
        max_name_length: settings_clone.name.max_name_length,
    };
    let providers = NameProviderFactory::create_providers(settings_clone.clone());
    let name_client = NameClient::new(providers, name_config);

    let user_agent = settings::service_user_agent("api", None);
    let chain_client = chain::ChainClient::new(ChainProviders::from_settings(&settings, &user_agent));
    let nodes_status_client = chain::node::NodesStatusClient::default();
    let portfolio_client = PortfolioClient::new(database.clone(), price_config);
    let endpoints = ProviderFactory::get_chain_endpoints(&settings);
    let native_provider = Arc::new(swapper::NativeProvider::new_with_endpoints(endpoints));
    let swapper = GemSwapper::new(native_provider.clone());

    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let pusher_client = PusherClient::new(settings.pusher.url.clone(), settings.pusher.ios.topic.clone());
    let devices_client = DevicesClient::new(database.clone(), pusher_client.clone());
    let transactions_client = TransactionsClient::new(database.clone());
    let address_names_client = AddressNamesClient::new(database.clone());
    let stream_producer = StreamProducer::new(&rabbitmq_config, "api", streamer::no_shutdown()).await.unwrap();
    let wallets_client = WalletsClient::new(database.clone(), stream_producer.clone());

    let security_providers = ScanProviderFactory::create_providers(&settings_clone);
    let scan_client = ScanClient::new(database.clone(), security_providers);
    let wallet_configuration_client = WalletConfigurationClient::new(database.clone(), ChainProviders::from_settings(&settings, &user_agent), cacher_client.clone());
    let assets_client = AssetsClient::new(database.clone(), price_config);
    let search_index_config = SearchIndexConfig {
        batch_size: config_cacher.get_usize(ConfigKey::SearchIndexBatchSize)?,
    };
    let search_index_client = SearchIndexClient::new(&settings_clone.meilisearch.url, &settings_clone.meilisearch.key, search_index_config);
    let search_client = SearchClient::new(&search_index_client, price_client.clone());
    let swap_client = SwapClient::new(database.clone());
    let fiat_providers = FiatProviderFactory::new_providers(settings_clone.clone());
    let fiat_ip_check_client = FiatProviderFactory::new_ip_check_client(settings_clone.clone());
    let fiat_client = FiatClient::new(
        database.clone(),
        cacher_client.clone(),
        fiat_providers,
        fiat_ip_check_client.clone(),
        stream_producer.clone(),
    );
    let fiat_quotes_client = FiatQuotesClient::new(database.clone(), fiat_client);
    let nft_config = NFTProviderConfig::from_settings(&settings);
    let nft_client = NFTClient::from_config(database.clone(), nft_config.clone(), settings.nft.url.clone());
    let nft_provider_client = NFTProviderClient::new(nft_config);
    let defi_config = DefiProviderConfig::from_settings(&settings);
    let defi_client = DefiClient::from_config(database.clone(), defi_config.clone());
    let defi_provider_client = DefiProviderClient::new(defi_config);
    let auth_client = AuthClient::new(cacher_client.clone());
    let markets_client = MarketsClient::new(database.clone(), cacher_client.clone());
    let webhooks_client = WebhooksClient::new(stream_producer.clone(), settings.support.webhook.key.secret.clone());
    let ip_check_providers: Vec<Arc<dyn IpCheckProvider>> = vec![
        Arc::new(AbuseIPDBClient::new(settings.ip.abuseipdb.url.clone(), settings.ip.abuseipdb.key.secret.clone())),
        Arc::new(IpApiClient::new(settings.ip.ipapi.url.clone(), settings.ip.ipapi.key.secret.clone())),
    ];
    let ip_security_client = IpSecurityClient::new(ip_check_providers, cacher_client.clone());
    let rewards_client = RewardsClient::new(database.clone(), stream_producer.clone(), ip_security_client, pusher_client.clone());
    let redemption_client = RewardsRedemptionClient::new(database.clone(), stream_producer.clone());
    let notifications_client = NotificationsClient::new(database.clone());
    let support_client = SupportApiClient::new(
        settings.support.url.clone(),
        settings.support.widget.ios.clone(),
        settings.support.widget.android.clone(),
        database.clone(),
    );
    let support_image_upload_config = SupportImageUploadConfig::new(&settings.support.types.images)?;
    let near_intents_client = swap::NearIntentsProxyClient::new(cacher_client.clone());
    let swaps_xyz_client = swap::SwapsXyzProxyClient::new(cacher_client.clone());
    let okx_provider = OkxProviderProxy::new(
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
        .manage(auth_config)
        .manage(database)
        .manage(fiat_quotes_client)
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
        .manage(nodes_status_client)
        .manage(swapper)
        .manage(markets_client)
        .manage(webhooks_client)
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
        .manage(auth_client)
        .manage(cacher_client)
        .manage(stream_producer);

    Ok(mount_routes(rocket, settings.api.admin.enabled))
}

async fn rocket_ws_prices(settings: Settings) -> Result<Rocket<Build>, Box<dyn Error + Send + Sync>> {
    let cacher_client = CacherClient::new(&settings.redis.url).await?;
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let price_client = PriceClient::new(database, cacher_client);
    let price_observer_config = PriceObserverConfig {
        redis_url: settings.redis.url.clone(),
    };
    Ok(rocket::build()
        .manage(price_client)
        .manage(price_observer_config)
        .mount("/", routes![websocket_prices::ws_health])
        .mount("/v1/ws", routes![websocket_prices::ws_prices])
        .register("/", catchers![catchers::default_catcher]))
}

async fn rocket_ws_stream(settings: Settings) -> Result<Rocket<Build>, Box<dyn Error + Send + Sync>> {
    let cacher_client = CacherClient::new(&settings.redis.url).await?;
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config_cacher = storage::ConfigCacher::new(database.clone());
    let price_client = PriceClient::new(database.clone(), cacher_client.clone());
    let stream_observer_config = websocket_stream::StreamObserverConfig {
        redis_url: settings.redis.url.clone(),
        cacher_client,
        retention: config_cacher.get_duration(primitives::ConfigKey::DeviceStreamRetention)?,
        history_limit: config_cacher.get_usize(primitives::ConfigKey::DeviceStreamHistoryLimit)?,
    };

    let jwt_config = devices::auth_config::JwtConfig {
        secret: settings.api.auth.jwt.secret.clone(),
        expiry: settings.api.auth.jwt.expiry,
    };
    let auth_config = devices::auth_config::AuthConfig::new(settings.api.auth.tolerance, jwt_config);

    Ok(rocket::build()
        .manage(auth_config)
        .manage(database)
        .manage(price_client)
        .manage(stream_observer_config)
        .mount("/v2/devices", routes![websocket_stream::ws_stream])
        .mount("/", routes![websocket_stream::ws_health])
        .register("/", catchers![catchers::default_catcher]))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Settings::new()?;

    let service = match std::env::args().nth(1) {
        Some(arg) => APIService::from_str(&arg).unwrap_or_else(|_| {
            let services: Vec<_> = APIService::iter().map(|s| format!("api {}", s.as_ref())).collect();
            panic!("unknown service: {arg}\nAvailable:\n {}", services.join("\n "))
        }),
        None => APIService::Api,
    };

    info_with_fields!("api start service", service = service.as_ref());

    let rocket = match service {
        APIService::Api => rocket_api(settings).await?,
        APIService::WebsocketPrices => rocket_ws_prices(settings).await?,
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
