use std::sync::Arc;

use primitives::{Chain, Currency, Transaction};

use super::model::GemTransactionParticipant;
use super::rules;
use crate::block_explorer::GemBlockExplorerLink;
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

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn participant(&self, transaction: Transaction) -> Option<GemTransactionParticipant> {
        let (role, address) = rules::transaction_participant(&transaction)?;
        let link = self.explorer.get_address_url(transaction.asset_id.chain, address.clone());
        Some(GemTransactionParticipant { role, address, link })
    }

    pub fn transaction_link(&self, chain: Chain, hash: String, provider: Option<String>, recipient: Option<String>, memo: Option<String>) -> GemBlockExplorerLink {
        self.explorer.get_transaction_link(chain, hash, provider, recipient, memo)
    }
}
