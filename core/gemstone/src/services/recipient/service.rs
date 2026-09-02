use std::sync::Arc;

use primitives::name::NameRecord;
use primitives::{Asset, Chain, Wallet, WalletId};

use super::model::{GemRecipientError, GemRecipientValidation};
use crate::GemstoneError;
use crate::models::payment::GemPayment;
use crate::payment::{GemPaymentConfirmTransfer, GemPaymentDestination, GemPaymentService, GemPaymentWalletAsset};
use crate::services::error::GemServiceError;
use crate::services::name::GemNameService;
use crate::services::transfer::model::{GemRecipient, GemTransferData};
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemRecipientService {
    names: Arc<GemNameService>,
    payments: Arc<GemPaymentService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemRecipientService {
    #[uniffi::constructor]
    pub fn new(names: Arc<GemNameService>, payments: Arc<GemPaymentService>, session: Arc<GemWalletSessionService>) -> Self {
        Self { names, payments, session }
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

    pub async fn get_name_record(&self, name: String, chain: Chain) -> Result<Option<NameRecord>, GemServiceError> {
        self.names.get_name_record(name, chain).await
    }

    pub fn other_wallets(&self, wallet_id: WalletId) -> Result<Vec<Wallet>, GemServiceError> {
        Ok(self.session.get_wallets()?.into_iter().filter(|wallet| wallet.id != wallet_id).collect())
    }

    pub fn scan_destination(&self, url: String, asset: GemPaymentWalletAsset) -> Result<GemPaymentDestination, GemstoneError> {
        Ok(match self.payments.decode_url(url)? {
            GemPayment::Request(request) => self.payments.transfer_destination(request, asset),
            GemPayment::Link(_) => GemPaymentDestination::Unsupported,
        })
    }

    pub fn transfer_data(&self, transfer: GemPaymentConfirmTransfer, asset: Asset) -> GemTransferData {
        self.payments.transfer_data(transfer, asset)
    }
}
