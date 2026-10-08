mod auth;
mod error;
mod metrics;
mod model;
mod request;
mod response;
mod routes;
mod state;
mod stream;
#[cfg(test)]
mod testkit;

use std::net::SocketAddr;
use std::{error::Error, str::FromStr, sync::Arc};

use ::defi::{DefiProviderClient, DefiProviderConfig};
use ::nft::{NFTProviderClient, NFTProviderConfig};
use chain_providers::ProviderFactory;
use gem_tracing::info_with_fields;
use http_server::{ServeConfig, ShutdownReceiver, serve, shutdown_channel, spawn_signal_handler};
use model::APIService;
use name_resolver::{NameClient, NameConfig, NameProviderFactory};
use services::Services;
use settings::Settings;
use strum::IntoEnumIterator;
use swapper::okx::{OkxClientConfig, OkxProviderProxy};
use swapper::swapper::GemSwapper;
use tokio_util::task::TaskTracker;
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;

use crate::auth::device::{DeviceAuthConfig, JwtConfig};
use crate::routes::devices::support::SupportImageUploadConfig;
use crate::state::{AppState, Clients};
use crate::stream::{StreamObserverConfig, StreamState};

fn device_auth_config(settings: &Settings) -> Arc<DeviceAuthConfig> {
    let jwt = JwtConfig {
        secret: settings.api.auth.jwt.secret.clone(),
        expiry: settings.api.auth.jwt.expiry,
    };
    Arc::new(DeviceAuthConfig::new(settings.api.auth.tolerance, jwt))
}

async fn api_clients(settings: &Settings) -> Result<Clients, Box<dyn Error + Send + Sync>> {
    let services = Services::new(Arc::new(settings.clone()))?;
    let cacher_client = services.cacher().await?;

    let prices = services.prices(cacher_client.clone());
    let name_config = NameConfig {
        max_name_length: settings.name.max_name_length,
    };
    let user_agent = settings::service_user_agent("api", None);
    let endpoints = ProviderFactory::get_chain_endpoints(settings);
    let native_provider = Arc::new(swapper::NativeProvider::new_with_endpoints(endpoints));
    let stream_producer = services.stream_producer("api", services::no_shutdown()).await?;
    let providers = services.scan_providers(cacher_client.clone()).await?;
    let metrics = Arc::new(metrics::Metrics::new(&providers));
    let assets = services.assets();
    let ip_security = services.ip_security().await?;
    let deposit_addresses = Arc::new(cacher_client.clone());

    Ok(Clients {
        auth_config: device_auth_config(settings),
        access: Arc::new(services.access()),
        fiat: Arc::new(services.fiat(stream_producer.clone()).await?),
        charts: Arc::new(services.charts()),
        config: Arc::new(services.app_config().await?),
        names: Arc::new(NameClient::new(NameProviderFactory::new_providers(settings.name.clone()), name_config)),
        devices: Arc::new(services.devices()),
        search: Arc::new(services.search(cacher_client.clone()).await?),
        transactions: Arc::new(services.transactions()),
        address_names: Arc::new(services.address_names()),
        address_details: Arc::new(services.address_details(&user_agent)),
        wallet_configuration: Arc::new(services.wallet_configuration(cacher_client.clone(), &user_agent)),
        scan: Arc::new(services.scan(providers, cacher_client.clone(), metrics.clone())),
        swap: Arc::new(services.swap().await?),
        nft: Arc::new(services.nft()),
        nft_provider: Arc::new(NFTProviderClient::new(NFTProviderConfig::from_settings(settings))),
        defi: Arc::new(services.defi()),
        defi_provider: Arc::new(DefiProviderClient::new(DefiProviderConfig::from_settings(settings))),
        price_alerts: Arc::new(services.price_alerts()),
        chain: Arc::new(services.chain(&user_agent)),
        fee_estimates: Arc::new(services.fee_estimates(assets.clone(), prices.clone(), cacher_client.clone(), &user_agent)),
        nodes_status: Arc::new(services.nodes_status()),
        swapper: Arc::new(GemSwapper::new(native_provider.clone())),
        markets: Arc::new(services.markets(cacher_client.clone())),
        webhooks: Arc::new(services.webhooks(stream_producer.clone())),
        indexer: Arc::new(services.indexer(cacher_client.clone(), stream_producer.clone())),
        rewards: Arc::new(services.rewards(cacher_client.clone(), stream_producer.clone(), ip_security)),
        rewards_redemption: Arc::new(services.rewards_redemption(stream_producer.clone())),
        wallets: Arc::new(services.wallets(stream_producer, cacher_client)),
        notifications: Arc::new(services.notifications()),
        support: Arc::new(services.support_api()),
        support_images: Arc::new(SupportImageUploadConfig::new(&settings.support.types.images)?),
        near_intents: Arc::new(services.near_intents(deposit_addresses.clone())),
        swaps_xyz: Arc::new(services.swaps_xyz(deposit_addresses)),
        okx: Arc::new(OkxProviderProxy::new(
            settings.swap.okx.url.clone(),
            OkxClientConfig {
                api_key: settings.swap.okx.key.public.clone(),
                secret_key: settings.swap.okx.key.secret.clone(),
                passphrase: settings.swap.okx.passphrase.clone(),
                project: settings.swap.okx.project.clone(),
            },
            native_provider,
        )),
        portfolio: Arc::new(services.portfolio()),
        auth: Arc::new(services.auth().await?),
        assets: Arc::new(assets),
        prices: Arc::new(prices),
        metrics,
    })
}

async fn stream_state(settings: &Settings, shutdown: ShutdownReceiver) -> Result<StreamState, Box<dyn Error + Send + Sync>> {
    let services = Services::new(Arc::new(settings.clone()))?;
    let cacher_client = services.cacher().await?;
    Ok(StreamState {
        auth_config: device_auth_config(settings),
        devices: Arc::new(services.devices()),
        prices: Arc::new(services.prices(cacher_client.clone())),
        observer: Arc::new(StreamObserverConfig {
            redis_url: settings.redis.url.clone(),
            device_stream: services.device_stream(cacher_client).await?,
        }),
        shutdown,
        connections: TaskTracker::new(),
    })
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

    let address: SocketAddr = settings.api.bind.parse().map_err(|error| format!("invalid api.bind {}: {error}", settings.api.bind))?;
    let config = ServeConfig {
        header_read_timeout: settings.server.header.timeout,
        grace: settings.api.shutdown.timeout,
    };
    let (sender, shutdown) = shutdown_channel();
    spawn_signal_handler(sender);
    match service {
        APIService::Api => {
            let clients = api_clients(&settings).await?;
            let http_metrics = clients.metrics.http().clone();
            let router = routes::router(AppState(Arc::new(clients)), settings.api.admin.enabled, settings.server.request.timeout, &http_metrics);
            serve(NormalizePathLayer::trim_trailing_slash().layer(router), address, config, shutdown).await
        }
        APIService::WebsocketStream => {
            let state = stream_state(&settings, shutdown.clone()).await?;
            let connections = state.connections.clone();
            serve(stream::router(state), address, config, shutdown).await?;
            connections.close();
            tokio::time::timeout(settings.api.shutdown.timeout, connections.wait()).await.ok();
            Ok(())
        }
    }
}
