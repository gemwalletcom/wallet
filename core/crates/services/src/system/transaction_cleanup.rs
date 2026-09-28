use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;

use chrono::Utc;
use storage::{Database, DatabaseError, TransactionsRepository, WalletsRepository};

#[derive(Clone)]
pub struct TransactionCleanupConfig {
    pub address_max_count: i64,
    pub address_limit: usize,
    pub lookback: Duration,
}

#[derive(Clone)]
pub struct TransactionCleanup {
    database: Database,
    config: TransactionCleanupConfig,
}

impl TransactionCleanup {
    pub fn new(database: Database, config: TransactionCleanupConfig) -> Self {
        Self { database, config }
    }

    pub async fn cleanup(&self) -> Result<HashMap<String, usize>, Box<dyn Error + Send + Sync>> {
        let since = (Utc::now() - self.config.lookback).naive_utc();
        let address_max_count = self.config.address_max_count;
        let address_limit = self.config.address_limit as i64;

        Ok(self
            .database
            .run(move |client| -> Result<HashMap<String, usize>, DatabaseError> {
                let heavy_addresses = client.get_transactions_addresses(address_max_count, address_limit, since)?;

                if heavy_addresses.is_empty() {
                    return Ok(HashMap::new());
                }

                client.add_subscriptions_exclude_addresses(heavy_addresses.clone())?;

                let total_addresses = heavy_addresses.len();

                let affected_transaction_ids = client.delete_transactions_addresses(heavy_addresses)?;
                let total_transactions_addresses = affected_transaction_ids.len();

                let unique_ids: Vec<i64> = affected_transaction_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();
                let total_deleted_transactions = client.delete_orphaned_transactions(unique_ids)?;

                Ok(HashMap::from([
                    ("addresses".to_string(), total_addresses),
                    ("transactions_addresses".to_string(), total_transactions_addresses),
                    ("transactions_deleted".to_string(), total_deleted_transactions),
                ]))
            })
            .await?)
    }
}
