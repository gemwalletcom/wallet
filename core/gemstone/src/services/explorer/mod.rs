mod rules;

use std::sync::Arc;

use primitives::{
    Chain,
    block_explorer::{BlockExplorerLink, get_block_explorers_by_chain},
};

use crate::block_explorer::{Explorer, GemBlockExplorerLink, GemExplorerInput};
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

    pub fn get_explorers(&self, chain: Chain) -> Vec<String> {
        get_block_explorers_by_chain(chain.as_ref()).into_iter().map(|explorer| explorer.name()).collect()
    }

    pub fn get_explorer_name(&self, chain: Chain) -> String {
        let selected = self.preferences.get_explorer_name(chain).ok().flatten();
        rules::selected_explorer(&self.get_explorers(chain), selected).unwrap_or_default()
    }

    pub fn set_explorer_name(&self, chain: Chain, name: String) -> Result<(), GemServiceError> {
        self.preferences.set_explorer_name(chain, name)
    }

    pub fn get_transaction_url(&self, chain: Chain, hash: String) -> GemBlockExplorerLink {
        let name = self.get_explorer_name(chain);
        link(&name, Explorer { chain }.get_transaction_url(&name, &hash))
    }

    pub fn get_transaction_link(&self, chain: Chain, hash: String, provider: Option<String>, recipient: Option<String>, memo: Option<String>) -> GemBlockExplorerLink {
        let name = self.get_explorer_name(chain);
        let explorer = Explorer { chain };
        provider
            .and_then(|provider| {
                let input = GemExplorerInput {
                    hash: hash.clone(),
                    recipient,
                    memo,
                };
                explorer.get_transaction_swap_url(&name, input, &provider)
            })
            .map(|url| link(&url.name, url.url))
            .unwrap_or_else(|| link(&name, explorer.get_transaction_url(&name, &hash)))
    }

    pub fn get_address_url(&self, chain: Chain, address: String) -> GemBlockExplorerLink {
        let name = self.get_explorer_name(chain);
        link(&name, Explorer { chain }.get_address_url(&name, &address))
    }

    pub fn get_token_url(&self, chain: Chain, address: String) -> Option<GemBlockExplorerLink> {
        let name = self.get_explorer_name(chain);
        Explorer { chain }.get_token_url(&name, &address).map(|url| link(&name, url))
    }

    pub fn get_nft_url(&self, chain: Chain, contract_address: String, token_id: String) -> Option<GemBlockExplorerLink> {
        let name = self.get_explorer_name(chain);
        Explorer { chain }.get_nft_url(&name, &contract_address, &token_id).map(|url| link(&name, url))
    }

    pub fn get_validator_url(&self, chain: Chain, address: String) -> Option<GemBlockExplorerLink> {
        let name = self.get_explorer_name(chain);
        Explorer { chain }.get_validator_url(&name, &address).map(|url| link(&name, url))
    }
}

fn link(name: &str, url: String) -> GemBlockExplorerLink {
    BlockExplorerLink {
        name: name.to_string(),
        link: url,
    }
}
