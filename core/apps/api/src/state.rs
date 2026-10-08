use std::sync::Arc;

use ::defi::DefiProviderClient;
use ::nft::NFTProviderClient;
use axum::extract::FromRef;
use name_resolver::NameClient;
use services::access::AccessClient;
use services::app::ConfigClient;
use services::assets::{AssetsClient, SearchClient};
use services::auth::AuthClient;
use services::chain::{ChainClient, FeeEstimatesClient, NodesStatusClient};
use services::defi::DefiClient;
use services::devices::{DevicesClient, WalletConfigurationClient, WalletsClient};
use services::fiat::FiatClient;
use services::indexer::IndexerClient;
use services::nft::NFTClient;
use services::notifications::NotificationsClient;
use services::prices::{ChartClient, MarketsClient, PortfolioClient, PriceAlertClient, PriceClient};
use services::rewards::{RewardsClient, RewardsRedemptionClient};
use services::security::ScanClient;
use services::support::SupportApiClient;
use services::swap::{NearIntentsProxyClient, SwapClient, SwapsXyzProxyClient};
use services::transactions::{AddressDetailsClient, AddressNamesClient, TransactionsClient};
use services::webhooks::WebhooksClient;
use swapper::RpcClient;
use swapper::okx::OkxProviderProxy;
use swapper::swapper::GemSwapper;

use crate::auth::device::DeviceAuthConfig;
use crate::metrics::Metrics;
use crate::routes::devices::support::SupportImageUploadConfig;

pub struct Clients {
    pub metrics: Arc<Metrics>,
    pub auth_config: Arc<DeviceAuthConfig>,
    pub access: Arc<AccessClient>,
    pub fiat: Arc<FiatClient>,
    pub prices: Arc<PriceClient>,
    pub charts: Arc<ChartClient>,
    pub config: Arc<ConfigClient>,
    pub names: Arc<NameClient>,
    pub devices: Arc<DevicesClient>,
    pub assets: Arc<AssetsClient>,
    pub search: Arc<SearchClient>,
    pub transactions: Arc<TransactionsClient>,
    pub address_names: Arc<AddressNamesClient>,
    pub address_details: Arc<AddressDetailsClient>,
    pub wallet_configuration: Arc<WalletConfigurationClient>,
    pub scan: Arc<ScanClient>,
    pub swap: Arc<SwapClient>,
    pub nft: Arc<NFTClient>,
    pub nft_provider: Arc<NFTProviderClient>,
    pub defi: Arc<DefiClient>,
    pub defi_provider: Arc<DefiProviderClient>,
    pub price_alerts: Arc<PriceAlertClient>,
    pub chain: Arc<ChainClient>,
    pub fee_estimates: Arc<FeeEstimatesClient>,
    pub nodes_status: Arc<NodesStatusClient>,
    pub swapper: Arc<GemSwapper>,
    pub markets: Arc<MarketsClient>,
    pub webhooks: Arc<WebhooksClient>,
    pub indexer: Arc<IndexerClient>,
    pub rewards: Arc<RewardsClient>,
    pub rewards_redemption: Arc<RewardsRedemptionClient>,
    pub wallets: Arc<WalletsClient>,
    pub notifications: Arc<NotificationsClient>,
    pub support: Arc<SupportApiClient>,
    pub support_images: Arc<SupportImageUploadConfig>,
    pub near_intents: Arc<NearIntentsProxyClient>,
    pub swaps_xyz: Arc<SwapsXyzProxyClient>,
    pub okx: Arc<OkxProviderProxy<RpcClient>>,
    pub portfolio: Arc<PortfolioClient>,
    pub auth: Arc<AuthClient>,
}

#[derive(Clone)]
pub struct AppState(pub Arc<Clients>);

macro_rules! state_clients {
    ($($field:ident: $client:ty),* $(,)?) => {
        $(
            impl FromRef<AppState> for Arc<$client> {
                fn from_ref(state: &AppState) -> Self {
                    state.0.$field.clone()
                }
            }
        )*
    };
}

state_clients! {
    metrics: Metrics,
    auth_config: DeviceAuthConfig,
    access: AccessClient,
    fiat: FiatClient,
    prices: PriceClient,
    charts: ChartClient,
    config: ConfigClient,
    names: NameClient,
    devices: DevicesClient,
    assets: AssetsClient,
    search: SearchClient,
    transactions: TransactionsClient,
    address_names: AddressNamesClient,
    address_details: AddressDetailsClient,
    wallet_configuration: WalletConfigurationClient,
    scan: ScanClient,
    swap: SwapClient,
    nft: NFTClient,
    nft_provider: NFTProviderClient,
    defi: DefiClient,
    defi_provider: DefiProviderClient,
    price_alerts: PriceAlertClient,
    chain: ChainClient,
    fee_estimates: FeeEstimatesClient,
    nodes_status: NodesStatusClient,
    swapper: GemSwapper,
    markets: MarketsClient,
    webhooks: WebhooksClient,
    indexer: IndexerClient,
    rewards: RewardsClient,
    rewards_redemption: RewardsRedemptionClient,
    wallets: WalletsClient,
    notifications: NotificationsClient,
    support: SupportApiClient,
    support_images: SupportImageUploadConfig,
    near_intents: NearIntentsProxyClient,
    swaps_xyz: SwapsXyzProxyClient,
    okx: OkxProviderProxy<RpcClient>,
    portfolio: PortfolioClient,
    auth: AuthClient,
}
