use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetId, Chain, Device, FiatAsset, FiatProvider, FiatProviderCountry, FiatProviderName, FiatTransaction, FiatTransactionUpdate, WalletId};
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database, DatabaseError, DevicesRepository, FiatAssetFilter, FiatRepository, FiatTransactionRecord, TagRepository, WalletAddress, WalletsRepository};

pub(crate) struct QuoteContext {
    pub(crate) countries: Vec<FiatProviderCountry>,
    pub(crate) fiat_assets: Vec<FiatAsset>,
    pub(crate) providers: Vec<FiatProvider>,
}

pub(crate) struct NotificationContext {
    pub(crate) asset: Asset,
    pub(crate) wallet_id: WalletId,
    pub(crate) devices: Vec<Device>,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn get_wallet_fiat_transactions(&self, device_row_id: i32, wallet_id: i32) -> Result<Vec<FiatTransaction>, DatabaseError>;
    async fn get_device_fiat_transactions(&self, device_id: String) -> Result<Vec<FiatTransaction>, DatabaseError>;
    async fn get_asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError>;
    async fn get_quote_context(&self, asset_id: AssetId) -> Result<QuoteContext, DatabaseError>;
    async fn add_fiat_transaction(&self, transaction: FiatTransaction, device_id: i32, wallet_id: i32, address_id: i32) -> Result<usize, DatabaseError>;
    async fn get_subscription_address(&self, device_id: i32, wallet_id: i32, chain: Chain) -> Result<WalletAddress, DatabaseError>;
    async fn get_asset_ids(&self, filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError>;
    async fn get_fiat_asset_ids(&self, filters: Vec<FiatAssetFilter>) -> Result<Vec<AssetId>, DatabaseError>;
    async fn update_assets(&self, filters: Vec<AssetFilter>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError>;
    async fn set_popular_assets_tag(&self, tag: String, since: NaiveDateTime, limit: i64) -> Result<usize, DatabaseError>;
    async fn update_payment_methods(&self, provider: FiatProviderName, payment_methods: serde_json::Value) -> Result<usize, DatabaseError>;
    async fn set_fiat_assets(&self, provider: FiatProviderName, assets: Vec<FiatAsset>) -> Result<usize, DatabaseError>;
    async fn set_countries(&self, provider: FiatProviderName, countries: Vec<FiatProviderCountry>) -> Result<usize, DatabaseError>;
    async fn update_fiat_transaction(&self, provider: FiatProviderName, update: FiatTransactionUpdate) -> Result<(Option<FiatTransactionRecord>, FiatTransactionRecord), DatabaseError>;
    async fn get_notification_context(&self, asset_id: AssetId, wallet_row_id: i32) -> Result<NotificationContext, DatabaseError>;
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
    async fn get_wallet_fiat_transactions(&self, device_row_id: i32, wallet_id: i32) -> Result<Vec<FiatTransaction>, DatabaseError> {
        self.database.run(move |client| client.get_fiat_transactions_by_device_and_wallet_id(device_row_id, wallet_id)).await
    }

    async fn get_device_fiat_transactions(&self, device_id: String) -> Result<Vec<FiatTransaction>, DatabaseError> {
        self.database
            .run(move |client| {
                let device_row_id = client.get_device_row_id(&device_id)?;
                client.get_fiat_transactions_by_device_id(device_row_id)
            })
            .await
    }

    async fn get_asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        self.database.run(move |client| client.get_asset(&asset_id)).await
    }

    async fn get_quote_context(&self, asset_id: AssetId) -> Result<QuoteContext, DatabaseError> {
        self.database
            .run(move |client| {
                Ok(QuoteContext {
                    countries: client.get_fiat_providers_countries()?,
                    fiat_assets: client.get_fiat_assets_for_asset_id(&asset_id)?,
                    providers: client.get_fiat_providers()?,
                })
            })
            .await
    }

    async fn add_fiat_transaction(&self, transaction: FiatTransaction, device_id: i32, wallet_id: i32, address_id: i32) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_fiat_transaction(transaction, device_id, wallet_id, address_id)).await
    }

    async fn get_subscription_address(&self, device_id: i32, wallet_id: i32, chain: Chain) -> Result<WalletAddress, DatabaseError> {
        self.database.run(move |client| client.get_subscriptions_wallet_address_for_chain(device_id, wallet_id, chain)).await
    }

    async fn get_asset_ids(&self, filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        self.database.run(move |client| client.get_asset_ids_by_filter(filters)).await
    }

    async fn get_fiat_asset_ids(&self, filters: Vec<FiatAssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        self.database.run(move |client| client.get_fiat_asset_ids_by_filter(filters)).await
    }

    async fn update_assets(&self, filters: Vec<AssetFilter>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_assets(filters, updates)).await
    }

    async fn set_popular_assets_tag(&self, tag: String, since: NaiveDateTime, limit: i64) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                let asset_ids = client.get_fiat_assets_popular(since, limit)?;
                client.set_assets_tags_for_tag(&tag, asset_ids)
            })
            .await
    }

    async fn update_payment_methods(&self, provider: FiatProviderName, payment_methods: serde_json::Value) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_fiat_provider_payment_methods(provider, payment_methods)).await
    }

    async fn set_fiat_assets(&self, provider: FiatProviderName, assets: Vec<FiatAsset>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_fiat_assets(provider, assets)).await
    }

    async fn set_countries(&self, provider: FiatProviderName, countries: Vec<FiatProviderCountry>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_fiat_providers_countries(provider, countries)).await
    }

    async fn update_fiat_transaction(&self, provider: FiatProviderName, update: FiatTransactionUpdate) -> Result<(Option<FiatTransactionRecord>, FiatTransactionRecord), DatabaseError> {
        self.database
            .run(move |client| {
                let existing = client.get_fiat_transaction(provider, &update.transaction_id)?;
                let updated = client.update_fiat_transaction(provider, update)?;
                Ok((existing, updated))
            })
            .await
    }

    async fn get_notification_context(&self, asset_id: AssetId, wallet_row_id: i32) -> Result<NotificationContext, DatabaseError> {
        self.database
            .run(move |client| {
                Ok(NotificationContext {
                    asset: client.get_asset(&asset_id)?,
                    wallet_id: client.get_wallet_by_id(wallet_row_id)?.wallet_id,
                    devices: client.get_devices_by_wallet_id(wallet_row_id)?,
                })
            })
            .await
    }
}
