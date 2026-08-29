pub mod rules;

use primitives::Chain;

#[derive(Default, uniffi::Object)]
pub struct GemChainService {}

#[uniffi::export]
impl GemChainService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_chains(&self, query: String) -> Vec<Chain> {
        rules::matching_chains(rules::chains_by_rank(), &query)
    }

    pub fn get_matching_chains(&self, chains: Vec<Chain>, query: String) -> Vec<Chain> {
        rules::matching_chains(chains, &query)
    }

    pub fn is_valid_network_id(&self, chain: Chain, network_id: String) -> bool {
        rules::is_valid_network_id(chain, &network_id)
    }
}
