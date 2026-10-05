use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use chrono::Utc;
use std::sync::Arc;

use super::repository::{Repository, TransactionCleanupResult};

#[derive(Clone)]
pub struct TransactionCleanupConfig {
    pub address_max_count: i64,
    pub address_limit: usize,
    pub lookback: Duration,
}

#[derive(Clone)]
pub struct TransactionCleanup {
    repository: Arc<dyn Repository>,
    config: TransactionCleanupConfig,
}

impl TransactionCleanup {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: TransactionCleanupConfig) -> Self {
        Self { repository, config }
    }

    pub async fn cleanup(&self) -> Result<HashMap<String, usize>, Box<dyn Error + Send + Sync>> {
        let since = (Utc::now() - self.config.lookback).naive_utc();
        let address_max_count = self.config.address_max_count;
        let address_limit = self.config.address_limit as i64;

        let Some(TransactionCleanupResult {
            addresses,
            transactions_addresses,
            transactions_deleted,
        }) = self.repository.cleanup_heavy_addresses(address_max_count, address_limit, since).await?
        else {
            return Ok(HashMap::new());
        };
        Ok(HashMap::from([
            ("addresses".to_string(), addresses),
            ("transactions_addresses".to_string(), transactions_addresses),
            ("transactions_deleted".to_string(), transactions_deleted),
        ]))
    }
}
