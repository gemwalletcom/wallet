use std::sync::Arc;

use primitives::{Currency, TransactionExtended};

use super::model::GemTransactionDetailRows;
use super::rules;
use crate::services::explorer::GemExplorerService;
use crate::services::preferences::GemPreferencesService;

#[derive(uniffi::Object)]
pub struct GemTransactionDetailsService {
    explorer: Arc<GemExplorerService>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemTransactionDetailsService {
    #[uniffi::constructor]
    pub fn new(explorer: Arc<GemExplorerService>, preferences: Arc<GemPreferencesService>) -> Self {
        Self { explorer, preferences }
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn detail_rows(&self, transaction: TransactionExtended) -> GemTransactionDetailRows {
        let chain = transaction.transaction.asset_id.chain;
        let participant = rules::participant(&transaction, |address| self.explorer.get_address_url(chain, address.to_string()));
        let explorer = self.explorer.get_transaction_link(
            chain,
            transaction.transaction.hash().to_string(),
            transaction.transaction.swap_metadata().and_then(|metadata| metadata.provider),
            Some(transaction.transaction.to.clone()),
            transaction.transaction.memo.clone(),
        );
        rules::detail_rows(&transaction, participant, explorer)
    }
}
