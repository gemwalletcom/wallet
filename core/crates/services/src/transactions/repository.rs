use std::collections::HashSet;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetAddress, AssetBasic, AssetId, AssetPriceMetadata, Chain, ChainAddress, Device, MAX_QUERY_LIMIT, NFTAssetId, PriceData, ScanAddress, ScanVerdict, Transaction, TransactionId, TransactionsResponse};
use storage::{
    AssetFilter, AssetsAddressesRepository, AssetsRepository, ChartResult, Database, DatabaseError, DevicesRepository, NftRepository, ParserState, ParserStateRepository, PricesRepository, ScanAddressesRepository, ScanDetectionsRepository,
    TransactionFilter, TransactionPerpetualRecord, TransactionSwapRecord, TransactionUpdate, TransactionsPerpetualsRepository, TransactionsRepository, TransactionsSwapsRepository, WalletRecord, WalletsRepository,
};

pub(crate) struct WalletTransactionsQuery {
    pub(crate) device_id: String,
    pub(crate) device_row_id: i32,
    pub(crate) wallet_id: i32,
    pub(crate) asset_id: Option<AssetId>,
    pub(crate) since: Option<NaiveDateTime>,
    pub(crate) limit: usize,
    pub(crate) offset: usize,
}

pub(crate) struct AssetPriceHistory {
    pub(crate) assets: Vec<AssetBasic>,
    pub(crate) price_at: Option<ChartResult>,
    pub(crate) prices: Vec<PriceData>,
}

pub(crate) struct AddressRecords {
    pub(crate) assets: Vec<Asset>,
    pub(crate) scan_addresses: Vec<ScanAddress>,
    pub(crate) verdicts: Vec<ScanVerdict>,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn wallet_transactions(&self, query: WalletTransactionsQuery) -> Result<TransactionsResponse, DatabaseError>;
    async fn device_transactions(&self, device_id: String) -> Result<TransactionsResponse, DatabaseError>;
    async fn transaction(&self, id: TransactionId) -> Result<Transaction, DatabaseError>;
    async fn wallet_transaction(&self, device_row_id: i32, wallet_id: i32, id: TransactionId) -> Result<(Vec<String>, Transaction), DatabaseError>;
    async fn transactions_by_hash(&self, hash: String) -> Result<Vec<Transaction>, DatabaseError>;
    async fn address_records(&self, address: ChainAddress, detection_max_age: Duration) -> Result<AddressRecords, DatabaseError>;
    async fn address_name_records(&self, addresses: Vec<ChainAddress>, asset_ids: Vec<AssetId>) -> Result<(Vec<ScanAddress>, Vec<Asset>), DatabaseError>;
    async fn parser_state(&self, chain: Chain) -> Result<ParserState, DatabaseError>;
    async fn parser_states(&self) -> Result<Vec<ParserState>, DatabaseError>;
    async fn set_parser_current_block(&self, chain: Chain, block: i64) -> Result<usize, DatabaseError>;
    async fn set_parser_latest_block(&self, chain: Chain, block: i64) -> Result<usize, DatabaseError>;
    async fn wallet_with_devices(&self, wallet_row_id: i32) -> Result<(WalletRecord, Vec<Device>), DatabaseError>;
    async fn asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError>;
    async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError>;
    async fn assets_with_prices(&self, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError>;
    async fn asset_price_history(&self, asset_id: AssetId, at: NaiveDateTime) -> Result<AssetPriceHistory, DatabaseError>;
    async fn nft_asset_ids(&self, identifiers: Vec<String>) -> Result<Vec<NFTAssetId>, DatabaseError>;
    async fn upsert_transactions(&self, transactions: Vec<Transaction>, batch_size: usize) -> Result<HashSet<TransactionId>, DatabaseError>;
    async fn add_asset_addresses(&self, addresses: Vec<AssetAddress>) -> Result<usize, DatabaseError>;
    async fn transactions(&self, filters: Vec<TransactionFilter>, limit: i64) -> Result<Vec<Transaction>, DatabaseError>;
    async fn update_transaction(&self, chain: Chain, hash: String, updates: Vec<TransactionUpdate>) -> Result<usize, DatabaseError>;
    async fn transaction_exists(&self, id: TransactionId) -> Result<bool, DatabaseError>;
    async fn upsert_transaction_swap(&self, id: TransactionId, record: TransactionSwapRecord) -> Result<usize, DatabaseError>;
    async fn upsert_transaction_perpetual(&self, id: TransactionId, record: TransactionPerpetualRecord) -> Result<usize, DatabaseError>;
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
    async fn wallet_transactions(&self, query: WalletTransactionsQuery) -> Result<TransactionsResponse, DatabaseError> {
        self.database
            .run(move |client| {
                let subscriptions = client.get_subscriptions_by_wallet_id(query.device_row_id, query.wallet_id)?;
                let addresses = subscriptions.iter().map(|subscription| subscription.address.clone()).collect::<Vec<_>>();
                let chains = subscriptions.iter().map(|subscription| subscription.chain.as_ref().to_string()).collect::<Vec<_>>();
                let transactions = client.get_transactions_by_device_id(&query.device_id, addresses.clone(), chains, query.asset_id, query.since, query.limit, query.offset)?;
                transactions_response(client, transactions, addresses)
            })
            .await
    }

    async fn device_transactions(&self, device_id: String) -> Result<TransactionsResponse, DatabaseError> {
        self.database
            .run(move |client| {
                let device_row_id = client.get_device_row_id(&device_id)?;
                let subscriptions = client.get_subscriptions(device_row_id)?;
                let addresses = subscriptions.iter().map(|(_, subscription)| subscription.address.clone()).collect::<Vec<_>>();
                let chains = subscriptions.iter().map(|(_, subscription)| subscription.chain.as_ref().to_string()).collect::<Vec<_>>();
                if addresses.is_empty() || chains.is_empty() {
                    return Ok(TransactionsResponse::new(Vec::new(), Vec::new()));
                }
                let transactions = client.get_transactions_by_device_id(&device_id, addresses.clone(), chains, None, None, MAX_QUERY_LIMIT, 0)?;
                transactions_response(client, transactions, addresses)
            })
            .await
    }

    async fn transaction(&self, id: TransactionId) -> Result<Transaction, DatabaseError> {
        self.database.run(move |client| client.get_transaction_by_id(&id, vec![])).await
    }

    async fn wallet_transaction(&self, device_row_id: i32, wallet_id: i32, id: TransactionId) -> Result<(Vec<String>, Transaction), DatabaseError> {
        self.database
            .run(move |client| {
                let addresses = client.get_subscriptions_by_wallet_id(device_row_id, wallet_id)?.into_iter().map(|subscription| subscription.address).collect::<Vec<_>>();
                let transaction = client.get_transaction_by_id(&id, addresses.clone())?;
                Ok((addresses, transaction))
            })
            .await
    }

    async fn transactions_by_hash(&self, hash: String) -> Result<Vec<Transaction>, DatabaseError> {
        self.database.run(move |client| client.get_transactions_by_hash(&hash)).await
    }

    async fn address_records(&self, address: ChainAddress, detection_max_age: Duration) -> Result<AddressRecords, DatabaseError> {
        self.database
            .run(move |client| {
                Ok(AddressRecords {
                    assets: client.get_assets(vec![AssetId::from(address.chain, Some(address.address.clone()))])?,
                    scan_addresses: client.get_scan_addresses(&[(address.chain, address.address.as_str())])?,
                    verdicts: client.get_scan_detections(vec![address.address.clone()], detection_max_age)?,
                })
            })
            .await
    }

    async fn address_name_records(&self, addresses: Vec<ChainAddress>, asset_ids: Vec<AssetId>) -> Result<(Vec<ScanAddress>, Vec<Asset>), DatabaseError> {
        self.database
            .run(move |client| {
                let queries = addresses.iter().map(|request| (request.chain, request.address.as_str())).collect::<Vec<_>>();
                Ok((client.get_scan_addresses(&queries)?, client.get_assets(asset_ids)?))
            })
            .await
    }

    async fn parser_state(&self, chain: Chain) -> Result<ParserState, DatabaseError> {
        self.database.run(move |client| client.get_parser_state(chain)).await
    }

    async fn parser_states(&self) -> Result<Vec<ParserState>, DatabaseError> {
        self.database.run(ParserStateRepository::get_parser_states).await
    }

    async fn set_parser_current_block(&self, chain: Chain, block: i64) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_parser_state_current_block(chain, block)).await
    }

    async fn set_parser_latest_block(&self, chain: Chain, block: i64) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_parser_state_latest_block(chain, block)).await
    }

    async fn wallet_with_devices(&self, wallet_row_id: i32) -> Result<(WalletRecord, Vec<Device>), DatabaseError> {
        self.database.run(move |client| Ok((client.get_wallet_by_id(wallet_row_id)?, client.get_devices_by_wallet_id(wallet_row_id)?))).await
    }

    async fn asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        self.database.run(move |client| client.get_asset(&asset_id)).await
    }

    async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError> {
        self.database.run(move |client| client.get_assets(asset_ids)).await
    }

    async fn assets_with_prices(&self, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        self.database.run(move |client| client.get_assets_with_prices(filters, price_max_age)).await
    }

    async fn asset_price_history(&self, asset_id: AssetId, at: NaiveDateTime) -> Result<AssetPriceHistory, DatabaseError> {
        self.database
            .run(move |client| {
                Ok(AssetPriceHistory {
                    assets: client.get_assets_basic(vec![asset_id.clone()])?,
                    price_at: client.get_price_at(&asset_id, at)?,
                    prices: client.get_prices_for_asset(&asset_id)?,
                })
            })
            .await
    }

    async fn nft_asset_ids(&self, identifiers: Vec<String>) -> Result<Vec<NFTAssetId>, DatabaseError> {
        self.database.run(move |client| client.get_nft_asset_ids(identifiers)).await
    }

    async fn upsert_transactions(&self, transactions: Vec<Transaction>, batch_size: usize) -> Result<HashSet<TransactionId>, DatabaseError> {
        self.database
            .run(move |client| {
                transactions.chunks(batch_size).try_fold(HashSet::new(), |inserted_ids, chunk| {
                    let chunk_inserted_ids = client.upsert_transactions(chunk.to_vec())?;
                    Ok(inserted_ids.into_iter().chain(chunk_inserted_ids).collect())
                })
            })
            .await
    }

    async fn add_asset_addresses(&self, addresses: Vec<AssetAddress>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_assets_addresses(addresses)).await
    }

    async fn transactions(&self, filters: Vec<TransactionFilter>, limit: i64) -> Result<Vec<Transaction>, DatabaseError> {
        self.database.run(move |client| client.get_transactions_by_filter(filters, limit)).await
    }

    async fn update_transaction(&self, chain: Chain, hash: String, updates: Vec<TransactionUpdate>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_transaction(chain.as_ref(), &hash, updates)).await
    }

    async fn transaction_exists(&self, id: TransactionId) -> Result<bool, DatabaseError> {
        self.database.run(move |client| client.get_transaction_exists(&id)).await
    }

    async fn upsert_transaction_swap(&self, id: TransactionId, record: TransactionSwapRecord) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.upsert_transaction_swap(&id, record)).await
    }

    async fn upsert_transaction_perpetual(&self, id: TransactionId, record: TransactionPerpetualRecord) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.upsert_transaction_perpetual(&id, record)).await
    }
}

fn transactions_response(client: &mut impl ScanAddressesRepository, transactions: Vec<Transaction>, addresses: Vec<String>) -> Result<TransactionsResponse, DatabaseError> {
    let transactions = transactions.into_iter().map(|transaction| transaction.finalize(addresses.clone()).without_utxo()).collect::<Vec<_>>();
    let address_names = client
        .get_scan_addresses_by_addresses(transactions.iter().flat_map(Transaction::addresses).collect())?
        .into_iter()
        .filter_map(|scan_address| scan_address.address_name())
        .collect();
    Ok(TransactionsResponse::new(transactions, address_names))
}
