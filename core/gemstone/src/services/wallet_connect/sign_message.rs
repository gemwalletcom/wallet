use std::sync::Arc;

use primitives::{Account, ApplicationMetadata, Asset, Chain, Wallet, WalletId};

use crate::application;
use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::message::payload::MessagePayloadPreview;
use crate::message::sign_type::{MessageType, SignMessage};
use crate::message::signer::MessageSigner;
use crate::models::copy::address_copy;
use crate::models::list::{GemListRow, GemListRowTitle};
use crate::services::assets::rules::asset_text;
use crate::services::confirm::GemSimulationValue;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::localization::GemLocalizedText;
use crate::services::name::GemNameService;
use crate::services::simulation::{GemSimulationFormatter, GemSimulationPayloadRow, address_requests, named_payload_rows, simulation_warning_rows};
use crate::services::wallet::GemKeystorePassword;
use crate::services::wallet::model::wallet_row;
use crate::services::wallet_connect::model::GemWalletConnectMessageRequest;
use primitives::BlockExplorerLink;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSignMessagePreview {
    pub title: GemLocalizedText,
    pub text: String,
    pub primary_fields: Vec<GemSimulationPayloadRow>,
    pub secondary_fields: Vec<GemSimulationPayloadRow>,
    pub has_critical_warning: bool,
    pub header: Option<GemSimulationValue>,
    pub rows: Vec<GemListRow>,
    pub warnings: Vec<GemListRow>,
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

    pub fn preview(&self, request: GemWalletConnectMessageRequest) -> GemSignMessagePreview {
        let GemWalletConnectMessageRequest {
            chain,
            wallet,
            account,
            session,
            simulation,
            message,
            assets,
            ..
        } = request;
        let signer = MessageSigner::new(message);
        let has_critical_warning = simulation.has_critical_warning();
        let warnings = simulation_warning_rows(simulation.warnings.clone());
        let header = GemSimulationValue::from_simulation(&simulation, &assets);
        let payload_fields = self.simulation_formatter.payload_fields(simulation.payload, header.is_some());
        let address_url = |chain, address| self.explorer.get_address_url(chain, address);
        let payload = signer.payload_preview(payload_fields, address_url).ok().flatten();
        GemSignMessagePreview {
            title: payload.as_ref().map(|preview| preview.message_type).unwrap_or(MessageType::Text).title(),
            text: signer.plain_preview(),
            primary_fields: payload.as_ref().map(|preview| preview.primary.clone()).unwrap_or_default(),
            secondary_fields: payload.map(|preview| preview.secondary).unwrap_or_default(),
            has_critical_warning,
            rows: review_rows(chain, &wallet, &account, &session.metadata, header.is_some(), address_url),
            warnings,
            header,
        }
    }

    pub fn payload_preview(&self, message: SignMessage) -> Option<MessagePayloadPreview> {
        MessageSigner::new(message).payload_preview(Vec::new(), |chain, address| self.explorer.get_address_url(chain, address)).ok().flatten()
    }

    pub async fn with_address_names(&self, chain: Chain, preview: GemSignMessagePreview) -> GemSignMessagePreview {
        let requests = [address_requests(&preview.primary_fields, chain), address_requests(&preview.secondary_fields, chain)].concat();
        if requests.is_empty() {
            return preview;
        }
        let names = self.names.get_address_names(requests).await.unwrap_or_default();
        GemSignMessagePreview {
            primary_fields: named_payload_rows(preview.primary_fields, &names),
            secondary_fields: named_payload_rows(preview.secondary_fields, &names),
            ..preview
        }
    }
}

fn review_rows(chain: Chain, wallet: &Wallet, account: &Account, metadata: &ApplicationMetadata, shows_app: bool, address_url: impl Fn(Chain, String) -> BlockExplorerLink) -> Vec<GemListRow> {
    let app = shows_app.then(|| GemListRow::App {
        name: metadata.short_name(),
        icon_url: application::icon_url(metadata),
        website_url: Some(metadata.url.clone()).filter(|url| !url.is_empty()),
    });
    let sender = GemListRow::Wallet {
        wallet: wallet_row(wallet.clone()),
        copy: address_copy(chain, account.address.clone()),
        explorer: address_url(chain, account.address.clone()),
    };
    let network = GemListRow::Network {
        title: GemListRowTitle::Network,
        chain,
        name: asset_text(&Asset::from_chain(chain)).network_name,
    };
    app.into_iter().chain([sender, network]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::WalletType;

    #[test]
    fn test_review_rows_name_the_app_only_beside_a_header() {
        let wallet = Wallet::mock_with_type(WalletType::Multicoin, &[Chain::Ethereum]);
        let link = |chain: Chain, address: String| BlockExplorerLink {
            name: "Etherscan".to_string(),
            link: format!("https://etherscan.io/address/{address}?{chain}"),
        };

        let account = wallet.account(Chain::Ethereum).unwrap().clone();
        let with_header = review_rows(Chain::Ethereum, &wallet, &account, &ApplicationMetadata::mock(), true, link);
        assert!(matches!(&with_header[0], GemListRow::App { name, website_url: Some(url), .. } if name == "Test Dapp" && url == "https://example.com"));
        assert!(matches!(&with_header[1], GemListRow::Wallet { copy, .. } if copy.value == account.address));
        assert!(matches!(&with_header[2], GemListRow::Network { chain: Chain::Ethereum, .. }));

        let plain = review_rows(Chain::Ethereum, &wallet, &account, &ApplicationMetadata::mock(), false, link);
        assert_eq!(plain.len(), 2, "the app header already names the app");
        assert!(matches!(plain[0], GemListRow::Wallet { .. }));
    }
}
