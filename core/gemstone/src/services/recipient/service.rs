use std::sync::Arc;

use primitives::{Chain, Wallet};

use super::model::{GemRecipientError, GemRecipientNext, GemRecipientScan, GemRecipientSection, GemRecipientType};
use super::rules::{recipient_sections, scan_route, select_step};
use crate::GemstoneError;
use crate::models::payment::GemPayment;
use crate::payment::{GemPaymentDestination, GemPaymentService, GemPaymentWalletAsset};
use crate::services::transfer::model::GemRecipient;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemRecipientService {
    payments: Arc<GemPaymentService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemRecipientService {
    #[uniffi::constructor]
    pub fn new(payments: Arc<GemPaymentService>, session: Arc<GemWalletSessionService>) -> Self {
        Self { payments, session }
    }

    pub fn recipient_sections(&self, wallets: Vec<Wallet>, chain: Chain, contacts: Vec<GemRecipient>) -> Vec<GemRecipientSection> {
        let current = self.session.get_current_wallet_id().unwrap_or_default();
        let others = wallets.into_iter().filter(|wallet| Some(&wallet.id) != current.as_ref()).collect();
        recipient_sections(others, chain, contacts)
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
            .map_err(|_| GemRecipientError::InvalidAddress { chain: asset.chain() })?;
        scan_route(destination, &recipient_type, |transfer| self.payments.transfer_data(transfer, asset))
    }

    pub fn select(&self, recipient_type: GemRecipientType, recipient: GemRecipient) -> Result<GemRecipientNext, GemRecipientError> {
        select_step(recipient_type, recipient)
    }
}

impl GemRecipientService {
    fn scan_destination(&self, url: String, asset: GemPaymentWalletAsset) -> Result<GemPaymentDestination, GemstoneError> {
        Ok(match self.payments.decode_url(url)? {
            GemPayment::Request { request } => self.payments.transfer_destination(request, asset),
            GemPayment::Link { link: _ } => GemPaymentDestination::Unsupported,
        })
    }
}
