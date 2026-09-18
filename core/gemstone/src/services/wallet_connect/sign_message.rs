use std::sync::Arc;

use primitives::{Asset, Chain, SimulationResult, WalletId};

use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::message::sign_type::{MessageType, SignMessage};
use crate::message::signer::MessageSigner;
use crate::services::confirm::GemSimulationValue;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::simulation::{GemSimulationFormatter, GemSimulationPayloadRow, address_requests, named_payload_rows};
use crate::services::wallet::GemKeystorePassword;
use primitives::BlockExplorerLink;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSignMessagePreview {
    pub message_type: MessageType,
    pub text: String,
    pub primary_fields: Vec<GemSimulationPayloadRow>,
    pub secondary_fields: Vec<GemSimulationPayloadRow>,
    pub has_critical_warning: bool,
    pub header: Option<GemSimulationValue>,
}

#[derive(uniffi::Object)]
pub struct GemSignMessageService {
    names: Arc<GemNameService>,
    explorer: Arc<GemExplorerService>,
    keystore: Arc<GemKeystore>,
    password: Arc<dyn GemKeystorePassword>,
    simulation_formatter: GemSimulationFormatter,
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
            simulation_formatter: GemSimulationFormatter::new(),
        }
    }

    pub async fn sign(&self, wallet_id: WalletId, message: SignMessage) -> Result<String, GemServiceError> {
        let password = decode_password(&self.password.get_password(false)?);
        Ok(MessageSigner::new(message).sign_with_keystore(self.keystore.clone(), keystore_id_for_wallet(wallet_id.id()), password)?)
    }

    pub fn preview(&self, message: SignMessage, simulation: SimulationResult, assets: Vec<Asset>) -> GemSignMessagePreview {
        let signer = MessageSigner::new(message);
        let has_critical_warning = simulation.has_critical_warning();
        let header = GemSimulationValue::from_simulation(&simulation, &assets);
        let payload_fields = self.simulation_formatter.payload_fields(simulation.payload, header.is_some());
        let payload = signer.payload_preview(payload_fields).ok().flatten();
        GemSignMessagePreview {
            message_type: payload.as_ref().map(|preview| preview.message_type).unwrap_or(MessageType::Text),
            text: signer.plain_preview(),
            primary_fields: payload.as_ref().map(|preview| preview.primary.clone()).unwrap_or_default(),
            secondary_fields: payload.map(|preview| preview.secondary).unwrap_or_default(),
            has_critical_warning,
            header,
        }
    }

    pub async fn with_address_names(&self, chain: Chain, preview: GemSignMessagePreview) -> GemSignMessagePreview {
        let requests = [address_requests(&preview.primary_fields, chain), address_requests(&preview.secondary_fields, chain)].concat();
        if requests.is_empty() {
            return preview;
        }
        let names = self.names.get_address_names(requests).await.unwrap_or_default();
        GemSignMessagePreview {
            primary_fields: named_payload_rows(preview.primary_fields, Some(chain), &names),
            secondary_fields: named_payload_rows(preview.secondary_fields, Some(chain), &names),
            ..preview
        }
    }

    pub fn address_url(&self, chain: Chain, address: String) -> BlockExplorerLink {
        self.explorer.get_address_url(chain, address)
    }
}
