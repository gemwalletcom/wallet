use std::sync::Arc;

use primitives::{Chain, Currency, Transaction, TransactionExtended};

use super::model::{GemTransactionDetails, GemTransactionHeaderKind, GemTransactionParticipant};
use super::rules;
use crate::services::explorer::GemExplorerService;
use crate::services::preferences::GemPreferencesService;
use primitives::BlockExplorerLink;

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

    pub fn details(&self, transaction: TransactionExtended) -> GemTransactionDetails {
        rules::details(&transaction)
    }

    pub fn header_kind(&self, transaction: Transaction) -> GemTransactionHeaderKind {
        rules::header_kind(&transaction)
    }

    pub fn participant(&self, transaction: Transaction) -> Option<GemTransactionParticipant> {
        let (role, address) = rules::transaction_participant(&transaction)?;
        let link = self.explorer.get_address_url(transaction.asset_id.chain, address.clone());
        Some(GemTransactionParticipant { role, address, link })
    }

    pub fn transaction_link(&self, chain: Chain, hash: String, provider: Option<String>, recipient: Option<String>, memo: Option<String>) -> BlockExplorerLink {
        self.explorer.get_transaction_link(chain, hash, provider, recipient, memo)
    }
}
