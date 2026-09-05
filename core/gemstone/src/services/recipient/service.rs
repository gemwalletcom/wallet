use std::sync::Arc;

use primitives::name::NameRecord;
use primitives::{Chain, Wallet};

use super::model::{GemRecipientError, GemRecipientNext, GemRecipientScan, GemRecipientType};
use super::rules::{next_step, scan_route};
use crate::GemstoneError;
use crate::models::payment::GemPayment;
use crate::payment::{GemPaymentDestination, GemPaymentRecipient, GemPaymentService, GemPaymentWalletAsset};
use crate::services::name::GemNameService;
use crate::services::transfer::model::GemRecipient;
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

    pub fn recipient_wallets(&self, wallets: Vec<Wallet>) -> Vec<Wallet> {
        let current = self.session.get_current_wallet_id().unwrap_or_default();
        wallets.into_iter().filter(|wallet| Some(&wallet.id) != current.as_ref()).collect()
    }

    pub fn scan(&self, url: String, recipient_type: GemRecipientType) -> Result<GemRecipientScan, GemRecipientError> {
        let asset = recipient_type.asset();
        let destination = self
            .scan_destination(
                url,
                GemPaymentWalletAsset {
                    asset_id: asset.id.clone(),
                    decimals: asset.decimals,
                },
            )
            .map_err(|_| GemRecipientError::InvalidAddress)?;
        scan_route(destination, &recipient_type, |transfer| self.payments.transfer_data(transfer, asset))
    }

    pub fn next(&self, recipient_type: GemRecipientType, payment: GemPaymentRecipient) -> GemRecipientNext {
        next_step(recipient_type, payment)
    }
}

impl GemRecipientService {
    fn scan_destination(&self, url: String, asset: GemPaymentWalletAsset) -> Result<GemPaymentDestination, GemstoneError> {
        Ok(match self.payments.decode_url(url)? {
            GemPayment::Request(request) => self.payments.transfer_destination(request, asset),
            GemPayment::Link(_) => GemPaymentDestination::Unsupported,
        })
    }
}
