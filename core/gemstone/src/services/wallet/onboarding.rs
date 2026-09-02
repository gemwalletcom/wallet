use std::sync::Arc;

use primitives::name::NameRecord;
use primitives::{Chain, Wallet, WalletId, WalletSource};

use crate::services::error::GemServiceError;
use crate::services::name::GemNameService;
use crate::services::recipient::{GemRecipientError, GemRecipientValidation};
use crate::services::transfer::model::GemRecipient;
use crate::services::wallet::{GemWalletImportResult, GemWalletImportType, GemWalletService};
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemOnboardingService {
    wallets: Arc<GemWalletService>,
    session: Arc<GemWalletSessionService>,
    names: Arc<GemNameService>,
}

#[uniffi::export]
impl GemOnboardingService {
    #[uniffi::constructor]
    pub fn new(wallets: Arc<GemWalletService>, session: Arc<GemWalletSessionService>, names: Arc<GemNameService>) -> Self {
        Self { wallets, session, names }
    }

    pub fn create_wallet(&self) -> Result<Vec<String>, GemServiceError> {
        self.wallets.create_wallet()
    }

    pub fn next_wallet_index(&self) -> Result<i32, GemServiceError> {
        self.wallets.next_wallet_index()
    }

    pub async fn import_wallet(&self, name: String, import: GemWalletImportType, source: WalletSource) -> Result<GemWalletImportResult, GemServiceError> {
        self.wallets.import_wallet(name, import, source).await
    }

    pub async fn rename(&self, wallet_id: WalletId, name: String) -> Result<(), GemServiceError> {
        self.wallets.rename(wallet_id, name).await
    }

    pub async fn setup_chains(&self, chains: Vec<Chain>) -> Result<Vec<Wallet>, GemServiceError> {
        self.wallets.setup_chains(chains).await
    }

    pub fn wallets(&self) -> Result<Vec<Wallet>, GemServiceError> {
        self.wallets.wallets()
    }

    pub fn set_current_wallet(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.session.set_current_wallet_id(Some(wallet_id))
    }

    pub fn validate_recipient(&self, chain: Chain, input: String, name_record: Option<NameRecord>) -> GemRecipientValidation {
        self.names.validate_recipient(chain, input, name_record)
    }

    pub fn recipient(
        &self,
        chain: Chain,
        input: String,
        name_record: Option<NameRecord>,
        memo: Option<String>,
        references: Vec<String>,
    ) -> Result<GemRecipient, GemRecipientError> {
        self.names.recipient(chain, input, name_record, memo, references)
    }

    pub fn is_name_supported(&self, name: String) -> bool {
        self.names.is_name_supported(name)
    }

    pub fn name_record_debounce_milliseconds(&self) -> u64 {
        self.names.name_record_debounce_milliseconds()
    }

    pub async fn get_name_record(&self, name: String, chain: Chain) -> Result<Option<NameRecord>, GemServiceError> {
        self.names.get_name_record(name, chain).await
    }
}
