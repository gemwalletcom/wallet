use std::sync::Arc;

use primitives::{Chain, ContactData, Wallet};

use super::model::{GemRecipientError, GemRecipientNext, GemRecipientScan, GemRecipientSection, GemRecipientType};
use super::rules::{contact_recipients, recipient_sections, scan_route, select_step};
use crate::models::payment::GemPayment;
use crate::payment::{GemPaymentService, asset_step};
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

    pub fn recipient_sections(&self, wallets: Vec<Wallet>, chain: Chain, contacts: Vec<ContactData>) -> Vec<GemRecipientSection> {
        let current = self.session.get_current_wallet_id().unwrap_or_default();
        let others = wallets.into_iter().filter(|wallet| Some(&wallet.id) != current.as_ref()).collect();
        recipient_sections(others, chain, contact_recipients(contacts, chain))
    }

    pub fn scan(&self, url: String, recipient_type: GemRecipientType) -> Result<GemRecipientScan, GemRecipientError> {
        let asset = recipient_type.asset();
        let step = match self.payments.decode_url(url) {
            Ok(GemPayment::Request { request }) => asset_step(&request, &asset),
            Ok(GemPayment::Link { .. }) | Err(_) => None,
        };
        let step = step.ok_or(GemRecipientError::InvalidAddress { chain: asset.chain() })?;
        Ok(scan_route(step, &recipient_type))
    }

    pub fn select(&self, recipient_type: GemRecipientType, recipient: GemRecipient) -> Result<GemRecipientNext, GemRecipientError> {
        select_step(recipient_type, recipient)
    }
}
