use std::sync::Arc;

use primitives::{AddressName, Chain, ChainAddress, SimulationPayloadField, SimulationPayloadFieldType, SimulationResult, WalletId};

use crate::block_explorer::GemBlockExplorerLink;
use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::message::sign_type::SignMessage;
use crate::message::signer::MessageSigner;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::wallet::GemKeystorePassword;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSignMessagePreview {
    pub text: String,
    pub primary_fields: Vec<SimulationPayloadField>,
    pub secondary_fields: Vec<SimulationPayloadField>,
    pub has_critical_warning: bool,
}

#[derive(uniffi::Object)]
pub struct GemSignMessageService {
    names: Arc<GemNameService>,
    explorer: Arc<GemExplorerService>,
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
}

#[uniffi::export]
impl GemSignMessageService {
    #[uniffi::constructor]
    pub fn new(names: Arc<GemNameService>, explorer: Arc<GemExplorerService>, keystore: Arc<GemKeystore>, password: Arc<dyn GemKeystorePassword>) -> Self {
        Self {
            names,
            explorer,
            keystore,
            password,
        }
    }

    pub async fn sign(&self, wallet_id: WalletId, message: SignMessage) -> Result<String, GemServiceError> {
        let password = decode_password(&self.password.get_password(false)?);
        Ok(MessageSigner::new(message).sign_with_keystore(self.keystore.clone(), keystore_id_for_wallet(wallet_id.id()), password)?)
    }

    pub fn preview(&self, message: SignMessage, simulation: SimulationResult) -> GemSignMessagePreview {
        let signer = MessageSigner::new(message);
        let has_critical_warning = simulation.has_critical_warning();
        let payload = signer.payload_preview(simulation.payload).ok().flatten();
        GemSignMessagePreview {
            text: signer.plain_preview(),
            primary_fields: payload.as_ref().map(|preview| preview.primary.clone()).unwrap_or_default(),
            secondary_fields: payload.map(|preview| preview.secondary).unwrap_or_default(),
            has_critical_warning,
        }
    }

    pub async fn address_names(&self, chain: Chain, preview: GemSignMessagePreview) -> Vec<AddressName> {
        let requests: Vec<ChainAddress> = preview
            .primary_fields
            .iter()
            .chain(preview.secondary_fields.iter())
            .filter(|field| field.field_type == SimulationPayloadFieldType::Address)
            .map(|field| ChainAddress::new(chain, field.value.clone()))
            .collect();
        if requests.is_empty() {
            return Vec::new();
        }
        self.names.get_address_names(requests).await.unwrap_or_default()
    }

    pub fn address_url(&self, chain: Chain, address: String) -> GemBlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }
}
