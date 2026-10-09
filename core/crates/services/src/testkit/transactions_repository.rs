use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetAddress, AssetId, AssetPriceMetadata, Chain, ChainAddress, Device, NFTAssetId, ScanAddress, Transaction, TransactionId, TransactionsResponse};
use storage::{AssetFilter, DatabaseError, ParserState, TransactionFilter, TransactionPerpetualRecord, TransactionSwapRecord, TransactionUpdate, WalletRecord};

use crate::transactions::repository::{AddressRecords, AssetPriceHistory, Repository, WalletTransactionsQuery};

#[derive(Default)]
pub(crate) struct MemoryTransactionsRepository {
    transactions: Vec<Transaction>,
    assets: Vec<Asset>,
    perpetual_records: Mutex<Vec<(TransactionId, TransactionPerpetualRecord)>>,
}

impl MemoryTransactionsRepository {
    pub(crate) fn new(transactions: Vec<Transaction>, assets: Vec<Asset>) -> Self {
        Self { transactions, assets, ..Self::default() }
    }

    pub(crate) fn perpetual_records(&self) -> Vec<(TransactionId, TransactionPerpetualRecord)> {
        self.perpetual_records.lock().unwrap().clone()
    }

    fn not_found(resource: &'static str, lookup: impl ToString) -> DatabaseError {
        DatabaseError::not_found(resource, lookup.to_string())
    }
}

#[async_trait]
impl Repository for MemoryTransactionsRepository {
    async fn get_wallet_transactions(&self, _query: WalletTransactionsQuery) -> Result<TransactionsResponse, DatabaseError> {
        Ok(TransactionsResponse::new(self.transactions.clone(), vec![]))
    }

    async fn get_device_transactions(&self, _device_id: String) -> Result<TransactionsResponse, DatabaseError> {
        Ok(TransactionsResponse::new(self.transactions.clone(), vec![]))
    }

    async fn get_transaction(&self, id: TransactionId) -> Result<Transaction, DatabaseError> {
        self.transactions.iter().find(|transaction| transaction.id == id).cloned().ok_or_else(|| Self::not_found("Transaction", id))
    }

    async fn get_wallet_transaction(&self, _device_row_id: i32, _wallet_id: i32, id: TransactionId) -> Result<(Vec<String>, Transaction), DatabaseError> {
        Ok((vec![], self.get_transaction(id).await?))
    }

    async fn get_transactions_by_hash(&self, hash: String) -> Result<Vec<Transaction>, DatabaseError> {
        Ok(self.transactions.iter().filter(|transaction| transaction.id.hash == hash).cloned().collect())
    }

    async fn get_address_records(&self, _address: ChainAddress, _detection_max_age: Duration) -> Result<AddressRecords, DatabaseError> {
        Ok(AddressRecords {
            assets: vec![],
            scan_addresses: vec![],
            verdicts: vec![],
        })
    }

    async fn get_address_name_records(&self, _addresses: Vec<ChainAddress>, _asset_ids: Vec<AssetId>) -> Result<(Vec<ScanAddress>, Vec<Asset>), DatabaseError> {
        Ok((vec![], vec![]))
    }

    async fn get_parser_state(&self, chain: Chain) -> Result<ParserState, DatabaseError> {
        Err(Self::not_found("ParserState", chain))
    }

    async fn get_parser_states(&self) -> Result<Vec<ParserState>, DatabaseError> {
        Ok(vec![])
    }

    async fn set_parser_current_block(&self, _chain: Chain, _block: i64) -> Result<usize, DatabaseError> {
        Ok(1)
    }

    async fn set_parser_latest_block(&self, _chain: Chain, _block: i64) -> Result<usize, DatabaseError> {
        Ok(1)
    }

    async fn get_wallet_with_devices(&self, wallet_row_id: i32) -> Result<(WalletRecord, Vec<Device>), DatabaseError> {
        Err(Self::not_found("Wallet", wallet_row_id))
    }

    async fn get_asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        self.assets.iter().find(|asset| asset.id == asset_id).cloned().ok_or_else(|| Self::not_found("Asset", asset_id))
    }

    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError> {
        Ok(self.assets.iter().filter(|asset| asset_ids.contains(&asset.id)).cloned().collect())
    }

    async fn get_assets_with_prices(&self, _filters: Vec<AssetFilter>, _price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        Ok(vec![])
    }

    async fn get_asset_price_history(&self, asset_id: AssetId, _at: NaiveDateTime) -> Result<AssetPriceHistory, DatabaseError> {
        Err(Self::not_found("Asset", asset_id))
    }

    async fn get_nft_asset_ids(&self, _identifiers: Vec<String>) -> Result<Vec<NFTAssetId>, DatabaseError> {
        Ok(vec![])
    }

    async fn set_transactions(&self, transactions: Vec<Transaction>, _batch_size: usize) -> Result<HashSet<TransactionId>, DatabaseError> {
        Ok(transactions.into_iter().map(|transaction| transaction.id).collect())
    }

    async fn add_asset_addresses(&self, addresses: Vec<AssetAddress>) -> Result<usize, DatabaseError> {
        Ok(addresses.len())
    }

    async fn get_transactions(&self, _filters: Vec<TransactionFilter>, _limit: i64) -> Result<Vec<Transaction>, DatabaseError> {
        Ok(self.transactions.clone())
    }

    async fn update_transactions(&self, _filters: Vec<TransactionFilter>, _updates: Vec<TransactionUpdate>) -> Result<usize, DatabaseError> {
        Ok(1)
    }

    async fn get_transaction_exists(&self, id: TransactionId) -> Result<bool, DatabaseError> {
        Ok(self.transactions.iter().any(|transaction| transaction.id == id))
    }

    async fn set_transaction_swap(&self, _id: TransactionId, _record: TransactionSwapRecord) -> Result<usize, DatabaseError> {
        Ok(1)
    }

    async fn set_transaction_perpetual(&self, id: TransactionId, record: TransactionPerpetualRecord) -> Result<usize, DatabaseError> {
        self.perpetual_records.lock().unwrap().push((id, record));
        Ok(1)
    }
}
