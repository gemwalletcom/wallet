use std::error::Error;

use primitives::{AssetId, Transaction, TransactionId, TransactionsResponse};
use storage::models::TransactionRow;
use storage::{Database, DevicesRepository, ScanAddressesRepository, TransactionsRepository, WalletsRepository};

use chrono::{DateTime, Utc};

const DEVICE_TRANSACTIONS_LIMIT: i64 = 50;

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
    ) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.wallets()?.get_subscriptions_by_wallet_id(device_row_id, wallet_id)?;
        let addresses = subscriptions.iter().map(|(_, addr)| addr.address.clone()).collect::<Vec<_>>();
        let chains = subscriptions.iter().map(|(sub, _)| sub.chain.0.as_ref().to_string()).collect::<Vec<_>>();
        let from_datetime = from_timestamp.and_then(|ts| DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|dt| dt.naive_utc()));
        let rows = self
            .database
            .transactions()?
            .get_transactions_by_device_id(device_id, addresses.clone(), chains, asset_id, from_datetime, None)?;

        self.transactions_response(rows, addresses)
    }

    pub fn get_transactions_by_device_id(&self, device_id: &str) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let device_row_id = self.database.devices()?.get_device_row_id(device_id)?;
        let subscriptions = self.database.wallets()?.get_subscriptions(device_row_id)?;
        let addresses = subscriptions.iter().map(|(_, _, addr)| addr.address.clone()).collect::<Vec<_>>();
        let chains = subscriptions.iter().map(|(_, sub, _)| sub.chain.0.as_ref().to_string()).collect::<Vec<_>>();

        if addresses.is_empty() || chains.is_empty() {
            return Ok(TransactionsResponse::new(Vec::new(), Vec::new()));
        }

        let rows = self
            .database
            .transactions()?
            .get_transactions_by_device_id(device_id, addresses.clone(), chains, None, None, Some(DEVICE_TRANSACTIONS_LIMIT))?;

        self.transactions_response(rows, addresses)
    }

    fn transactions_response(&self, rows: Vec<TransactionRow>, addresses: Vec<String>) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let transactions = rows.into_iter().map(|x| x.as_primitive(addresses.clone()).finalize(addresses.clone())).collect::<Vec<_>>();

        let address_names = self
            .database
            .scan_addresses()?
            .get_scan_addresses_by_addresses(transactions.iter().flat_map(|x| x.addresses()).collect())?
            .into_iter()
            .filter_map(|x| x.as_primitive())
            .collect();

        Ok(TransactionsResponse::new(transactions, address_names))
    }

    pub fn get_transaction_by_id(&self, id: &TransactionId) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.database.transactions()?.get_transaction_by_id(id)?.as_primitive(vec![]))
    }
}
