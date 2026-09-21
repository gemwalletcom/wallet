mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use block_explorers::get_block_explorers_by_chain;
use primitives::{BlockExplorerLink, Chain};

use crate::block_explorer::{Explorer, GemExplorerInput};
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
        let selected = self.preferences.get_explorer_name(chain);
        rules::selected_explorer(&self.get_explorers(chain), selected).unwrap_or_default()
    }

    pub fn set_explorer_name(&self, chain: Chain, name: String) -> Result<(), GemServiceError> {
        self.preferences.set_explorer_name(chain, name)
    }

    pub fn get_address_url(&self, chain: Chain, address: String) -> BlockExplorerLink {
        let name = self.get_explorer_name(chain);
        link(&name, Explorer { chain }.get_address_url(&name, &address))
    }

    pub fn get_token_url(&self, chain: Chain, address: String) -> Option<BlockExplorerLink> {
        let name = self.get_explorer_name(chain);
        Explorer { chain }.get_token_url(&name, &address).map(|url| link(&name, url))
    }

    pub fn get_explorers(&self, chain: Chain) -> Vec<String> {
        get_block_explorers_by_chain(chain.as_ref()).into_iter().map(|explorer| explorer.name()).collect()
    }
    pub fn get_transaction_link(&self, chain: Chain, hash: String, provider: Option<String>, recipient: Option<String>, memo: Option<String>) -> BlockExplorerLink {
        let name = self.get_explorer_name(chain);
        let explorer = Explorer { chain };
        provider
            .and_then(|provider| {
                let input = GemExplorerInput { hash: hash.clone(), recipient, memo };
                explorer.get_transaction_swap_url(&name, input, &provider)
            })
            .map(|url| link(&url.name, url.url))
            .unwrap_or_else(|| link(&name, explorer.get_transaction_url(&name, &hash)))
    }
    pub fn get_nft_url(&self, chain: Chain, contract_address: String, token_id: String) -> Option<BlockExplorerLink> {
        let name = self.get_explorer_name(chain);
        Explorer { chain }.get_nft_url(&name, &contract_address, &token_id).map(|url| link(&name, url))
    }
    pub fn get_validator_url(&self, chain: Chain, address: String) -> Option<BlockExplorerLink> {
        let name = self.get_explorer_name(chain);
        Explorer { chain }.get_validator_url(&name, &address).map(|url| link(&name, url))
    }
}

fn link(name: &str, url: String) -> BlockExplorerLink {
    BlockExplorerLink { name: name.to_string(), link: url }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_transaction_link_without_a_provider_uses_the_selected_explorer() {
        let service = GemExplorerService::mock();
        let selected = service.get_explorer_name(Chain::Ethereum);
        let link = service.get_transaction_link(Chain::Ethereum, "0xhash".to_string(), None, None, None);

        assert_eq!(link.name, selected);
        assert!(link.link.contains("0xhash"), "{}", link.link);
    }

    #[test]
    fn test_a_swap_provider_names_its_own_explorer_and_an_unknown_one_falls_back() {
        let service = GemExplorerService::mock();
        let selected = service.get_explorer_name(Chain::Ethereum);

        let across = service.get_transaction_link(Chain::Ethereum, "0xhash".to_string(), Some("across".to_string()), None, None);
        let unknown = service.get_transaction_link(Chain::Ethereum, "0xhash".to_string(), Some("not-a-provider".to_string()), None, None);

        assert_ne!(across.name, selected, "a cross-chain swap is followed on the provider's explorer");
        assert_eq!(unknown.name, selected, "an unreadable provider falls back to the selected explorer");
        assert!(unknown.link.contains("0xhash"), "{}", unknown.link);
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
