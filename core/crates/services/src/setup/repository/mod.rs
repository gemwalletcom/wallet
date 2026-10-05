mod dev_seed;

use std::collections::{HashMap, HashSet};
use std::error::Error;

use async_trait::async_trait;
use config_keys::{ConfigKey, ConfigParamKey};
use gem_tracing::info_with_fields;
use primitives::{AssetBasic, AssetTag, Chain, FiatProviderName, PriceProvider, Release, ScanAddress};
use rewards::UsernameRules;
use storage::{
    ApiClientGrant, ApiClientsRepository, AssetsRepository, ChainsRepository, ConfigRepository, Database, DatabaseError, FiatRepository, MigrationsRepository, ParserStateRepository, PricesProvidersRepository, ReleasesRepository,
    ScanAddressesRepository, TagRepository,
};

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
    async fn run_migrations(&self) -> Result<(), DatabaseError>;
    async fn seed(&self, seed: SetupSeed) -> Result<(), DatabaseError>;
    async fn add_missing_scan_addresses(&self, addresses: HashMap<(Chain, String), ScanAddress>) -> Result<usize, DatabaseError>;
    async fn seed_dev(&self, username_rules: UsernameRules) -> Result<(), Box<dyn Error + Send + Sync>>;
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
    async fn run_migrations(&self) -> Result<(), DatabaseError> {
        self.database.run(MigrationsRepository::run_migrations).await
    }

    async fn seed(&self, seed: SetupSeed) -> Result<(), DatabaseError> {
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

    async fn seed_dev(&self, username_rules: UsernameRules) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.database
            .run(move |client| -> Result<_, Box<dyn Error + Send + Sync>> {
                dev_seed::setup_dev_currency(client)?;
                dev_seed::setup_dev_api_clients(client)?;
                dev_seed::setup_dev_devices(client, &username_rules)?;
                dev_seed::setup_dev_assets(client)
            })
            .await
    }
}
