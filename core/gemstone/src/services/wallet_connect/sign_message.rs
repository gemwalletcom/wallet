use std::sync::Arc;

use primitives::{Account, ApplicationMetadata, Asset, Chain, OptionStringExt, Wallet, WalletConnectionSessionProposal, WalletConnectionVerificationStatus, WalletId};

use crate::application;
use crate::keystore::{GemKeystore, decode_password, keystore_id_for_wallet};
use crate::message::sign_type::{MessageType, SignMessage};
use crate::message::signer::MessageSigner;
use crate::models::copy::address_copy;
use crate::models::list::{GemGroup, GemListRow, GemListRowTitle};
use crate::services::assets::rules::asset_text;
use crate::services::confirm::GemSimulationValue;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::localization::GemLocalizedText;
use crate::services::name::GemNameService;
use crate::services::simulation::{address_requests, named_payload_rows, payload_fields, warning_rows};
use crate::services::wallet::GemKeystorePassword;
use crate::services::wallet::model::wallet_row;
use crate::services::wallet_connect::GemWalletConnectAuthMessage;
use crate::services::wallet_connect::model::{GemWalletConnectMessageRequest, authentication_sections};
use gem_wallet_connect::SignDigestType;
use primitives::BlockExplorerLink;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSignMessagePreview {
    pub title: GemLocalizedText,
    pub text: String,
    pub primary_fields: Vec<GemListRow>,
    pub secondary_fields: Vec<GemListRow>,
    pub has_critical_warning: bool,
    pub header: Option<GemSimulationValue>,
    pub rows: Vec<GemListRow>,
    pub warnings: Vec<GemListRow>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAuthenticationPreview {
    pub title: GemLocalizedText,
    pub text: String,
    pub primary_fields: Vec<GemListRow>,
    pub secondary_fields: Vec<GemListRow>,
    pub sections: Vec<GemGroup>,
    pub message: SignMessage,
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
        Self { names, explorer, keystore, password }
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
        let warnings = warning_rows(&simulation.warnings);
        let header = GemSimulationValue::from_simulation(&simulation, &assets);
        let fields = payload_fields(simulation.payload, header.is_some());
        let address_url = |chain, address| self.explorer.get_address_url(chain, address);
        let payload = signer.payload_preview(fields, address_url).ok().flatten();
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

    pub fn authentication_preview(&self, message: GemWalletConnectAuthMessage, proposal: WalletConnectionSessionProposal, wallet_id: WalletId, status: WalletConnectionVerificationStatus) -> GemAuthenticationPreview {
        authentication_preview(message, &proposal, wallet_id, status, |chain, address| self.explorer.get_address_url(chain, address))
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

fn authentication_preview(
    message: GemWalletConnectAuthMessage,
    proposal: &WalletConnectionSessionProposal,
    wallet_id: WalletId,
    status: WalletConnectionVerificationStatus,
    address_url: impl Fn(Chain, String) -> BlockExplorerLink,
) -> GemAuthenticationPreview {
    let chain = message.account.chain;
    let sign_message = SignMessage {
        chain,
        sign_type: SignDigestType::Siwe,
        data: message.message.as_bytes().to_vec(),
    };
    let payload = MessageSigner::new(sign_message.clone()).payload_preview(Vec::new(), address_url).ok().flatten();
    GemAuthenticationPreview {
        title: payload.as_ref().map(|preview| preview.message_type).unwrap_or(MessageType::Text).title(),
        text: message.message,
        primary_fields: payload.as_ref().map(|preview| preview.primary.clone()).unwrap_or_default(),
        secondary_fields: payload.map(|preview| preview.secondary).unwrap_or_default(),
        sections: authentication_sections(proposal, wallet_id, chain, status),
        message: sign_message,
    }
}

fn review_rows(chain: Chain, wallet: &Wallet, account: &Account, metadata: &ApplicationMetadata, shows_app: bool, address_url: impl Fn(Chain, String) -> BlockExplorerLink) -> Vec<GemListRow> {
    let app = shows_app.then(|| GemListRow::app(metadata.short_name(), application::icon_url(metadata), Some(metadata.url.clone()).non_empty()));
    let sender = GemListRow::wallet(wallet_row(wallet.clone()), address_copy(chain, account.address.clone()), address_url(chain, account.address.clone()));
    app.into_iter().chain([sender, network_row(chain)]).collect()
}

pub(crate) fn network_row(chain: Chain) -> GemListRow {
    GemListRow::Network {
        title: GemListRowTitle::Network,
        chain,
        name: asset_text(&Asset::from_chain(chain)).network_name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::list::GemRowMenuItem;
    use gem_evm::testkit::siwe_mock::mock_siwe_message;
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
        assert!(matches!(&with_header[0], GemListRow::App { name, menu, .. } if name == "Test Dapp" && matches!(menu.as_slice(), [GemRowMenuItem::Open { url, .. }] if url == "https://example.com")));
        assert!(matches!(&with_header[1], GemListRow::Wallet { menu, .. } if matches!(menu.first(), Some(GemRowMenuItem::Copy { copy }) if copy.value == account.address)));
        assert!(matches!(&with_header[2], GemListRow::Network { chain: Chain::Ethereum, .. }));

        let plain = review_rows(Chain::Ethereum, &wallet, &account, &ApplicationMetadata::mock(), false, link);
        assert_eq!(plain.len(), 2, "the app header already names the app");
        assert!(matches!(plain[0], GemListRow::Wallet { .. }));
    }
    #[test]
    fn test_authentication_preview() {
        let wallet = Wallet::mock_with_type(WalletType::Multicoin, &[Chain::Ethereum]);
        let account = wallet.account(Chain::Ethereum).unwrap().clone();
        let text = mock_siwe_message("example.org", 1);
        let proposal = WalletConnectionSessionProposal::mock_with_wallets(vec![wallet.clone()]);

        let preview = authentication_preview(
            GemWalletConnectAuthMessage { account, message: text.clone() },
            &proposal,
            wallet.id.clone(),
            WalletConnectionVerificationStatus::Verified,
            |_, address| BlockExplorerLink::mock_with_address(&address),
        );

        assert_eq!(
            preview.message,
            SignMessage {
                chain: Chain::Ethereum,
                sign_type: SignDigestType::Siwe,
                data: text.as_bytes().to_vec(),
            }
        );
        assert_eq!(preview.text, text);
        assert_eq!(preview.title, GemLocalizedText::SignInWith { chain: Chain::Ethereum });
        assert_eq!(preview.primary_fields.len(), 2);
        assert_eq!(preview.sections[0].rows()[1], network_row(Chain::Ethereum));
    }
}
