use crate::models::button::GemButtonState;
use std::iter::once;
use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Wallet};

use super::rules;
use crate::models::list::{GemListRow, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle, GemNoticeKind};
use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::error::{GemServiceError, required_account};
use crate::services::explorer::GemExplorerService;
use crate::services::localization::GemLocalizedText;
use chain_primitives::checksum_address;
use primitives::BlockExplorerLink;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAddAssetPhase {
    Idle,
    Loading,
    Found { asset: Asset },
    Failed,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddAssetViewState {
    pub phase: GemAddAssetPhase,
    pub can_add: bool,
    pub button: GemButtonState,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddAssetChains {
    pub chains: Vec<Chain>,
    pub default_chain: Option<Chain>,
    pub shows_picker: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddAssetSession {
    pub chain: Option<Chain>,
    pub address: String,
    pub asset: Option<Asset>,
    pub is_loading: bool,
    pub is_adding: bool,
    pub failed: bool,
}

impl GemAddAssetSession {
    pub fn new(chain: Option<Chain>) -> Self {
        Self {
            chain,
            address: String::new(),
            asset: None,
            is_loading: false,
            is_adding: false,
            failed: false,
        }
    }

    fn sections(&self, explorer: Option<BlockExplorerLink>) -> Vec<GemListSection> {
        let section = |rows: Vec<GemListRow>| GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows,
        };
        let text = |title: GemListRowTitle, value: String| GemListRow::Text { title, value };
        match &self.asset {
            Some(asset) => once(section(vec![
                text(GemListRowTitle::Name, asset.name.clone()),
                text(GemListRowTitle::Symbol, asset.symbol.clone()),
                text(GemListRowTitle::Decimals, asset.decimals.to_string()),
                text(GemListRowTitle::Type, asset.asset_type.as_ref().to_string()),
            ]))
            .chain(explorer.map(|link| section(vec![GemListRow::Explorer { name: link.name, url: link.link }])))
            .collect(),
            None if self.failed => vec![section(vec![GemListRow::Notice {
                title: GemListRowTitle::Error,
                message: Some(GemLocalizedText::InvalidTokenId),
                kind: GemNoticeKind::Error,
            }])],
            None => Vec::new(),
        }
    }

    fn cleared(&self, chain: Option<Chain>, address: String) -> Self {
        Self {
            chain,
            address,
            asset: None,
            is_loading: false,
            is_adding: false,
            failed: false,
        }
    }
}

#[uniffi::export]
impl GemAddAssetSession {
    pub fn on_chain(&self, chain: Option<Chain>) -> Self {
        self.cleared(chain, self.address.clone())
    }

    pub fn on_address(&self, address: String) -> Self {
        self.cleared(self.chain, address.trim().to_string())
    }

    pub fn on_loading(&self) -> Self {
        Self {
            is_loading: self.searches_token(),
            ..self.cleared(self.chain, self.address.clone())
        }
    }

    pub fn on_adding(&self, is_adding: bool) -> Self {
        Self { is_adding, ..self.clone() }
    }

    pub fn on_found(&self, chain: Chain, address: String, asset: Asset) -> Self {
        if self.chain != Some(chain) || address.trim() != self.address {
            return self.clone();
        }
        Self {
            asset: Some(asset),
            is_loading: false,
            failed: false,
            ..self.clone()
        }
    }

    pub fn on_failed(&self, chain: Chain, address: String) -> Self {
        if self.chain != Some(chain) || address.trim() != self.address {
            return self.clone();
        }
        Self {
            asset: None,
            is_loading: false,
            failed: true,
            ..self.clone()
        }
    }

    pub fn searches_token(&self) -> bool {
        self.chain.is_some() && !self.address.is_empty()
    }

    pub fn view_state(&self) -> GemAddAssetViewState {
        let phase = if self.is_loading {
            GemAddAssetPhase::Loading
        } else if let Some(asset) = &self.asset {
            GemAddAssetPhase::Found { asset: asset.clone() }
        } else if self.failed {
            GemAddAssetPhase::Failed
        } else {
            GemAddAssetPhase::Idle
        };
        let can_add = matches!(phase, GemAddAssetPhase::Found { .. });
        let button = match (self.is_loading || self.is_adding, can_add) {
            (true, _) => GemButtonState::Loading,
            (false, true) => GemButtonState::Enabled,
            (false, false) => GemButtonState::Disabled,
        };
        GemAddAssetViewState { can_add, button, phase }
    }
}

#[derive(uniffi::Object)]
pub struct GemAddAssetService {
    assets: Arc<GemAssetsService>,
    balances: Arc<GemBalanceService>,
    explorer: Arc<GemExplorerService>,
}

#[uniffi::export]
impl GemAddAssetService {
    #[uniffi::constructor]
    pub fn new(assets: Arc<GemAssetsService>, balances: Arc<GemBalanceService>, explorer: Arc<GemExplorerService>) -> Self {
        Self { assets, balances, explorer }
    }

    pub fn new_session(&self, chain: Option<Chain>) -> GemAddAssetSession {
        GemAddAssetSession::new(chain)
    }

    pub fn chain_picker(&self, wallet: Wallet) -> GemAddAssetChains {
        let chains = rules::token_chains(&wallet);
        GemAddAssetChains {
            default_chain: rules::default_token_chain(&chains),
            shows_picker: chains.len() > 1,
            chains,
        }
    }

    pub fn sections(&self, session: GemAddAssetSession) -> Vec<GemListSection> {
        let explorer = session.asset.as_ref().and_then(|asset| self.explorer.get_token_url(asset.id.chain, asset.id.token_id.clone()?));
        session.sections(explorer)
    }

    pub async fn token(&self, chain: Chain, address: String) -> Result<Asset, GemServiceError> {
        self.assets.ensure_token_asset(AssetId::from(chain, Some(checksum_address(&address, chain)))).await
    }

    pub async fn add(&self, wallet: Wallet, asset_id: AssetId) -> Result<(), GemServiceError> {
        required_account(&wallet, asset_id.chain)?;
        let asset = self.assets.ensure_token_asset(asset_id).await?;
        self.balances.enable_assets(wallet.id, vec![asset.id]).await
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;

    #[test]
    fn test_a_token_is_searched_only_with_a_chain_and_an_address() {
        let session = GemAddAssetSession::new(Some(Chain::Ethereum));

        assert!(!session.searches_token(), "an empty address searches nothing");
        assert!(!session.on_address("   ".to_string()).searches_token(), "whitespace is not an address");
        assert!(session.on_address("0xabc".to_string()).searches_token());
        assert!(!session.on_address("0xabc".to_string()).on_chain(None).searches_token());
    }

    #[test]
    fn test_a_new_address_drops_the_token_found_for_the_previous_one() {
        let found = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xabc".to_string()).on_found(Chain::Ethereum, "0xabc".to_string(), Asset::mock());
        assert!(found.view_state().can_add);

        let retyped = found.on_address("0xdef".to_string());
        assert_eq!(retyped.view_state().phase, GemAddAssetPhase::Idle);
        assert!(!retyped.view_state().can_add, "a token that was never looked up cannot be added");
    }

    #[test]
    fn test_the_button_spins_while_looking_up_or_adding_and_enables_on_a_found_token() {
        let idle = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xabc".to_string());
        assert_eq!(idle.view_state().button, GemButtonState::Disabled);
        assert_eq!(idle.on_loading().view_state().button, GemButtonState::Loading);

        let found = idle.on_found(Chain::Ethereum, "0xabc".to_string(), Asset::mock());
        assert_eq!(found.view_state().button, GemButtonState::Enabled);
        assert_eq!(found.on_adding(true).view_state().button, GemButtonState::Loading);
        assert_eq!(found.on_adding(true).on_adding(false).view_state().button, GemButtonState::Enabled);
    }

    #[test]
    fn test_switching_chain_starts_over() {
        let found = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xabc".to_string()).on_found(Chain::Ethereum, "0xabc".to_string(), Asset::mock());

        assert_eq!(found.on_chain(Some(Chain::SmartChain)).view_state().phase, GemAddAssetPhase::Idle);
    }

    #[test]
    fn test_a_lookup_for_an_earlier_address_is_ignored() {
        let session = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xdef".to_string()).on_loading();

        assert_eq!(session.on_found(Chain::Ethereum, "0xabc".to_string(), Asset::mock()), session);
        assert_eq!(session.on_failed(Chain::Ethereum, "0xabc".to_string()), session);
        assert!(session.on_found(Chain::Ethereum, " 0xdef ".to_string(), Asset::mock()).view_state().can_add);
    }

    #[test]
    fn test_a_lookup_for_the_chain_the_user_left_is_ignored() {
        let address = "0xabc".to_string();
        let session = GemAddAssetSession::new(Some(Chain::SmartChain)).on_address(address.clone()).on_loading();

        assert_eq!(session.on_found(Chain::Ethereum, address.clone(), Asset::mock()), session, "the same contract text on another network is another token");
        assert_eq!(session.on_failed(Chain::Ethereum, address.clone()), session);
        assert!(session.on_found(Chain::SmartChain, address, Asset::mock()).view_state().can_add);
    }

    #[test]
    fn test_a_failed_lookup_is_not_an_empty_screen() {
        let failed = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xabc".to_string()).on_failed(Chain::Ethereum, "0xabc".to_string());

        assert_eq!(failed.view_state().phase, GemAddAssetPhase::Failed);
        assert!(!failed.view_state().can_add);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(rows: Vec<GemListRow>) -> GemListSection {
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows,
        }
    }

    #[test]
    fn test_sections_describe_a_found_asset_and_nothing_before_it() {
        let session = GemAddAssetSession::new(Some(Chain::Ethereum));
        assert!(session.sections(None).is_empty(), "there is nothing to describe until a token is found");

        let text = |title: GemListRowTitle, value: &str| GemListRow::Text { title, value: value.to_string() };
        let link = BlockExplorerLink {
            name: "Etherscan".to_string(),
            link: "https://etherscan.io/token/0xabc".to_string(),
        };
        assert_eq!(
            session.on_found(Chain::Ethereum, String::new(), Asset::from_chain(Chain::Ethereum)).sections(Some(link)),
            vec![
                section(vec![
                    text(GemListRowTitle::Name, "Ethereum"),
                    text(GemListRowTitle::Symbol, "ETH"),
                    text(GemListRowTitle::Decimals, "18"),
                    text(GemListRowTitle::Type, "NATIVE"),
                ]),
                section(vec![GemListRow::Explorer {
                    name: "Etherscan".to_string(),
                    url: "https://etherscan.io/token/0xabc".to_string(),
                }]),
            ]
        );
    }

    #[test]
    fn test_a_failed_lookup_reads_as_an_invalid_token_id() {
        let failed = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xabc".to_string()).on_failed(Chain::Ethereum, "0xabc".to_string());

        assert_eq!(
            failed.sections(None),
            vec![section(vec![GemListRow::Notice {
                title: GemListRowTitle::Error,
                message: Some(GemLocalizedText::InvalidTokenId),
                kind: GemNoticeKind::Error,
            }])]
        );
        assert!(failed.on_address("0xdef".to_string()).sections(None).is_empty(), "retyping clears the failure");
    }
}
