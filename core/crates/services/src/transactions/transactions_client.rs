use std::error::Error;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use primitives::{AssetId, Transaction, TransactionId, TransactionsResponse};

use super::repository::{Repository, WalletTransactionsQuery};

pub struct TransactionsClient {
    repository: Arc<dyn Repository>,
}

impl TransactionsClient {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn get_transactions_by_wallet_id(
        &self,
        device_id: &str,
        device_row_id: i32,
        wallet_id: i32,
        asset_id: Option<AssetId>,
        from_timestamp: Option<u64>,
        limit: usize,
        offset: usize,
    ) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let since = from_timestamp.and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp as i64, 0).map(|datetime| datetime.naive_utc()));
        let query = WalletTransactionsQuery {
            device_id: device_id.to_string(),
            device_row_id,
            wallet_id,
            asset_id,
            since,
            limit,
            offset,
        };
        Ok(self.repository.wallet_transactions(query).await?)
    }

    pub async fn get_transactions_by_device_id(&self, device_id: &str) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.device_transactions(device_id.to_string()).await?)
    }

    pub async fn get_transaction_by_id(&self, id: &TransactionId) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.transaction(id.clone()).await?)
    }

    pub async fn get_transaction_by_wallet_id(&self, device_row_id: i32, wallet_id: i32, id: &TransactionId) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let (addresses, transaction) = self.repository.wallet_transaction(device_row_id, wallet_id, id.clone()).await?;
        Ok(transaction.finalize(addresses).without_utxo())
    }

    pub async fn get_transactions_by_hash(&self, hash: &str) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.transactions_by_hash(hash.to_string()).await?)
    }
}
