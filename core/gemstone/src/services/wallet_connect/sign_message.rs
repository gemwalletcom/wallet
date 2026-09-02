use std::sync::Arc;

use primitives::{AddressName, Chain, ChainAddress, SimulationPayloadField, SimulationPayloadFieldType, SimulationResult};

use crate::block_explorer::GemBlockExplorerLink;
use crate::message::sign_type::SignMessage;
use crate::message::signer::MessageSigner;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSignMessagePreview {
    pub text: String,
    pub primary_fields: Vec<SimulationPayloadField>,
    pub secondary_fields: Vec<SimulationPayloadField>,
}

#[derive(uniffi::Object)]
pub struct GemSignMessageService {
    names: Arc<GemNameService>,
    explorer: Arc<GemExplorerService>,
}

#[uniffi::export]
impl GemSignMessageService {
    #[uniffi::constructor]
    pub fn new(names: Arc<GemNameService>, explorer: Arc<GemExplorerService>) -> Self {
        Self { names, explorer }
    }

    pub fn preview(&self, message: SignMessage, simulation: SimulationResult) -> GemSignMessagePreview {
        let signer = MessageSigner::new(message);
        let payload = signer.payload_preview(simulation.payload).ok().flatten();
        GemSignMessagePreview {
            text: signer.plain_preview(),
            primary_fields: payload.as_ref().map(|preview| preview.primary.clone()).unwrap_or_default(),
            secondary_fields: payload.map(|preview| preview.secondary).unwrap_or_default(),
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
