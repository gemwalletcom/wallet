#[cfg(test)]
pub(crate) mod testkit;

use std::str::FromStr;
use std::sync::Arc;

use block_explorers::{BlockExplorer, ExplorerInput, get_block_explorer, get_block_explorers, swap_explorer};
use primitives::{AssetId, BlockExplorerLink, Chain, SwapProvider};

use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;

#[derive(uniffi::Object)]
pub struct GemExplorerService {
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemExplorerService {
    #[uniffi::constructor]
    pub fn new(preferences: Arc<GemPreferencesService>) -> Self {
        Self { preferences }
    }
}

impl GemExplorerService {
    pub fn get_explorer_name(&self, chain: Chain) -> String {
        self.explorer(chain).name()
    }

    pub fn set_explorer_name(&self, chain: Chain, name: String) -> Result<(), GemServiceError> {
        self.preferences.set_explorer_name(chain, name)
    }

    pub fn get_address_url(&self, chain: Chain, address: String) -> BlockExplorerLink {
        let explorer = self.explorer(chain);
        link(explorer.as_ref(), explorer.get_address_url(&address))
    }

    pub fn get_token_url(&self, asset_id: &AssetId) -> Option<BlockExplorerLink> {
        let explorer = self.explorer(asset_id.chain);
        explorer.get_token_url(asset_id).map(|url| link(explorer.as_ref(), url))
    }

    pub fn get_explorers(&self, chain: Chain) -> Vec<String> {
        get_block_explorers(chain).into_iter().map(|explorer| explorer.name()).collect()
    }

    pub fn get_transaction_link(&self, chain: Chain, hash: String, provider: Option<String>, recipient: Option<String>, memo: Option<String>) -> BlockExplorerLink {
        match provider.and_then(|provider| SwapProvider::from_str(&provider).ok()).and_then(|provider| swap_explorer(provider, chain)) {
            Some(explorer) => link(explorer.as_ref(), explorer.get_swap_tx_url(&ExplorerInput { hash, recipient, memo })),
            None => {
                let explorer = self.explorer(chain);
                link(explorer.as_ref(), explorer.get_tx_url(&hash))
            }
        }
    }

    pub fn get_nft_url(&self, chain: Chain, contract_address: String, token_id: String) -> Option<BlockExplorerLink> {
        let explorer = self.explorer(chain);
        explorer.get_nft_url(&contract_address, &token_id).map(|url| link(explorer.as_ref(), url))
    }

    pub fn get_validator_url(&self, chain: Chain, address: String) -> Option<BlockExplorerLink> {
        let explorer = self.explorer(chain);
        explorer.get_validator_url(&address).map(|url| link(explorer.as_ref(), url))
    }

    fn explorer(&self, chain: Chain) -> Box<dyn BlockExplorer> {
        get_block_explorer(chain, &self.preferences.get_explorer_name(chain).unwrap_or_default())
    }
}

fn link(explorer: &dyn BlockExplorer, url: String) -> BlockExplorerLink {
    BlockExplorerLink { name: explorer.name(), link: url }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_transaction_link_without_a_provider_uses_the_selected_explorer() {
        let service = GemExplorerService::mock();

        assert_eq!(
            service.get_transaction_link(Chain::Ethereum, "0xhash".to_string(), None, None, None),
            BlockExplorerLink {
                name: "Etherscan".to_string(),
                link: "https://etherscan.io/tx/0xhash".to_string()
            }
        );
    }

    #[test]
    fn test_a_swap_provider_names_its_own_explorer_and_an_unknown_one_falls_back() {
        let service = GemExplorerService::mock();

        let across = service.get_transaction_link(Chain::Ethereum, "0xhash".to_string(), Some(SwapProvider::Across.as_ref().to_string()), None, None);
        let unknown = service.get_transaction_link(Chain::Ethereum, "0xhash".to_string(), Some("not-a-provider".to_string()), None, None);
        let chainflip = service.get_transaction_link(Chain::Polkadot, "0xhash".to_string(), Some(SwapProvider::Chainflip.as_ref().to_string()), None, None);

        assert_eq!(
            across,
            BlockExplorerLink {
                name: "Across".to_string(),
                link: "https://across.to/transfer/0xhash".to_string()
            },
            "a cross-chain swap is followed on the provider's explorer"
        );
        assert_eq!(
            unknown,
            BlockExplorerLink {
                name: "Etherscan".to_string(),
                link: "https://etherscan.io/tx/0xhash".to_string()
            },
            "an unreadable provider falls back to the selected explorer"
        );
        assert_eq!(
            chainflip,
            BlockExplorerLink {
                name: "SubScan".to_string(),
                link: "https://assethub-polkadot.subscan.io/extrinsic/0xhash".to_string()
            },
            "a provider without its own explorer falls back to the selected one"
        );
    }

    #[test]
    fn test_the_transaction_link_follows_the_explorer_the_user_picked() {
        let service = GemExplorerService::mock();
        let explorers = service.get_explorers(Chain::Ethereum);
        let other = explorers.last().unwrap().clone();
        service.set_explorer_name(Chain::Ethereum, other.clone()).unwrap();

        assert_eq!(service.get_transaction_link(Chain::Ethereum, "0xhash".to_string(), None, None, None).name, other);
        assert_eq!(service.get_address_url(Chain::Ethereum, "0xaddress".to_string()).name, other);
    }
}
