use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Wallet};

use super::rules;
use crate::address::checksum_address;
use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::chain::rules::matching_chains;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use primitives::BlockExplorerLink;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAddAssetPhase {
    Idle,
    Loading,
    Found { asset: Asset },
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAssetInfoKind {
    Name,
    Symbol,
    Decimals,
    Kind,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAssetInfoRow {
    pub kind: GemAssetInfoKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddAssetViewState {
    pub phase: GemAddAssetPhase,
    pub can_add: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddAssetSession {
    pub chain: Option<Chain>,
    pub address: String,
    pub asset: Option<Asset>,
    pub is_loading: bool,
    pub failed: bool,
}

impl GemAddAssetSession {
    pub fn new(chain: Option<Chain>) -> Self {
        Self {
            chain,
            address: String::new(),
            asset: None,
            is_loading: false,
            failed: false,
        }
    }

    fn cleared(&self, chain: Option<Chain>, address: String) -> Self {
        Self {
            chain,
            address,
            asset: None,
            is_loading: false,
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

    pub fn on_found(&self, asset: Asset) -> Self {
        Self {
            asset: Some(asset),
            is_loading: false,
            failed: false,
            ..self.clone()
        }
    }

    pub fn on_failed(&self) -> Self {
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

    pub fn asset_rows(&self) -> Vec<GemAssetInfoRow> {
        let Some(asset) = &self.asset else {
            return Vec::new();
        };
        let row = |kind: GemAssetInfoKind, value: String| GemAssetInfoRow { kind, value };
        vec![
            row(GemAssetInfoKind::Name, asset.name.clone()),
            row(GemAssetInfoKind::Symbol, asset.symbol.clone()),
            row(GemAssetInfoKind::Decimals, asset.decimals.to_string()),
            row(GemAssetInfoKind::Kind, asset.asset_type.as_ref().to_string()),
        ]
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
        GemAddAssetViewState {
            can_add: matches!(phase, GemAddAssetPhase::Found { .. }),
            phase,
        }
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

    pub fn chains(&self, wallet: Wallet) -> Vec<Chain> {
        rules::token_chains(&wallet)
    }

    pub fn default_chain(&self, chains: Vec<Chain>) -> Option<Chain> {
        rules::default_token_chain(&chains)
    }

    pub fn matching_chains(&self, chains: Vec<Chain>, query: String) -> Vec<Chain> {
        matching_chains(chains, &query)
    }

    pub fn token_url(&self, chain: Chain, token_id: String) -> Option<BlockExplorerLink> {
        self.explorer.get_token_url(chain, token_id)
    }

    pub async fn token(&self, chain: Chain, address: String) -> Result<Asset, GemServiceError> {
        self.assets.ensure_token_asset(AssetId::from(chain, Some(checksum_address(&address, chain)))).await
    }

    pub async fn add(&self, wallet: Wallet, asset_id: AssetId) -> Result<(), GemServiceError> {
        if wallet.account(asset_id.chain).is_none() {
            return Err(GemServiceError::NotFound {
                msg: format!("wallet has no account for {}", asset_id.chain),
            });
        }
        let asset = self.assets.ensure_token_asset(asset_id).await?;
        self.balances.set_assets_enabled(wallet.id, vec![asset.id], true).await
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
        let found = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xabc".to_string()).on_found(Asset::mock());
        assert!(found.view_state().can_add);

        let retyped = found.on_address("0xdef".to_string());
        assert_eq!(retyped.view_state().phase, GemAddAssetPhase::Idle);
        assert!(!retyped.view_state().can_add, "a token that was never looked up cannot be added");
    }

    #[test]
    fn test_switching_chain_starts_over() {
        let found = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xabc".to_string()).on_found(Asset::mock());

        assert_eq!(found.on_chain(Some(Chain::SmartChain)).view_state().phase, GemAddAssetPhase::Idle);
    }

    #[test]
    fn test_a_failed_lookup_is_not_an_empty_screen() {
        let failed = GemAddAssetSession::new(Some(Chain::Ethereum)).on_address("0xabc".to_string()).on_failed();

        assert_eq!(failed.view_state().phase, GemAddAssetPhase::Failed);
        assert!(!failed.view_state().can_add);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_rows_describe_a_found_asset_and_nothing_before_it() {
        let session = GemAddAssetSession::new(Some(Chain::Ethereum));
        assert!(session.asset_rows().is_empty(), "there is nothing to describe until a token is found");

        let rows = session.on_found(Asset::from_chain(Chain::Ethereum)).asset_rows();
        assert_eq!(
            rows.iter().map(|row| row.kind).collect::<Vec<_>>(),
            vec![GemAssetInfoKind::Name, GemAssetInfoKind::Symbol, GemAssetInfoKind::Decimals, GemAssetInfoKind::Kind]
        );
        assert_eq!(rows[0].value, "Ethereum");
        assert_eq!(rows[2].value, "18");
    }
}
