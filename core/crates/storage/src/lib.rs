use tokio::task::spawn_blocking;

pub mod database;
pub mod error;
pub(crate) mod models;
pub mod repositories;
pub(crate) mod schema;
pub(crate) mod sql_types;
#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

diesel::allow_columns_to_appear_in_same_group_by_clause!(schema::transactions_addresses::address, schema::transactions::chain,);

pub use self::database::DatabaseClient;
pub use self::error::{DatabaseError, DieselResultExt};
pub use self::models::{ApiClientGrant, ApiClientResource, ApiClientScope};
pub use self::repositories::{
    api_clients_repository::ApiClientsRepository,
    assets_addresses_repository::AssetsAddressesRepository,
    assets_links_repository::AssetsLinksRepository,
    assets_repository::{AssetFilter, AssetSupply, AssetUpdate, AssetsRepository},
    assets_usage_ranks_repository::AssetsUsageRanksRepository,
    chains_repository::ChainsRepository,
    charts_repository::{ChartFilter, ChartPoint, ChartsRepository},
    config_repository::ConfigRepository,
    devices_repository::{DeviceFieldUpdate, DeviceRecord, DevicesRepository},
    fiat_repository::{FiatAssetFilter, FiatRepository, FiatTransactionRecord},
    migrations_repository::MigrationsRepository,
    nft_repository::{NftCollectionFilter, NftRepository},
    notifications_repository::{NewNotification, NotificationsRepository},
    parser_state_repository::{ParserState, ParserStateRepository},
    perpetuals_repository::PerpetualsRepository,
    price_alerts_repository::PriceAlertsRepository,
    prices_providers_repository::{PriceProviderConfig, PricesProvidersRepository},
    prices_repository::{AssetWithMarket, AssetsWithPricesFilter, PriceAsset, PriceFilter, PriceUpdate, PricesRepository},
    releases_repository::ReleasesRepository,
    rewards_redemptions_repository::{RedemptionRecord, RedemptionUpdate, RewardsRedemptionsRepository},
    rewards_repository::{ReferralRecord, ReferrerInfo, RewardIdentityRecord, RewardsEligibilityConfig, RewardsFilter, RewardsRecord, RewardsRepository, RewardsVerification},
    risk_signals_repository::{AbusePatterns, RiskSignalsRepository},
    scan_addresses_repository::ScanAddressesRepository,
    scan_detections_repository::ScanDetectionsRepository,
    support_sessions_repository::SupportSessionsRepository,
    tag_repository::{AssetTagLink, PerpetualTagLink, Tag, TagRepository},
    transactions_repository::{TransactionFilter, TransactionUpdate, TransactionsRepository},
    wallets_repository::{NewWallet, WalletAddress, WalletRecord, WalletsRepository},
};

#[derive(Clone)]
pub struct Database(database::PgPool);

impl Database {
    pub fn new(database_url: &str, pool_size: u32) -> Result<Self, DatabaseError> {
        Ok(Self(database::create_pool(database_url, pool_size)?))
    }

    pub async fn run<T, E, F>(&self, operation: F) -> Result<T, E>
    where
        T: Send + 'static,
        E: From<DatabaseError> + Send + 'static,
        F: FnOnce(&mut DatabaseClient) -> Result<T, E> + Send + 'static,
    {
        let pool = self.0.clone();
        match spawn_blocking(move || operation(&mut DatabaseClient::from_pool(&pool)?)).await {
            Ok(result) => result,
            Err(error) => Err(DatabaseError::from(error).into()),
        }
    }

    pub async fn transaction<T, E, F>(&self, operation: F) -> Result<T, E>
    where
        T: Send + 'static,
        E: From<DatabaseError> + Send + 'static,
        F: FnOnce(&mut DatabaseClient) -> Result<T, E> + Send + 'static,
    {
        self.run(move |client| client.transaction(operation)).await
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use primitives::Chain;

    use crate::{ChainsRepository, Database, DatabaseError, ParserStateRepository};

    #[tokio::test]
    async fn test_transaction() {
        let database = Database::mock();
        database
            .run(|client| -> Result<_, DatabaseError> {
                client.add_chains(vec![Chain::Ethereum])?;
                client.add_parser_state(Chain::Ethereum, 12_000)
            })
            .await
            .unwrap();
        let initial = database.run(|client| client.get_parser_state(Chain::Ethereum)).await.unwrap().current_block;

        let rolled_back: Result<(), DatabaseError> = database
            .transaction(move |client| {
                client.set_parser_state_current_block(Chain::Ethereum, initial + 100)?;
                Err(DatabaseError::Error("rollback".to_string()))
            })
            .await;

        assert!(rolled_back.is_err());
        assert_eq!(database.run(|client| client.get_parser_state(Chain::Ethereum)).await.unwrap().current_block, initial);

        database.transaction(move |client| client.set_parser_state_current_block(Chain::Ethereum, initial + 100)).await.unwrap();

        assert_eq!(database.run(|client| client.get_parser_state(Chain::Ethereum)).await.unwrap().current_block, initial + 100);
    }
}
