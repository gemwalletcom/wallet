use std::error::Error;

use chrono::{DateTime, Utc};
use primitives::{AssetId, MAX_QUERY_LIMIT, Transaction, TransactionId, TransactionsResponse};
use storage::{Database, DatabaseError, DevicesRepository, ScanAddressesRepository, TransactionsRepository, WalletsRepository};

pub struct TransactionsClient {
    database: Database,
}

impl TransactionsClient {
    pub fn new(database: Database) -> Self {
        Self { database }
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
        let device_id = device_id.to_string();
        let from_datetime = from_timestamp.and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp as i64, 0).map(|datetime| datetime.naive_utc()));
        Ok(self
            .database
            .run(move |client| {
                let subscriptions = client.get_subscriptions_by_wallet_id(device_row_id, wallet_id)?;
                let addresses = subscriptions.iter().map(|subscription| subscription.address.clone()).collect::<Vec<_>>();
                let chains = subscriptions.iter().map(|subscription| subscription.chain.as_ref().to_string()).collect::<Vec<_>>();
                let transactions = client.get_transactions_by_device_id(&device_id, addresses.clone(), chains, asset_id, from_datetime, limit, offset)?;
                transactions_response(client, transactions, addresses)
            })
            .await?)
    }

    pub async fn get_transactions_by_device_id(&self, device_id: &str) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        Ok(self
            .database
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
            .await?)
    }

    pub async fn get_transaction_by_id(&self, id: &TransactionId) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let id = id.clone();
        Ok(self.database.run(move |client| client.get_transaction_by_id(&id, vec![])).await?)
    }

    pub async fn get_transaction_by_wallet_id(&self, device_row_id: i32, wallet_id: i32, id: &TransactionId) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let id = id.clone();
        let (addresses, transaction) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let addresses = client.get_subscriptions_by_wallet_id(device_row_id, wallet_id)?.into_iter().map(|subscription| subscription.address).collect::<Vec<_>>();
                let transaction = client.get_transaction_by_id(&id, addresses.clone())?;
                Ok((addresses, transaction))
            })
            .await?;
        Ok(transaction.finalize(addresses))
    }

    pub async fn get_transactions_by_hash(&self, hash: &str) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let hash = hash.to_string();
        Ok(self.database.run(move |client| client.get_transactions_by_hash(&hash)).await?)
    }
}

fn transactions_response(client: &mut impl ScanAddressesRepository, transactions: Vec<Transaction>, addresses: Vec<String>) -> Result<TransactionsResponse, DatabaseError> {
    let transactions = transactions.into_iter().map(|transaction| transaction.finalize(addresses.clone())).collect::<Vec<_>>();

    let address_names = client
        .get_scan_addresses_by_addresses(transactions.iter().flat_map(Transaction::addresses).collect())?
        .into_iter()
        .filter_map(|scan_address| scan_address.address_name())
        .collect();

    Ok(TransactionsResponse::new(transactions, address_names))
}
