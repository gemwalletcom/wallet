mod dev_seed;

use std::collections::{HashMap, HashSet};
use std::error::Error;

use async_trait::async_trait;
use chrono::Utc;
use config_keys::{ConfigKey, ConfigParamKey};
use gem_tracing::info_with_fields;
use primitives::currency::Currency;
use primitives::{AssetBasic, AssetTag, Chain, ChartTimeframe, FiatProviderName, FiatRate, FiatRateProvider, PriceProvider, Release, ScanAddress};
use storage::{
    ApiClientGrant, ApiClientsRepository, AssetsRepository, ChainsRepository, ChartsRepository, ConfigRepository, Database, DatabaseError, DevicesRepository, FiatRepository, MigrationsRepository, NotificationsRepository,
    ParserStateRepository, PriceAlertsRepository, PricesProvidersRepository, PricesRepository, ReleasesRepository, ScanAddressesRepository, TagRepository, WalletsRepository,
};

use crate::setup::api_clients::{SETUP_DEV_API_CLIENT_NAME, SETUP_DEV_API_CLIENT_SECRET, api_client_access_grants};

pub(crate) struct SetupSeed {
    pub(crate) chains: Vec<Chain>,
    pub(crate) assets: Vec<AssetBasic>,
    pub(crate) fiat_providers: Vec<FiatProviderName>,
    pub(crate) api_client_grants: Vec<ApiClientGrant>,
    pub(crate) releases: Vec<Release>,
    pub(crate) tags: Vec<AssetTag>,
    pub(crate) price_providers: Vec<PriceProvider>,
    pub(crate) config_keys: Vec<ConfigKey>,
    pub(crate) config_params: Vec<ConfigParamKey>,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn update_schema(&self) -> Result<(), DatabaseError>;
    async fn set_seed(&self, seed: SetupSeed) -> Result<(), DatabaseError>;
    async fn add_missing_scan_addresses(&self, addresses: HashMap<(Chain, String), ScanAddress>) -> Result<usize, DatabaseError>;
    async fn set_dev_seed(&self) -> Result<i32, Box<dyn Error + Send + Sync>>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn update_schema(&self) -> Result<(), DatabaseError> {
        self.database.run(MigrationsRepository::update_schema).await
    }

    async fn set_seed(&self, seed: SetupSeed) -> Result<(), DatabaseError> {
        self.database
            .run(move |client| {
                info_with_fields!("setup", step = "chains", chains = format!("{:?}", seed.chains));

                info_with_fields!("setup", step = "add chains");
                let _ = client.add_chains(seed.chains.clone());

                info_with_fields!("setup", step = "parser state");
                for chain in seed.chains.iter().copied() {
                    let _ = client.add_parser_state(chain, chain.block_time() as i32);
                }

                info_with_fields!("setup", step = "assets");
                let _ = client.add_assets(seed.assets);

                info_with_fields!("setup", step = "fiat providers");
                let _ = client.add_fiat_providers(seed.fiat_providers);

                info_with_fields!("setup", step = "api clients");
                let _ = client.add_api_client_grants(seed.api_client_grants);

                info_with_fields!("setup", step = "releases");
                let _ = client.add_releases(seed.releases);

                info_with_fields!("setup", step = "assets tags");
                let _ = client.add_tags(seed.tags);

                info_with_fields!("setup", step = "prices providers");
                let _ = client.add_prices_providers(seed.price_providers);

                let valid: HashSet<String> = seed.config_keys.iter().map(|key| key.as_ref().to_string()).chain(seed.config_params.iter().map(ConfigParamKey::key)).collect();

                info_with_fields!("setup", step = "config");
                let _ = client.add_config_keys(seed.config_keys);

                info_with_fields!("setup", step = "param config");
                let _ = client.add_config_params(seed.config_params);

                info_with_fields!("setup", step = "cleanup stale config keys");
                let stale: Vec<String> = client.get_config_keys()?.into_iter().filter(|key| !valid.contains(key)).collect();
                if !stale.is_empty() {
                    info_with_fields!("setup", step = "delete stale config keys", count = stale.len(), keys = format!("{:?}", stale));
                    let _ = client.delete_keys(stale);
                }

                Ok(())
            })
            .await
    }

    async fn add_missing_scan_addresses(&self, addresses: HashMap<(Chain, String), ScanAddress>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                let existing = client
                    .get_scan_addresses_by_addresses(addresses.keys().map(|(_, address)| address.clone()).collect())?
                    .into_iter()
                    .map(|scan_address| (scan_address.chain, scan_address.address))
                    .collect::<HashSet<_>>();
                let values = addresses.into_iter().filter_map(|(key, value)| (!existing.contains(&key)).then_some(value)).collect::<Vec<_>>();
                if values.is_empty() { Ok(0) } else { client.add_scan_addresses(values) }
            })
            .await
    }

    async fn set_dev_seed(&self) -> Result<i32, Box<dyn Error + Send + Sync>> {
        self.database
            .run(|client| -> Result<_, Box<dyn Error + Send + Sync>> {
                info_with_fields!("setup_dev", step = "add currency");
                info_with_fields!("setup_dev", step = "add rate", currency = "USD");
                client.set_fiat_rates(FiatRateProvider::Coingecko, vec![FiatRate { symbol: Currency::USD, rate: 1.0 }])?;
                client.set_fiat_rates_enabled(vec![Currency::USD], true)?;

                info_with_fields!("setup_dev", step = "api clients");
                client.add_api_client_grants(api_client_access_grants(SETUP_DEV_API_CLIENT_NAME))?;
                client.set_api_client_secret(SETUP_DEV_API_CLIENT_NAME, SETUP_DEV_API_CLIENT_SECRET)?;

                info_with_fields!("setup_dev", step = "add devices");
                for device in dev_seed::devices() {
                    let device_id = device.id.clone();
                    client.add_device(device)?;
                    info_with_fields!("setup_dev", step = "device added", device_id = device_id);
                }
                let ios_device_row_id = client.get_device_row_id(&dev_seed::ios_device_id())?;
                let android_device_row_id = client.get_device_row_id(&dev_seed::android_device_id())?;

                info_with_fields!("setup_dev", step = "add wallet");
                let wallet = client.get_or_create_wallet(dev_seed::wallet())?;
                info_with_fields!("setup_dev", step = "wallet added", wallet_id = wallet.id);

                info_with_fields!("setup_dev", step = "add wallet subscriptions");
                let result = client.add_subscriptions(ios_device_row_id, dev_seed::subscriptions(wallet.id))?;
                info_with_fields!("setup_dev", step = "ios wallet subscription added", count = result);
                let result = client.add_subscriptions(android_device_row_id, dev_seed::subscriptions(wallet.id))?;
                info_with_fields!("setup_dev", step = "android wallet subscription added", count = result);

                info_with_fields!("setup_dev", step = "add fiat transactions");
                let count = dev_seed::fiat_transactions()
                    .into_iter()
                    .map(|(transaction, chain)| {
                        let address_id = client.get_subscriptions_wallet_address_for_chain(ios_device_row_id, wallet.id, chain)?.id;
                        client.add_fiat_transaction(transaction, ios_device_row_id, wallet.id, address_id)
                    })
                    .sum::<Result<usize, DatabaseError>>()?;
                info_with_fields!("setup_dev", step = "fiat transactions added", count = count);

                info_with_fields!("setup_dev", step = "add notifications");
                let result = client.add_notifications(dev_seed::notifications(wallet.id))?;
                info_with_fields!("setup_dev", step = "notifications added", count = result);

                info_with_fields!("setup_dev", step = "add price alerts");
                let result = client.add_price_alerts(&dev_seed::ios_device_id(), dev_seed::price_alerts())?;
                info_with_fields!("setup_dev", step = "price alerts added", count = result);

                info_with_fields!("setup_dev", step = "add assets");
                client.add_assets(dev_seed::assets())?;
                for (id, associations) in dev_seed::asset_associations() {
                    client.set_asset_associations(id, associations)?;
                }

                info_with_fields!("setup_dev", step = "add fiat assets");
                let result = client.add_fiat_assets(dev_seed::fiat_assets())?;
                info_with_fields!("setup_dev", step = "fiat assets added", count = result);

                info_with_fields!("setup_dev", step = "add fiat provider countries");
                let result = client.add_fiat_providers_countries(dev_seed::fiat_countries())?;
                info_with_fields!("setup_dev", step = "fiat provider countries added", count = result);

                info_with_fields!("setup_dev", step = "add prices and charts");
                let series = dev_seed::price_series(Utc::now().naive_utc());
                let result = client.add_prices(series.prices)?;
                info_with_fields!("setup_dev", step = "prices added", count = result);
                let result = client.set_prices_assets(series.price_assets)?;
                info_with_fields!("setup_dev", step = "prices_assets added", count = result);
                for (hourly, daily) in series.charts {
                    client.add_charts(ChartTimeframe::Hourly, hourly)?;
                    client.add_charts(ChartTimeframe::Daily, daily)?;
                }
                info_with_fields!("setup_dev", step = "charts added");

                Ok(wallet.id)
            })
            .await
    }
}
